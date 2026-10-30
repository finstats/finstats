# Muutosloki

Kaikki, mikä FinStatsissa on muuttunut, uusin ensin. Tämä tiedosto käännetään osaksi ohjelmaa,
ja sen näyttää sovelluksen **Muutosloki**-välilehti.

Muoto: `## [version] - date`, tai `## [version] - first day to release day` usean päivän aikana tehdylle
julkaisulle, ja sen jälkeen `### Added`, `### Changed`, `### Security`, `### Performance`, `### Stability`,
`### Fixed` tai `### Removed`, yksi kohta kutakin muutosta kohden. Tietoturva tarkoittaa, että joku olisi voinut
käyttää jotakin väärin, Suorituskyky, että jokin nopeutui tai keveni, Vakaus, että jokin ei enää voi kaatua,
pysähtyä, vuotaa tai hukata tietoja, ja Korjattu, että jokin antoi väärän vastauksen. `x.y.0`-julkaisun
otsikon alla on lyhyt otsikkorivi: sitä sovellus käyttää koko sarjan pääotsikkona.

## [2.3.1] - 2026-10-26 to 2026-10-28

### Security
- **Käyttöönotto kysyy kertakäyttöistä koodia**: kunnes FinStats on otettu käyttöön, se kirjoittaa käynnistyessään lokiinsa koodin (`docker logs finstats`), ja ohjattu asennus kysyy sitä ennen kuin se testaa osoitetta tai ottaa mitään käyttöön. Muuten portin ensimmäisenä tavoittanut olisi voinut ohjata FinStatsin omaan Jellyfiniinsä ja tulla sen ylläpitäjäksi tai käyttää testiä verkon tutkimiseen.
- **Kirjastojen käyttöoikeus seuraa Jellyfiniä**: kukin näkee vain ne kirjastot, jotka Jellyfin avaa hänelle, ja niissä vain ne nimikkeet, jotka Jellyfin näyttää hänelle hänen ikärajansa, tunnisteidensa ja luokittelemattomien nimikkeiden sääntöjensä mukaan, kirjastosivuilla, haussa, näyttelijöissä ja tekijöissä, kuvissa, Viimeksi lisätyissä, katselulistalla, Tulossa-näkymässä, kalenterisyötteessä ja uusien nimikkeiden ilmoituksissa. Jokainen kirjautunut saattoi selata jokaista kirjastoa.
- Linkki saattoi ohjata jonkun tervetulosivulta toiselle sivustolle heti kirjautumisen jälkeen. Nyt seurataan vain FinStatsin omia sivuja.
- Palvelimen loki ja oivallusten epäonnistuneet kirjautumiset antoivat kaikkien kirjautumisosoitteet ja toiminnan kenelle tahansa, jolla oli oikeus *Näe palvelin*. Jonkun toisen rivit vaativat oikeuden *Näe kaikkien toiminta* ja osoite oikeuden *Näe verkkotiedot*, kuten kaikkialla muualla.
- Henkilökohtainen ilmoituskohde saattoi tavoittaa lähiverkon nimen kautta, joka vastasi toisella hakukerralla eri tavalla. Yhteys otetaan nyt tarkistettuun osoitteeseen, useampi osoitealue, joka ei koskaan johda internetiin, lasketaan paikalliseksi, myös kotitalouden oma julkinen osoite hylätään, ja nimi, jota ei voi hakea, hylätään samoin sanoin kuin paikallinen.
- Sellaisen välityspalvelimen takana, joka lisää oman `X-Forwarded-For`-rivinsä, tai IPv6-verkosta jokainen kirjautumisyritys saattoi tulla uudesta osoitteesta, eikä raja koskaan pätenyt. IPv6-/48 saa nyt 50 yritystä viidessä minuutissa, täysi raja hylkää uuden osoitteen sen sijaan, että unohtaisi vanhan, koko palvelin pysähtyy 60 epäonnistumiseen minuutissa, ja epäonnistunut kirjautuminen kirjataan tarkastuslokiin kerran osoitetta ja aikaikkunaa kohden.
- `FINSTATS_TRUST_PROXY` nimeää välityspalvelimet (osoitteet tai osoitealueet), joiden `X-Forwarded-For`, `X-Forwarded-Host` ja `X-Forwarded-Proto` uskotaan, ja `1` tai `true` uskoo nyt vain loopbackissa tai yksityisessä verkossa olevaa vastapuolta. Päällä ollessaan mikä tahansa vastapuoli saattoi valita oman osoitteensa; pois päältä välityspalvelimen takana kymmenen väärää arvausta lukitsi kaikki ulos.
- Kun käyttäjät on luettu, kirjautumiseen nimellä, joka on tuntematon, poistettu käytöstä, poistettu tai jota ei ole päästetty FinStatsiin, vastataan vääränä salasanana kysymättä Jellyfiniltä, eikä aiemmin kuin Jellyfin olisi vastannut. Tällaisilla nimillä saattoi lukita Jellyfin-tilejä, kiertää Jellyfinin etäkäytön säännön tai piiloutua FinStatsin osoitteen taakse.
- Se, jonka Jellyfin rajaa kotiverkkoon (*Salli etäyhteydet* pois päältä), kirjautuu FinStatsiin vain sieltä, koska Jellyfin arvioi asetuksen FinStatsin oman osoitteen perusteella, ja käyttäjien luku, joka ei kerro, onko joku poistettu käytöstä, pitää hänet ulkona.
- Istuntoeväste merkitään Secure-määreellä aina, kun kirjautumissivu on https-yhteydellä, myös sellaisen välityspalvelimen takana, joka ei kerro sitä.
- Onnistunut kirjautuminen ei enää anna anteeksi sitä edeltäneitä arvauksia. https-yhteydellä istuntoeväste on `__Host-`-eväste, jota naapurinimellä oleva sivusto ei voi korvata ja jota ei hyväksytä millään muulla nimellä, ja uloskirjautuminen päättää jokaisen istunnon, jonka selain lähetti.
- Vain kalenteriavain avaa kalenterisyötteen sen osoitteesta; mikä tahansa muu avain kuuluu otsakkeeseen.
- API-avaimella ei voi enää myöntää oikeuksia, muuttaa sitä, kuka pääsee sisään tai mikä on julkista, palauttaa varmuuskopiota eikä lisätä, muuttaa tai testata palvelua tai ilmoituskohdetta: kukin niistä kestää avainta kauemmin, joten se vaatii kirjautuneen selaimen.
- Jokainen vastaus sisältää `Cross-Origin-Opener-Policy: same-origin` ja `Permissions-Policy`-otsakkeen, API-vastaukset sisältävät `Cross-Origin-Resource-Policy: same-origin` ja ovat `no-store`, ellei niissä toisin sanota, eikä CSP enää salli `blob:`-kuvia. Sivu, joka oli avannut FinStatsin, saattoi myöhemmin muuttaa välilehtensä omaksi kirjautumissivukseen.
- Kun FinStats tietää, millä skeemalla se tavoitetaan (luotetun välityspalvelimen `X-Forwarded-Proto` tai julkinen osoite), toisesta skeemasta tuleva kirjoitus hylätään sivustorajat ylittävänä.
- `/auth/me`, nimikkeen sivu ja Kirjaston kunto linkittävät Jellyfiniin sen julkisessa osoitteessa; osoite, josta FinStats itse lukee Jellyfiniä, näytetään vain ylläpitäjille.
- Kuva haetaan Jellyfinistä vain sellaiselle, mitä FinStatsilla on, julkaistun profiilin julisteet ja kortit mukaan lukien, joten muiden kirjastojen kuvia ei voi pyytää tunnisteella.
- Yksityisen trackerin avaimet peitetään nyt myös latausjonon virheistä, niin muotonsa kuin nimensä perusteella, ja sille, jolla on vain oikeus *Näe lataukset*, kerrotaan, mikä palvelu epäonnistui ja miten, mutta ei koskaan sen sisäistä osoitetta tai sitä, minne se ohjaa.
- Palvelin-sivu nimeää laitteen viimeisimmän käyttäjän vain oikeudella *Näe kaikkien toiminta*, ja Linkittämätön media vaatii sen oikeuden *Hallinnoi FinStatsia* lisäksi. Ilman sitä Tehtävät-osio laskee vain omat toistot ja suoratoistot, poistettu kirjasto luetellaan omien toistojen perusteella, eikä odottavaa Tautulli-kytkentätaulua voi peruuttaa.
- Asetukset näyttävät julkisen osoitteen palvelun pelkällä isäntänimellä ja Jellyfinin osoitteen sekä datakansion vain ylläpitäjille, ja kirjastosta kadonnut nimike kuvataan vain toistoista, jotka lukija saa nähdä.
- Tautulli-varmuuskopio, jonka taulut eivät olleet tavallisia tauluja, saattoi pitää kaikki muut kirjoitukset odottamassa, kunnes FinStats käynnistettiin uudelleen. Tällainen tiedosto hylätään, ja SQLite lukee varmuuskopion varovaisesti.
- Yli 4 GB:n lähetys hylätään, eikä zip-pakattua Tautulli-varmuuskopiota pureta ilmoittamaansa kokoa pidemmälle; kumpikin olisi voinut täyttää levyn, jolla FinStatsin oma tietokanta on.
- Vastausta Jellyfiniltä, Sonarrilta, Radarrilta, Seerriltä, julkisen osoitteen haulta tai GeoIP-lataukselta ei lueta suurinta sallittua määrää pidemmälle purkamisen jälkeen laskettuna.
- Kuka tahansa Jellyfin-käyttäjä saattoi pysäyttää toistojen tallentamisen kaikilta ilmoittamalla tuhansia lyhyitä toistoja yhdestä nimikkeestä. Sen selvittäminen, ketkä katsoivat yhdessä, tehdään nyt yhdellä läpikäynnillä, eikä se pidä lukkoa työn aikana.
- Peräkkäiset toistot tai yksi nimike, jota toistetaan vuorotellen laitteelta toiselle, eivät enää yhdistä toisilleen vieraita yhdeksi ryhmäksi, ja sille, joka saa nähdä vain itsensä, kerrotaan vain niistä ihmisistä, joiden rinnalla oma toisto kulki, ja vain siltä ajalta.
- Linkki saattoi yhä ohjata jonkun tervetulosivulta toiselle sivustolle kirjautumisen jälkeen polun kuten `/.//` kautta.
- FinStatsia hallinnoiva ei voi enää muuttaa säilytettävien varmuuskopioiden määrää tai sitä, kuinka usein niitä kirjoitetaan, yhdistämisikkunaa (nyt enintään tunti) eikä sitä, milloin käyttäjät, toimintaloki ja varmuuskopiot luetaan, eivätkä hänen varmuuskopionsa koskaan syrjäytä ylläpitäjän varmuuskopioita; kotiosoitteet ja matkustussäännöt vaativat samat oikeudet kuin hälytykset, ja historian tuonti vaatii lisäksi oikeuden *Näe kaikkien toiminta*.
- Tallennettu avain, tunnus tai salasana lähetetään vain palvelimelle, jolle se tallennettiin, ja ilmoituskohteen voi luoda tai muuttaa vain kirjautuneena, ei koskaan API-avaimella. Ylläpitäjä voi poistaa käytöstä tai poistaa jonkun toisen henkilökohtaisen kohteen, mutta ei ohjata sitä muualle, kytkeä sen varmennetarkistuksia pois eikä lukea sen ntfy-aihetta tai Pushover-käyttäjäavainta.
- Discordin ja Slackin webhookien on oltava `https`, koska niiden osoite on niiden tunnus.
- Epäonnistuneiden kirjautumisten tulva ei voi enää kätkeä oikeaa kirjautumista tai hukuttaa hälytyksiä: nimet, jotka eivät ole tilejä, tuottavat yhden ketään nimeämättömän ilmoituksen vartissa, hälytykset lähtevät ensin, ja Slack ja Discord näyttävät viestin sisällön tekstinä eivätkä mainitse ketään.
- Vuosikatsauksen sijoitus, Kapasiteetti, ilmoitus ja julkaistu profiili eivät kerro enempää kuin sivut: ei sijoitusta tai osuutta kaikkien ajasta sille, joka saa nähdä vain itsensä, kiireiset hetket ilman oikeutta *Näe kaikkien toiminta* vain päivinä, joina oli kolme tai useampia suoratoistoja, ei epäonnistunutta tehtävää tai yhteyttä sille, joka ei saa hallinnoida FinStatsia, ja julkinen profiili muuttuu kerran päivässä.
- Tautulli-varmuuskopio, jossa on generoitu sarake, hylätään, ja tuodun toiston kuva haetaan vasta, kun kirjastossa on sen nimike.
- Palautus ei koskaan avaa FinStatsia laajemmin kuin se on: asetukset palautetaan vain pyydettäessä, kirjautuminen, oikeudet ja julkiset profiilit ovat sitä, minkä sekä tämä asennus että varmuuskopio sallivat, ja julkiset osoitteet pysyvät tämän asennuksen omina. `finstats restore <file>` palauttaa vain historian, kuten sivukin, ja `--with-settings` myös asetukset.
- Pelkän historian palautus ei enää tuo takaisin kotiosoitteita, mykistettyjä hälytyksiä, tarkastuslokin merkintöjä, jotka näyttävät tämän asennuksen omilta, tai paikannettuja nimikkeitä, joita kirjastossa ei ole, eikä mikään palautus tuo toistoja, lokirivejä tai laitteita, jotka on päivätty ennen vuotta 1990 tai huomisen jälkeen. Palautettu toisto lasketaan tuoduksi riippumatta siitä, minkä lähteen se ilmoittaa, joten uudelleenkäynnistys ei koskaan jatka sitä.
- Datakansio kuuluu vain FinStatsin omalle käyttäjälle (0700, sen tiedostot 0600, `umask 077` imagessa). FinStats kieltäytyy toimimasta rootina toisen käyttäjän omistaman datakansion päällä ja kertoo, mitä `docker exec -u` -komentoa käyttää, ja entrypoint vaihtaa omistajan vain kansiolta ja FinStatsin omilta merkinnöiltä.
- Kohteen nimessä ei saa olla ohjausmerkkejä, eikä laitteen, henkilön tai nimikkeen nimi voi enää väärentää riviä lokiin.
- Linkin reittiparametri ei voi enää tavoittaa muuta API-polkua kuin oman sivunsa, ja FinUI:n elementtien rakentaja asettaa osoitteen vain, jos se on http(s) tai suhteellinen, ei koskaan `srcdoc`.
- Haku, julkaistut profiilit ja käsin käynnistetty työ eivät voi enää pitää FinStatsia kiireisenä: haku katkeaa 200 merkkiin, profiili säilytetään kielittäin, kortit odottavat lyhyessä jonossa, ja tehtävä, osoitehaku, sijaintitietokannan lataus ja testiviesti käynnistyvät vain rajallisen usein, pyydettiinpä niitä miten tahansa, testiviestit lähettäjäkohtaisesti laskettuina. Sijaintitietokannan hakeminen vaatii lisäksi samat oikeudet kuin Tietoturva-sivun lukeminen.
- Pyynnön sisällön on jatkettava saapumista ensimmäisten 15 sekunnin jälkeen, vastaus, jota kukaan ei lue, hylätään 30 sekunnin jälkeen, ja yksi osoite pitää enintään 64 yhteyttä, luotettua välityspalvelinta lukuun ottamatta. Muutama sata pyyntöä, jotka eivät koskaan lähettäneet sisältöään, saattoi estää FinStatsia vastaamasta kenellekään.
- Sähköpostipalvelinta ei lueta pidemmälle kuin mikään vastaus voi olla (1 Mt koko kirjeeltä), ja testiviesti päättyy yhtä pitkän ajan jälkeen kuin jonossa oleva viesti saa kestää. Palvelin, joka ei koskaan päättänyt riviä, saattoi saada FinStatsin pitämään yhä enemmän muistia, kunnes se pysäytettiin.
- Kirjaston lukua arvioidaan lukujen tallentamien nimikkeiden mukaan, ei tuonnin tuomien, ja Jellystat-nimike jo luetussa kirjastossa tulee poistettuna. Keksittyjen nimikkeiden tuonti saattoi saada jokaisen kirjaston luvun pysäyttämään FinStatsin, joka käynnistyksellä.
- Koko palvelimen 60 epäonnistuneen kirjautumisen minuuttiraja laskee vain Jellyfinille välitetyt yritykset, myös vastausta yhä odottavat. FinStatsin itse hylkäämät nimet antoivat kenen tahansa muutamalla osoitteella estää kaikkia kirjautumasta, ja monet yhtä aikaa lähetetyt arvaukset päätyivät kaikki Jellyfiniin.
- Jonka Jellyfin rajaa kotiverkkoon, pysyy siinä jokaisessa pyynnössä, myös istunto ja avaimet, ei vain kirjautuessa; koti on yksityinen verkko tai talouden haettu julkinen osoite, ei koskaan kodiksi kirjoitettu osoite, ja välityspalvelimen takana, jota `FINSTATS_TRUST_PROXY` ei nimeä, hänet pidetään ulkona. Kodista viety eväste tai avain avasi kaiken mistä tahansa, ja ylläpitäjä saattoi avata kodin valitsemalleen osoitteelle.
- API-avain ei voi enää kytkeä pois sitä, mikä huomaisi sen väärinkäytön, eikä varmuuskopioita, jotka kumoaisivat sen teot: montako varmuuskopiota säilytetään ja kuinka usein, yhdistämisikkunaa, kotiosoitteita ja matkasääntöjä, käyttäjien, toimintalokin ja varmuuskopioiden aikatauluja, kaikkien hälytysten mykistämistä tai kuittaamista, varmuuskopiota roskakoriin eikä ilmoituskohteen poistamista. Jokainen kesti avainta pidempään, kun se peruttiin.
- Sonarrin ja Radarrin julisteet sekä toivelistan seuraava jakso noudattavat myös kirjastojen käyttöoikeuksia: sarjalla, jonka Jellyfin piilottaa joltakulta, ei ole hänelle julistetta eikä päivää. Palveluiden tunnisteiden laskeminen paljasti, mitkä piilotetut sarjat olivat tulossa, ja lista kertoi piilotetun sarjan seuraavan jakson.
- Koosteen Yhdessä-luku nimeää vain ne, joiden toisto kulki omasi rinnalla, kuten Yhdessä-sivu. Ryhmä, johon päätyi jonkun toisen kautta, nimesi ihmisiä, joiden kanssa ei koskaan katsonut.
- Jellystat- tai Streamystats-tuonti lukee koko tiedostonsa ennen kuin se ottaa tietokannan kirjoituslukon, ja kirjoittaa sitten vain säilyttämänsä rivit. Lukon alla luettuna tiedosto, jossa oli muutama oikea rivi ja gigatavuja mitä tahansa muuta, piti kaikki muut kirjoitukset odottamassa, toistot ja kirjautumiset mukaan lukien.
- Palautus pitää yhden oikeuden henkilöä kohden, enintään monen talouden verran ihmisiä, ja tarkistaa tarkastusmerkinnän kaksoiskappaleen lukematta jokaista juuri kirjoittamaansa merkintää. Muokattu varmuuskopio saattoi palautettaessa pitää muistia ja tietokannan kirjoituslukkoa yhä pidempään.
- Asetuksineen tehty palautus kytkee julkisen osoitteen haun ja sijaintitietokannan latauksen päälle vain, jos tässäkin asennuksessa ne ovat päällä, eikä koskaan säilytä vähemmän varmuuskopioita tai kirjoita niitä harvemmin kuin tämä asennus. Varmuuskopio, joka säilytti yhden, saattoi saada seuraavan ajon poistamaan kaikki muut.

### Performance
- Julkaistu profiili lasketaan enintään kerran minuutissa, ei uudelleen jokaista sen kuvaa ja korttia varten.
- Henkilön historiaa käydään läpi tietoturvahälytysten varalta vain yksi kerrallaan, vaikka toistoja alkaisi kuinka monta.
- Sisään- tai uloskirjautuminen, avaimen peruminen tai yhden henkilön oikeuksien muuttaminen pyytää tarkistamaan tilanteen vain niitä reaaliaikaisia yhteyksiä, joita se koskee.
- Tautullin kytkentätaulu lukee historian kerran eikä kerran jokaista Plex-käyttäjää kohden.
- Välimuistista puuttuva kuva haetaan yhdellä pyynnöllä muiden odottaessa, eikä kuvaa, jota lähde ei pystynyt antamaan, pyydetä uudelleen 30 sekuntiin.
- Verkkokäyttöliittymän tiedostot pakataan gzipillä kerran ja säilytetään, eikä fontteja pakata uudelleen.

### Stability
- Jellystatin vanhemmassa muodossa oleva varmuuskopio luetaan rivi kerrallaan eikä kokonaan, ja mitä tahansa varmuuskopiota pidempi rivi hylätään.
- Kaksi samanaikaista Tautulli-lähetystä eivät enää kirjoita samaan tiedostoon.
- Sivunumero kaukana kirjaston nimikkeiden tai Kirjaston kunnon lopun jälkeen on tyhjä sivu, ei virhe.
- Tuonti tai palautus, jonka lähetys katkesi, ei enää hylkää kaikkia muita uudelleenkäynnistykseen asti, eikä se tai ennen alkamistaan hylätty tuonti jätä tiedostoa jälkeensä.
- Varmuuskopion yhtä riviä luetaan enintään 8 MB, ja Tautulli-varmuuskopioita otetaan vastaan yksi kerrallaan.
- Toisto säilyttää enintään 2 000 tapahtumaa, ja tuonti ohittaa toiston, joka ei ala ja pääty vuoden 1990 ja huomisen välillä.
- Yksi kaukana nollan alapuolella oleva jakson numero ei enää jumita jokaista kirjaston lukua: Kirjaston kunto (myös sen yhteenveto), Kapasiteetti, uudelleenlinkitys ja paikantaminen laskevat yhteen ja vertailevat lukuja ilman ylivuotoa, ja julkaisuversio tarkistaa ylivuodot, joten kaikki huomaamatta jäänyt pysäyttää yhden tehtävän eikä jää silmukkaan.
- Kauden, jakson tai loppunumeron arvo säilytetään vain, jos Jellyfin pystyisi pitämään sen (0:sta 2 147 483 647:ään), yhtä lailla Jellyfinistä, Jellystatista ja Tautullista.
- Tautulli-varmuuskopio, jonka tunnisteet toistuvat, hylätään, koska muutama MB rivejä yhden tunnisteen alla vei prosessilta muistin, ja varmuuskopio avataan muuttumattomana, eikä se jätä jälkeensä `-wal`- tai `-shm`-tiedostoa.
- Yhteyden on lähetettävä pyyntönsä 15 sekunnin kuluessa, ja enintään 512 palvellaan kerralla: muutama sata puoliavointa sokettia kaatoi FinStatsin. Reaaliaikaisen kanavan kehys on rajattu 4 kt:hen.
- Muutama kilotavu Sonarrilta, Radarrilta tai Seerriltä ei voi enää paisua gigatavuiksi muistia: lista ei säilytä enempää kuin FinStats siitä lukee.
- Ilmoituskohde, joka ei koskaan vastaa, viivästää vain itseään: kohteisiin lähetetään rinnakkain, ja kohde, joka aikakatkaistaan yhä uudelleen, jätetään rauhaan kymmeneksi minuutiksi.
- Jonon jäljellä oleva aika ja kokonaisnopeus, palautetun toiston luvut ja päivä, jona toisto poistuu roskakorista, eivät voi enää vuotaa yli.
- Palautus lukee koko tiedoston ennen kuin se ottaa kirjoituslukon, hylkää tiedoston, joka purkautuu paljon kokoaan suuremmaksi, ja kirjoittaa erissä, joten keräin jatkaa tallentamista sillä välin.
- Varmuuskopiot nimetään UTC-ajassa eikä niitä koskaan kirjoiteta olemassa olevan tiedoston päälle, ja vain FinStatsin omat päivitystä edeltävät kopiot karsitaan.
- Tulevaisuuteen päivätty palvelimen lokirivi ei enää pysäytä toimintalokin lukemista: se luetaan uusimmasta rivistä, joka ei ole tulevaisuudessa.
- Samaan aikaan julkaistuun linkkiin saapuvat pyynnöt laskevat sen vastauksen kerran, ja sen kortit piirretään omalla kaistallaan, joten linkin haltija ei voi enää saada omistajan Vuosikatsaus-kortteja vastaamaan 503.

### Fixed
- Tilin epäonnistuneet kirjautumiset ovat yksi varoitus aikaikkunaa kohden, ja se kasvaa hälytykseksi kahdessakymmenessä, ja nimi lasketaan tiliä vastaan vain, kun se on kirjoitettu kokonaan ("x alice" ei enää laske alicea vastaan).
- Discord- ja Slack-viestit suojataan ensin ja lyhennetään sen jälkeen: Markdownia täynnä oleva laitteen nimi sai Discordin hylkäämään jokaisen sen nimeävän hälytyksen.
- Julkinen IPv6-osoite hyväksytään henkilökohtaiselle kohteelle.
- Välityspalvelimen takana, joka kirjaa ihmiset sisään itse (nginx `auth_basic`), FinStatsin evästettä ei enää hylätä.
- Näyttönimi, jossa on `{{url}}`, näytetään julkaistun sivun esikatselussa sellaisena kuin se on kirjoitettu.
- Vuoden 9999 kooste näyttää oletusvuoden virheen sijaan.
- Reaaliaikainen sivu, jonka oikeus poistettiin ja palautettiin sen ollessa auki, pysyy taas ajan tasalla ilman uudelleenlatausta.

### Removed
- **Synnmørsk** ei ole enää FinStatsin kieliä: selain, joka oli valinnut sen, näyttää uusnorjaa (nynorsk), kirjakieltä, josta se tehtiin.

## [2.3.0] - 2026-10-07 to 2026-10-26

Kapasiteetti, reaaliaikainen kanava, yksinkertainen tila ja kielet

### Added
- **Kielet**: FinStats puhuu 23 kieltä. Brittienglannin, amerikanenglannin sekä norjan bokmålin ja nynorskin kirjoittaa ja tarkistaa ylläpitäjä; tanska, saksa, espanja, suomi, ranska, hindi, islanti, italia, japani, korea, hollanti, puola, portugali (Brasilia), venäjä, ruotsi, turkki, ukraina ja kiina (yksinkertaistettu ja perinteinen) ovat käännöksiä, joita kukaan täällä ei ole tarkistanut, ja ne sanovat englanniksi sen, mitä niistä vielä puuttuu. Kummankin asettelun jokainen sivu, julkinen profiili, Muutosloki ja kaikki, minkä palvelin vastaa sanoin (virheilmoitus, Vuosikatsauksen kortit, mitä Jellyfinin tehtävä tekee), ovat kohdassa Asetukset → Ulkoasu → Kieli valitulla kielellä, selainkohtaisesti; kunnes kieli valitaan, ratkaisee selaimen oma kieli. Luvut, päivämäärät, yksiköt ja luettelot kirjoitetaan niin kuin kyseinen kieli ne kirjoittaa. Skripti voi pyytää kieltä `Accept-Language`-otsakkeella.
- **Yksinkertainen tila**: toinen asettelu Plexistä tuleville, aseteltu klassisen Plexin tilastopalvelimen tapaan. Valikon sijaan palkki ylhäällä, ja pelkät tilastosivut: Etusivu, Kirjastot, Käyttäjät, Historia, Kaaviot ja Asetukset sekä rataskuvake, josta pääsee omaan profiiliin, Asetuksiin, Muutoslokiin, teemaan ja uloskirjautumiseen. Valitaan selainkohtaisesti kohdassa Asetukset → Ulkoasu → Asettelu. Se lataa vain oman puoliskonsa FinStatsista, joten se käynnistyy nopeammin; Vuosikatsaus, Yhdessä, Tulossa, Tietoturva ja palvelinsivut jäävät FinStats-asetteluun. Se käyttää FinStatsin omia värejä, vaaleana ja tummana, eikä siinä liiku mikään.
- **Yksinkertaisen tilan Etusivu**: mitä toistetaan, kortti kutakin toistoa kohden (miten se toistuu, kotoa vai muualta, milloin se päättyy), katselutilastot järjestettyinä kortteina toistomäärän tai keston mukaan viimeisten N päivän ajalta, mitä kukin kirjasto sisältää ja mitä on äskettäin saapunut.
- **Yksinkertaisen tilan Kaaviot**: toistot mediatyypin mukaan (päivä päivältä, viikonpäivän ja tunnin mukaan, kymmenen suosituinta sovellusta ja henkilöä), toistotavan mukaan (päivä päivältä, kunkin päivän eniten samanaikaisia toistoja, tiedoston ja lähetetty resoluutio, sovellukset ja henkilöt) sekä kuukausittain, toistomäärän tai keston mukaan, kaikille tai valituille henkilöille.
- **Yksinkertaisen tilan Kirjastot, Käyttäjät ja nimikkeet**: taulukot jokaisesta kirjastosta ja henkilöstä sekä siitä, mitä toistettiin viimeksi, millä ja mistä (osoite vain oikeudella *Näe verkkotiedot*), ja sivu jokaiselle kirjastolle, henkilölle ja nimikkeelle lukuineen ja omine historioineen.
- Skripteille: `/api/stats/split` (toistot sovelluksen, henkilön, resoluution, päivän tai kuukauden mukaan, jaoteltuna mediatyypin tai toistotavan mukaan); kärkilistat eri katsojien määrän mukaan (`sort=users`); toiminta rajattuna katsottuihin toistoihin (`watched=1`); oivallukset vain pyydetyistä osista (`parts=`); huippu jaettuna transkoodauksiin, suoratoistoihin ja suoriin toistoihin, myös päivittäin; mitä kukin kirjasto sisältää ja mitä siinä toistettiin viimeksi; viimeksi lisätyt lajeittain (`kind=`); lämpökartta mediatyypin mukaan; henkilön viimeisin toisto; toistetaanko toistoa kotoa.
- **Reaaliaikainen kanava**: yleiskatsaus, Tulossa-sivun lataukset ja tilapalkki pysyvät ajan tasalla yhden WebSocket-yhteyden kautta välilehteä kohden (`/api/live`), joten se, mitä toistetaan ja ladataan, muuttuu näytöllä sitä mukaa kuin se muuttuu eikä vasta seuraavalla tarkistuksella. Käänteisen välityspalvelimen takana, joka ei välitä WebSocket-päivityksiä, tai yhteydellä, joka katkesi sanaakaan sanomatta, sivut kysyvät muutaman sekunnin välein kuten ennenkin. `/api/now-playing`, `/api/downloads` ja `/api/summary` pysyvät ennallaan skriptejä varten.
- **Palvelin → Kapasiteetti**: pärjääkö tämä palvelin, sen perusteella mitä FinStats on nähnyt. Kiireisimmät hetket koskaan ja viimeisten 90 päivän ajalta: samanaikaiset toistot ja kuinka monen kuva transkoodattiin laitteistokiihdytyksellä, ilman sitä tai ilman tietoa (Tautullista tuotu historia on Plexin, ja se jätetään pois). Jokainen levy, jolla kirjastot ovat, ja kuinka nopeasti Jellyfinin kuuden kuukauden aikana lisäämä sisältö sen täyttää (äskettäin täytetty levy mitataan kuukauden päästä sen ensimmäisestä tiedostosta, koska ensimmäinen kuukausi on sen täyttämistä), ja, jos Jellyfin 10.11 tai uudempi ilmoittaa vapaan tilan, kuinka kauan se riittää sillä tahdilla. FinStats ei koskaan väitä, montako toistoa palvelin kestäisi.
- **Vie .ics** Tulossa-sivun Tulevat-osiossa: luettelo suodatettuna, ladattuna kalenteritiedostoksi kertatuontia varten. Se ei päivity itsestään; kohdan Asetukset → API-avaimet kalenteriavain antaa edelleen kalenterin, joka päivittyy.
- **Katsomisen arvoista**: Kapasiteetti huomauttaa, kun jokainen Jellyfinin 90 päivän aikana raportoima transkoodaus tehtiin ilman laitteistokiihdytystä, vaikka transkoodaus on yleistä, kun Jellyfinin kirjastoskannaus on käynnissä neljänneksen ajasta tai enemmän, ja kun levyllä on tilaa alle kuudeksi kuukaudeksi. Jokaisen mukana tulevat sen luvut, ja sen voi sivuuttaa samoin kuin Kirjaston kunnon havainnot.
- **Mistä kirjastosi koostuu** voidaan rajata yhteen tai useampaan kirjastoon, Kirjastot-sivulla ja kunkin kirjaston omalla sivulla (`library_id=a,b`).
- **Päivämäärä- ja aikamuodot** kohdassa Asetukset → Ulkoasu: päivämäärä muodossa 31.01.2026, 31/01/2026, 31-01-2026, 01/31/2026, 2026-01-31, 2026.01.31 tai 2026/01/31 ja kellonaika 24 tai 12 tunnin muodossa, etunollalla tai ilman, tai kumpikin kielen omalla tavalla. Tallennetaan tähän selaimeen.
- **Tervetuloa**: kun joku kirjautuu ensimmäistä kertaa, millä selaimella tahansa, ja käyttöönoton lopussa, FinStats kysyy kielen, päivämäärä- ja aikamuodot ja asettelun, ja kustakin näytetään kuva. Ohita pitää brittienglannin, 31/01/2026, 13:05 ja FinStatsin.

### Changed
- Sivun sisällä logo on pelkkä tunnus nimen vieressä, Inter-fontilla ladottuna, valitsipa esiasetus minkä fontin tahansa; yksinään (sivuston kuvake) se säilyttää pyöristetyn ruutunsa.
- Ohjelmistolla transkoodattu toisto tallennetaan sellaisena, kun Jellyfin niin kertoo; aiemmin se näytti samalta kuin toisto palvelimelta, joka ei kerro. Sivuilla molemmat näkyvät edelleen nimellä Ohjelmisto.
- Päivitys ajaa nyt kerran ja vanhimmasta versiosta alkaen sen, mitä versio vaatii vanhemman FinStatsin kirjoittamalle tietokannalle, migraatioidensa ohella. Version 2.3.0 vaihe lukee palvelimen tiedot ensimmäisellä käynnistyksellä, joten Kapasiteetti on heti valmis.

### Performance
- **Auki jätetty sivu ei kysy mitään**: yleiskatsaus teki 30 pyyntöä minuutissa pysyäkseen ajan tasalla ja Tulossa-sivun lataukset 18; reaaliaikaisen kanavan kanssa kumpikaan ei tee yhtään. Avoimien välilehtien määrästä riippumatta FinStats ei kysy Jellyfiniltä, Sonarrilta tai Radarrilta mitään enempää kuin ennen, ja piilotettu välilehti antaa yhä latausjonon hidastua.
- Henkilön sivu lukee vain kyseisen henkilön toistot: hänen kokonaismääränsä ja viimeisin toistonsa laskettiin palvelimen jokaisesta toistosta ja rajattiin vasta sitten häneen, noin neljä sekuntia miljoonalla toistolla.
- Kirjaston sivu lukee vain kyseisen kirjaston nimikkeet ja toistot, kun se ennen laski kaikkien kirjastojen ja piti yhden: kolmannes ajasta miljoonalla toistolla.
- Kun Jellyfinin kello oli FinStatsin kelloa edellä, *Jellyfinin kirjastoskannauksen jälkeen* luki koko kirjaston uudelleen joka minuutti, kunnes kellojen eron verran aikaa oli kulunut; kutakin skannausta seurataan nyt kerran.
- GeoIP-tiedosto, jota ei voitu avata, kopioitiin uudelleen, sata megatavua ja enemmän, ja jokainen osoite paikannettiin uudelleen joka vartti; sitä yritetään uudelleen vasta, kun se muuttuu.
- Kirjaston kunto avautuu heti, kun osoitin on levännyt sen linkin päällä: etukäteen haettu tieto säilytettiin paikassa, jota sivu ei koskaan katsonut.

### Stability
- Päivitys, joka epäonnistui migraation jo ajettua ja jonka Docker käynnisti heti uudelleen, otti jokaisella käynnistyksellä uuden kopion puoliksi päivitetystä tietokannasta, ja kolme käynnistystä karsi pois ainoan kopion, jonka edellinen versio pystyi avaamaan. Päivitystä edeltävä kopio säilytetään nyt.
- Jellyfinissä kirjaston luennan aikana poistettu nimike (Radarrin korvaama tiedosto) saattoi saada luennan ohittamaan toisen, yhä olemassa olevan nimikkeen ja merkitsemään sen poistetuksi, ja sen toistot saatettiin sitten siirtää sen toiselle kopiolle. Sitä, mitä luenta ei nähnyt, kysytään nyt ensin tunnisteella.
- Varmuuskopion palauttaminen ajalta ennen tiedoston uudelleennimeämistä tai päivittämistä palautti toiseen kertaan jokaisen uudelle tiedostolle sittemmin siirretyn toiston, pysyvästi kahdennettuna.
- Pyyntö Seerrin vastauksessa, jota FinStats ei pystynyt lukemaan (Seerrin päivityksessä uudelleennimetty kenttä), merkittiin poistetuksi päivittäisessä täydessä luennassa.
- Ohjelmavirheeseen törmännyt tehtävä jäi pysyvästi tilaan ”käynnissä”, eikä mikään ajastin käynnistänyt sitä uudelleen ennen kuin FinStats käynnistettiin uudelleen; nyt se päättyy epäonnistuneena.
- Vaurioitunut Tautulli-zip keskeytti latauksen poistamatta tiedostoa, joka sisältää jokaisen Plex-käyttäjän tunnukset; tiedosto jäi paikalleen kuuden tunnin siivoukseen asti.
- Tuonti tai palautus, joka keskeytettiin tiedoston saapuessa, jätti tiedoston datakansioon pysyvästi osoitteineen kaikkineen; se poistetaan seuraavalla käynnistyksellä.
- Kuva saatettiin tarjoilla puoliksi kirjoitettuna sen ollessa välimuistiin tallentumassa, ja selain säilytti sen sitten sellaisena.
- Peruutetut API-avaimet säilytettiin pysyvästi, vaikka niitä olisi luotu ja peruutettu kuinka monta tahansa.
- Jonkun katsottu- ja suosikkimerkintöjen luku, joka palasi tyhjänä tai lukukelvottomana, tyhjensi ne kaikki seuraavaan lukuun asti; se hylätään nyt, kuten kirjaston tyhjä luku.
- Jakso, jonka loppunumero ei voi olla minkään jakson (siihen kirjoitettu päivämäärä), saattoi pitää Kirjaston kunnon työssä loputtomiin kirjaston luvun jälkeen.

### Fixed
- Muutoslokin merkintä, jossa koodia oli lihavoinnin sisällä (2.0.2:n ja 2.0.3:n), näytti takahipsunsa koodin sijaan.
- Määriä näyttävän kaavion akseli saattoi olla 0, 3, 5, 8, 10 tai jopa 0, 0, 1, 1, 1: 2,5:n askelia pyöristettyinä. Määrät nousevat nyt kokonaisin askelin.
- Muun kuin hallinnoijan luoma kalenteriavain sai linkin selaimen omaan osoitteeseen FinStatsin julkisen osoitteen sijaan, joten se toimi vain sisäverkosta; sivu myös pyysi asetuksia, joita siltä evättiin.
- Henkilön viimeisin toisto saattoi olla muutaman sekunnin liian lyhyt laskettavaksi toistoksi missään muualla, jolloin se nimesi nimikkeen, jota hänen Historiansa ei näytä.
- Epäonnistunut palautus kohdasta Äskettäin poistetut ei sanonut mitään, ja sen painike jäi pyörimään; samoin epäonnistunut *Palauta* Kirjaston kunnossa sekä epäonnistunut *Avaa uudelleen* tai *Ratkaise kaikki* Tietoturvassa.
- Kalenterisyötteen nimi kalenterisovelluksessa on FinStats, ei finstats.
- Latausjono kertoi jokaiselle, joka sai nähdä sen, kuka oli pyytänyt kunkin latauksen; jonkun toisen pyyntö vaatii nyt oikeuden *Näe kaikkien toiminta*, kuten Tulossa-sivulla.
- Oikeuden *Vastaanota ilmoituksia* poistaminen ei lopettanut sitä, mitä henkilön kohteisiin lähetettiin, eikä hän voinut enää kytkeä niitä pois.
- Yhdistämisikkunan sisällä tauotettu ja jatkettu toisto kerrottiin kerran pysähtyneeksi, eikä sen jatkumisesta ja todellisesta päättymisestä kerrottu koskaan.
- Kappaleen toistot saatettiin siirtää toisen esittäjän samannimiselle kappaleelle (”Intro”), kun sen albumi poistui kirjastosta.
- Ilmoituskohde, joka oli kirjoitettu IPv6-osoitteen sisään upotettuna IPv4-osoitteena (NAT64, 6to4), hyväksyttiin julkiseksi, vaikka se tavoitti lähiverkon.
- Toiseen palvelimeen osoitettu Sonarr- tai Radarr-yhteys näytti vanhan palvelimen julisteita vielä viikon ajan.
- Tietoturvan hälytykset kertoivat, ettei mikään vaatinut huomiota, kun valittuna oli kaksi henkilöä tai enemmän, eikä sivulta, jonka viimeisen hälytyksen ratkaiseminen jätti tyhjäksi, päässyt takaisin.
- Jokainen API-avain, joka ei ollut vanhentunut, väitti vanhentuneensa ”juuri nyt”.
- Valintaikkuna jäi sivun päälle Takaisin-painalluksen jälkeen, eikä sivua voinut vierittää ennen kuin ikkuna suljettiin.
- Tarkastusloki näytti kahdeksan merkintälajia (palautettu toisto tai varmuuskopio, tyhjennetty roskakori, julkiset profiilit, muutettu ulkoasu) tunnisteillaan, eikä Tiedot-sarakkeessa ollut mitään.
- Nimikkeen lisääminen katselulistalle sen pikavalikosta epäonnistui sanaakaan sanomatta, kun palvelin kieltäytyi.
- *Katsottu yhdessä* yleiskatsauksessa käsitteli kerralla valitut useat henkilöt yhtenä henkilönä.
- Vuosikatsaus kirjoitti alle kymmenen tunnin ja päivän määrät selaimen desimaalipilkulla omien tuhaterotinpilkkujensa vieressä.
- Kapasiteetti sanoi levyn täyttyvän "viimeisten 0 kuukauden tahtia".
- Tietoturva-sivulla paikattomat osoitteet laskivat kaikkien toistot kaikilta ajoilta, rajattiinpa sivu mihin henkilöön ja jaksoon tahansa.
- Joku, jolla oli *Hallinnoi FinStatsia* mutta ei *Näe kaikkien toiminta*, pystyi siirtämään kenen tahansa toiston roskakoriin.
- Vuosikatsauksen tai julkisen profiilin kortti saattoi tulla selaimen välimuistista aiemmin valitulla kielellä.
- Live-kanavan lähettämät lataukset olivat brittienglanniksi, oli sivu millä kielellä tahansa.
- Kirjautuminen, joka muutti sitä, mitä joku saa nähdä, tavoitti hänen muut avoimet välilehtensä vasta seuraavassa puolen minuutin tarkistuksessa.
- Välilehti, joka luovutti live-yhteytensä uudemmalle, otti seuraavalla sivullaan yhden takaisin, joten saman henkilön välilehdet sulkivat jatkuvasti toistensa yhteyksiä.
- Taulukot lajittelivat koot 1 000–1 023 yksikköä tekstinä kielillä, jotka ryhmittelevät numerot välilyönnillä, ja turkinkieliset prosentit samoin.
- Nimike, jonka nimessä oli paikkamerkki, kuten "Big {device} Bunny", täyttyi palvelimen viesteissä.
- Taulukot lajittelivat kestot ja osuudet kokonaismäärästä (5/26) tekstinä kaikilla kielillä paitsi englanniksi ja Pohjoismaiden kielillä.
- Yhdellä kielellä yhä avoinna ollut välilehti sai palvelimen vastaukset toisella kielellä heti, kun toinen välilehti oli valinnut sen.
- Luvun vieressä oleva sana sai väärän muodon puolaksi, venäjäksi ja ukrainaksi ("5 wieczory", "2 потоков").
- Kapasiteetti laski kirjaston ensimmäisen täytön kasvuksi ja jakoi kunkin kirjaston lisäykset kuudella kuukaudella, mitattiinpa niitä kuinka pitkältä ajalta tahansa; toisen tilalle tullut tiedosto (Radarrin tai Sonarrin päivitys tai tiedosto, jolle Jellyfin antoi uuden tunnisteen) laskettiin kokonaan uudeksi, vaikka vain erotus oli uutta.
- SyncPlayllä yhdessä katsoneet eivät olleet ryhmä, jos toinen liittyi minuutteja toisen jälkeen; samaan aikaan samassa kohdassa päättyvät toistot kuuluvat nyt yhteen, ja historia ryhmitellään kerran uudelleen.
- Käyttäjä ilman oikeutta *Näe kaikkien toiminta* saattoi päätellä kirjaston kohdasta *Ei koskaan katsottu*, että joku muu oli toistanut nimikkeen; nyt se laskee vain hänen omat katselunsa.

## [2.2.0] - 2026-10-01 to 2026-10-07

Kirjaston kunto ja FinUI

### Added
- **Oma valikko tietokoneella**: Asetukset → Ulkoasu tarjoaa kahdeksan: sivupalkki entisellään, sivujen käyttötarkoituksen mukaan ryhmitelty, kuvakekisko, joka avautuu osoittimen alla sivupalkiksi, haulla alkava sivupalkki, telakka, komentopalkki, ylimmäksi kiinnitetyt sivut ja värilliset ruudut. Sivupalkit voivat olla oikealla. Telakka liukuu pois vieritettäessä alaspäin ja palaa, kun vierität ylöspäin tai osoitat ikkunan alareunaa; sen kanssa tilapalkki siirtyy ylös.
- **Oma valikko puhelimella**: välilehtipalkki ja Lisää (uusi oletus), puoliväliin avautuva paneeli, koko näyttö, peukalon ympärille levitetyt sivut tai osoitepalkki, joka laajenee valikoksi. Jokainen hakee sivuistaan ja välittää muut sanat haulle.
- Molemmat säilyvät selaimessa, jossa ne valittiin, joten puhelin ja tietokone pitävät kumpikin omansa; molemmat ovat FinUI-komponentteja, `desktop-nav` ja `mobile-nav`.
- **Pikavalikko**: napsauta hiiren oikealla nimikettä, henkilöä tai kirjastoa missä tahansa FinStatsissa (juliste yleiskatsauksessa, luettelon rivi, nimi taulukossa, Tulossa-kortin elokuva, jota kirjastossa ei vielä ole), tai pidä sormea sen päällä, tai paina valikkonäppäintä, niin painetusta kohdasta kasvaa valikko: avaa se, avaa se uudessa välilehdessä, lisää elokuva tai sarja katselulistallesi, avaa se Jellyfinissä tai TMDB:ssä ja TVDB:ssä, kopioi sen linkki. Shift ja hiiren oikea painike avaavat yhä selaimen oman valikon. FinUI:n `context-menu`.
- **Haku ilman valintaikkunaa**: haku kasvaa valikostasi (sivupalkki levenee hauksi, telakka nousee hauksi, komentopalkin nimi muuttuu hauksi, puhelimen valikko pitää sen sisällään) tai, kohdassa Asetukset → Ulkoasu niin valittuna, ottaa sivun paikan valikon pysyessä. Ctrl+Space avaa sen kuten ennenkin; kaiken päällä ollut ikkuna on poissa. FinUI:n `search`.
- **Kaikki kirjaston sisältö**: kirjaston Viimeksi lisätyt -kortti tarjoaa kohdan *Kaikki sisältö*, eli jokaisen sen sisältämän elokuvan, sarjan tai albumin julisteina, nimen, vuoden, lisäyspäivän tai koon mukaan, rajautuen kirjoittaessasi. Lisää tulee vierittäessä.
- **Palvelin → Kirjaston kunto**: FinStats vertaa jokaista tiedostoa naapureihinsa ja luettelee poikkeamat, kunkin taustalla olevine lukuineen. Se vain raportoi; Jellyfinissä ei muuteta, poisteta tai skannata uudelleen mitään.
- **Aukot**: kauden kahden tiedoston välistä puuttuvat jaksot (”Kausi 2: jaksot 4 ja 5 puuttuvat jaksojen 3 ja 6 välistä”). Kaksi jaksoa sisältävä tiedosto lasketaan molemmiksi; erikoisjaksot ja kauden loppu jätetään rauhaan.
- **Poikkeavat kaudet**, jotka eroavat sarjansa muista kausista resoluutioltaan, HDR:ltään tai koodekiltaan, ja **poikkeavat jaksot**: yksi kaudestaan poikkeava tiedosto, kuten SD-jakso 1080p-kaudessa. Resoluutio määräytyy leveyden tai korkeuden mukaan, joten rajattu 1920×800-elokuva on 1080p.
- **Kopiot ja versiot**: sama elokuva tai jakso kahdesti, samojen palveluntarjoajien tunnisteiden mukaan, joita katselulista ja Tulossa jo käyttävät. Kaksi kopiota samassa resoluutiossa näyttää kahdesti varatun tilan; 4K-kopio 1080p-kopion rinnalla luetellaan kahtena versiona eikä sitä koskaan lasketa hukaksi.
- **Ohuet tiedostot**: bittinopeus selvästi alle sen, mitä resoluutio ja koodekki yleensä vaativat (”4K HEVC 1,1 Mbps:llä”); kauden ohuet jaksot ovat yksi havainto.
- **Katkeavat jälkiäänitykset**: ääniraidan kieli, joka joillakin kausilla on jokaisessa jaksossa ja toisilla ei yhdessäkään.
- **Tunnistamattomat**: elokuvat ja sarjat, jotka Jellyfin täsmäsi ilman TMDB-, TVDB- tai IMDb-tunnistetta.
- **Sivuuta** havainto, joka on tarkoituksella kunnossa, ja lisää huomautus. Se palaa itsestään, kun sen tiedostot muuttuvat. Sivuuttaa voivat ne, jotka saavat hallinnoida FinStatsia ja nähdä palvelimen; sivuutukset näkyvät tarkastuslokissa ja kuuluvat varmuuskopioon.
- Kirjaston sivu kertoo, montako tarkistettavaa asiaa siellä on, ja vie niiden luo.
- **FinUI**, komponentit, joista FinStats on rakennettu, on oma projektinsa (`github.com/finstats/finui`): jokainen painike, kortti ja säädin kaikissa tiloissa ja molemmissa teemoissa osoitteessa `finui.finstats.no`, sekä **FinUI create**, jolla voit muuttaa sen värejä, pyöristystä, tiheyttä ja muuta ja nähdä tuloksen heti sekä viedä sen kotiin yhdellä komennolla. **Asetukset → Järjestelmä** vie sinne.
- **Asetukset → Ulkoasu**: jokainen valitsee, miltä FinStats hänelle näyttää, FinUI-esiasetuksen koodilla: pohjaväri, korostusväri, kaavioiden värit, kontrasti, pyöristys, tiheys, reunat, kortit, painikkeet, kentät, taulukot, korostus, liike, kuvakkeet, valikko, kaiken takana oleva sivu, kohdistuskehys, otsikoiden asettelu sekä tekstin, otsikoiden ja lukujen fontit. Valintasi on vain sinun; se näyttää, mitä koodi tarkoittaa, ja vie FinUI createen sen muuttamiseksi.
- **Tyylit**: yksitoista kokonaista ulkoasua, joista voi aloittaa yhdellä valinnalla FinUI createssa, jonka koodin Asetukset → Ulkoasu ottaa. Ne ovat Washi (FinStats sellaisena kuin se toimitetaan), Studio, Noir, Matinee, Console, Pastille, Ledger, Gazette, Arcade, Canopy ja Harbour. FinStats itse toimitetaan yhdellä ulkoasulla eikä tarjoa tyyliluetteloa.
- **Liikkuvat kuvakkeet**: jokainen linkin, painikkeen tai välilehden kuvake tekee oman liikkeensä kerran, kun sitä osoitetaan (päivitys pyörähtää, latauksen nuoli putoaa lokeroonsa, roskakori nostaa kantensa), ja painettu päivitys pyörii, kunnes vastaus on tullut. Ne pysyvät paikallaan, kun liikettä on vähennetty, ja asetuksella Liike: Pois.
- **Liikettä FinMotionilla**: FinUI:n komponentit liikkuvat nyt: kytkimen nuppi sinkoaa toiselle puolelle, välilehden alleviivaus siirtyy seuraavan välilehden alle, hylätty kenttä pudistaa päätään, kopiointi kuitataan yhdellä vedolla, ilmoitukset kasautuvat kuin korttikäsi ja levittäytyvät viuhkaksi osoitettaessa, puhelimen paneeli joustaa kuin kuminauha vedettäessä, lajitellun taulukon rivit siirtyvät paikoilleen, muuttuva luku pyörähtää kuin matkamittari ja kaaviot piirtävät itsensä, kun ne nähdään ensimmäisen kerran. Kaikki noudattaa kohdan Asetukset → Ulkoasu Liike-valintaa ja pysyy paikallaan, kun liikettä on vähennetty. FinMotion on oma projektinsa (`github.com/finstats/finmotion`), joka rakentuu FinUI:n päälle.
- **Lisää kuvakkeita FinUI:ssa**: nyt 239, aiemmin 71, jokaisella oma liikkeensä (äänenvoimakkuus ylös ja alas, kuulokemikrofoni ja mykistetty sellainen, kirjoittava kynä, soiva kello, väriseva puhelin), ja jokaisen voi ladata FinUI:n galleriasta SVG-muodossa, paikallaan pysyvänä tai animoituna.
- **Fontit**: kolmekymmentäneljä avoimesti lisensoitua fonttiperhettä valittavana (Geist, Roboto, Manrope, IBM Plex Sans, Montserrat, Lora, Playfair Display, Fira Code ja muita), mukana Interin tapaan, ei koskaan haettuna muualta, ja ladattuna vain valittaessa.
- **Lisää pohjia ja korostusvärejä**: Nord, Solarized, Gruvbox, Catppuccin, Rosé Pine, Tokyo Night, Dracula, Everforest ja Ocean; kuusitoista uutta korostusväriä, joista kymmenen on päivällä yhtä ja yöllä toista väriä. Jokaisella pohjalla on nyt päivällä oma paperinsa: harmaat näyttivät vaaleassa teemassa samanlaisilta.
- **Poiston voi perua 30 päivän ajan**: poistettu toisto tai varmuuskopio odottaa roskakorissa, ja **Kumoa** sitä seuraavassa ilmoituksessa tuo sen takaisin. Toiminta ja Asetukset → Varmuuskopiot luettelevat kumpikin äskettäin poistetut ja sen, milloin ne poistuvat lopullisesti, aina kun niitä on. Roskakorissa oleva toisto ei ole mukana missään kokonaismäärässä, kaaviossa, vuosikatsauksessa tai profiilissa, eikä tuonti tai palautus tuo sitä takaisin uutena; se, joka katsoi sen jonkun kanssa, lasketaan yksin katsoneeksi, kunnes toisto palaa.

### Changed
- Dark Reader ja sen kaltaiset laajennukset jättävät FinStatsin sivut rauhaan (`darkreader-lock`): FinStats piirtää molemmat teemat itse, ja uudelleen väritetty sivu taisteli niitä vastaan.
- Puhelimella valikko ei enää liu'u sisään vasemmalta: välilehtipalkki on alareunassa, ellei jotain muuta ole valittu.
- FinStats lukee kirjastosi kerran tämän päivityksen jälkeen oppiakseen, mihin useita jaksoja sisältävät tiedostot päättyvät.
- Nimikkeen sivulla **Lisää katselulistalle** on yhtä korkea kuin sen vieressä oleva **Avaa Jellyfinissä**; se oli kokoa pienempi.
- Aikaväli- ja henkilösuodattimet pysyvät näytön yläreunassa sivua vieritettäessä, joten aikavälin 7 pv voi vaihtaa Kaikki-valintaan pitkän sivun alareunastakin; samoin Tulossa-sivun pyyntö- ja tulevat-suodattimet.
- Ylös vierittävä painike pysyy poissa sen valikon tieltä, joka pitää ikkunan alareunaa: alhaalla kulmassa telakan vieressä, juuri puhelimen välilehtipalkin, peukalopainikkeen tai osoitepalkin yläpuolella ja muuten tilapalkin yläpuolella. Se on FinUI:n `to-top`.
- Toiston tai varmuuskopion poistaminen siirtää sen roskakoriin sen sijaan, että se poistettaisiin heti (`PUT` ja `{"deleted": true}`; `DELETE` on poistettu API:sta). Säilytettävän määrän ylittävät varmuuskopiot poistetaan edelleen lopullisesti.
- **Muutosloki** on yksi arkki korttipinon sijaan: jokainen merkintä on oma rivinsä, ja sen laji (Lisätty, Muutettu, Korjattu ja muut) sekä lajin kuvake ovat sen edellä omassa sarakkeessaan, kunkin sarjan otsikon alla. Versioluettelo pysyy oikealla.

### Performance
- FinStatsin avaaminen lataa vain sen, mikä muuttui: selaimella jo oleva tyylitiedosto, skripti tai fontti vahvistetaan muutamalla tavulla sen sijaan, että se lähetettäisiin uudelleen. Jokainen käynti haki ennen kaikki FinStatsin tyylitiedostot ja moduulit kokonaan, vaikka jokaisella oli tunniste niiden erottamiseksi.

### Stability
- Varmuuskopion palauttaminen lukee asetukset pelkästään asetusriviltä: uudemman FinStatsin tiedostosta, joka säilyttää siellä enemmän, asetukset pudotettiin sanaakaan sanomatta.
- Avoin välilehti ei enää kasva jokaisella vieraillulla sivulla: jokainen jätetty sivu jäi muistiin näytöllä olevan taakse (noin 2 600 elementtiä käyntiä kohden) FinMotionin liikutettavien luettelon ja suodatinpalkin vierityskuuntelijan pitämänä, kunnes välilehti suljettiin.
- Väärin kirjoitettu komento (`finstats bakcup`), tai `restore` ilman tiedostoaan, ei enää luo tai päivitä tietokantaa ennen kuin kertoo, ettei tiedä mitä tehdä.

### Fixed
- Ruudun pieni kaavio ei enää valu ruudun yli, kun sen vieressä oleva luku on leveä (”56 pv 23 t”) tai kun neljä ruutua jakaa kapeamman rivin: kaavio antaa tilaa ja luku säilyttää leveytensä.
- Vaaleassa teemassa ensisijainen painike muuttui osoittimen alla violetiksi; nyt se säilyttää sinetin värin.
- Asetuksissa kytkin on samalla tasolla kuin asetuksen nimi ja ohje; se roikkui rivin yläreunassa.
- Järjestetyssä luettelossa, jossa nimen alla ei ole mitään, kuten yleiskatsauksen aktiivisimmissa käyttäjissä, nimi oli rivinsä yläreunassa keskitetyn kuvan vieressä; nyt se on keskitetty.
- Tilastoruuturivi puhelimen ja työpöydän väliin jäävällä leveydellä (neljä ruutua kolme rinnakkain) jätti viimeisen yksin omalle rivilleen; neljä ruutua menee nyt neljä, kaksi tai yksi rinnakkain, ja kolme ruutua kolme tai yksi.
- Palvelimen luenta laski laitteen, jolla kukaan ei ollut kirjautunut ja jota se ei säilytä: tehtävä kertoi kolme laitetta, kun Palvelin-sivu luetteli kaksi.
- Yksi sekunti, yksi minuutti, yksi tunti: hälytykset sanoivat englanniksi ”1 seconds apart” ja ”in 1 minutes”, ja minuutin välein ajettava Jellyfin-tehtävä ajettiin ”every 1 minutes”.
- `finstats --help` nimeää jokaisen muuttujan, jonka FinStats lukee; `FINSTATS_PUBLIC_IP_URL`, `FINSTATS_ALLOW_LIBRARY_SHRINK` ja `FINSTATS_SKIP_PREUPDATE_BACKUP` puuttuivat.
- Käyttöönottotoiminto sanoi osoitteesta, joka ei ollut kelvollinen osoite, ettei siihen ”saatu yhteyttä”, lauseessa, joka valui seuraavaan; nyt se kertoo, ettei se ole kelvollinen osoite.
- Kohdassa Asetukset → Varmuuskopiot varmuuskopion kirjoittamisen aikana esitetty kysymys katosi, kun varmuuskopio valmistui.
- Kun valikkona oli avautuva kisko, haun sulkeminen näytti avoimen haun ja suljetun kiskon yhtä aikaa ja jätti sitten kiskon auki, kunnes osoitin tuli ja meni. Esc palauttaa nyt näppäimistön kohdistuksen sinne, missä se oli haun avautuessa, ja päästää napsautuksen kohdistuksen menemään.
- Puhelimella haun sulkeminen piti sivun himmennettynä, kunnes haku oli poissa, ja kirkastui sitten yhdellä kertaa; himmennys häivytetään nyt sen mukana.
- Tautulli-tuonnin ohjeet sanoivat, että varmuuskopio latautuu **Backup Database** -painalluksen jälkeen. Tautulli tallentaa sen omalle koneelleen: ohjeet kertovat nyt missä (sen `backups`-kansio, Dockerilla `/config`), mikä tiedosto otetaan ja että se kopioidaan tänne.
- Haku, joka suljettiin ennen kuin se oli avautunut kokonaan, jätti avautuvan kiskon ilman animaatiotaan: siitä lähtien se avautui ja sulkeutui hetkessä.
- Avautuva kisko jäi auki sivun valinnan jälkeen, kunnes napsautettiin muualle, ja joissakin selaimissa tai lähennettynä se piirsi vierityspalkin kuvakkeidensa alle. Nyt se avautuu vain osoittimelle ja näppäimistölle eikä piirrä vierityspalkkia suljettuna.
- Tietoturva-sivulla osoite, jonka tietokanta sijoittaa maahan mutta ei kartalle, lasketaan kyseiseen maahan eikä osoitteisiin, joiden paikkaa ei tiedetä.

## [2.1.2] - 2026-10-01

### Added
- **Tuonti Tautullista**: tuo Plex-historiasi mukaan. Lataa Tautullin tietokannan varmuuskopio (`.db`, tai sen sisältävä `.zip`) kohdassa **Asetukset → Tuonti**; sen toistoilla on oma kohtansa Toiminta-sivun *Tallentaja*-suodattimessa.
- **Kytkentätaulu, jolla kerrot, kuka on kuka**: Plex-käyttäjät toisella puolella, Jellyfin-käyttäjät toisella. Vedä johto jokaisesta Plex-käyttäjästä siihen, joka hän nyt on, tai napsauta ensin yhtä ja sitten toista, tai käytä näppäimistöä. Kytkemättä jätettyjä ei tuoda, ja kaksi Plex-tiliä voi mennä samalle henkilölle.
- Elokuvat ja jaksot täsmätään kirjastoosi nimen perusteella (elokuva nimensä ja vuotensa, jakso sarjansa ja numeronsa mukaan), ja palvelimeltasi vielä puuttuva säilytetään ja liitetään, kun se saapuu. Jatkettu katselukerta on yksi toisto; musiikkia ei tuoda.
- Varmuuskopio sisältää Plexin käyttöoikeustunnukset: FinStats ei koskaan lue niitä, ja se poistaa tiedoston, kun tuot tai aloitat alusta.
- **Linkittämätön media** kohdassa **Asetukset → Tiedot**: jokainen Tautullin, Jellystatin tai Streamystatsin nimike, joka ei vastaa mitään kirjastossasi, sekä mistä se tuli ja mistä se todennäköisimmin jo löytyy. **Paikanna** linkittää sen parhaista arvauksista tai haulla (elokuvan voi paikantaa sarjan erikoisjaksoksi), ja FinStats muistaa valinnan, joten uudelleen tuotu sama historia linkittyy itsestään. Valinnat kuuluvat varmuuskopioon.

### Performance
- **Vuosikatsaus avautuu kolmesta viiteen kertaa nopeammin**: vuoden katsotuimmat näyttelijät ja ohjaajat laskettiin käymällä vuosi läpi uudelleen jokaista siinä esiintyvää henkilöä kohden. Todellisella historialla mitattuna vuosi nopeutui 0,29 sekunnista 0,09 sekuntiin ja viimeiset 12 kuukautta 0,53 sekunnista 0,12 sekuntiin.

### Fixed
- **Tuotu historia löytää nimikkeet, joilla on toinen nimi**: toisto täsmää myös nimikkeen alkukieliseen nimeen, joten sarja, jonka Plex tai Tautulli tunsi nimellä 오징어 게임, liittyy Squid Gameen ja *Im Westen nichts Neues* elokuvaan *All Quiet on the Western Front*. Kirjastosi alkuperäiset nimet luetaan sen seuraavassa skannauksessa.
- Nimiä verrataan samoin kuin haku niitä vertaa (kirjainkoko, tarkkeet, välimerkit, nimeen kirjoitettu vuosi, alun ”The”), ja julkaisuvuosi saa poiketa yhdellä, joten ”Star Wars: Episode I - The Phantom Menace” löytää nimen ”… – The Phantom Menace” ja vuoden 2014 Kingsman vuoden 2015 version. Kokonaan toisella nimellä tunnettua nimikettä ei edelleenkään koskaan arvata.
- Epäonnistunut Streamystats-tuonti tai -palautus ilmoitettiin ilmoituksissa sisäisellä nimellään.
- Vuosikatsauksen *Jaa vuosi kortteina* -painikkeen ja sen vieressä olevan katsojasijoituksen välissä ei ollut tilaa.

## [2.1.1] - 2026-10-01

### Changed
- **Katselulistaa tarjotaan kaikkialla, missä nimike on**: yleiskatsauksen Tulossa-korteissa, kuten kohdassa Tulossa → Tulevat, sekä jakson tai kauden sivulla, josta sen sarja lisätään listalle.
- Yleiskatsauksen julisteessa katselulistapainike näkyy, kun osoitat korttia tai saavutat sen näppäimistöllä, sen sijaan että se peittäisi jokaisen julisteen kulman; kosketusnäytöllä se on aina näkyvissä.

### Stability
- Sonarr-, Radarr- tai Seerr-yhteyden tallentaminen, kun joku piti **Asetukset → Lähtevät yhteydet** auki, saattoi jättää molemmat odottamaan toisiaan pysyvästi.
- **Asetukset, joita ei voi lukea, pysäyttävät FinStatsin käynnistyksessä** ja nimeävät virheellisen asetuksen sen sijaan, että jokainen asetus palautettaisiin hiljaa oletukseensa.
- **Kaksi samanaikaista asetusmuutosta** (esimerkiksi asetussivu ja tehtävän ajoitus) eivät enää kumoa toisiaan.
- Tauko, siirtymä tai raidan vaihto, joka nähtiin tietokannan ollessa varattu, kirjoitetaan seuraavalla kierroksella sen sijaan, että se hukattaisiin.
- Kirjaston luenta ei enää epäonnistu, kun katselulistalla on useita merkintöjä, jotka osoittautuvat yhdeksi nimikkeeksi.

### Fixed
- **Korvattu tiedosto ei ole ”nyt palvelimella”**: katselulistalla olevan elokuvan tiedoston päivittäminen ei enää kerro listan omistajalle, että elokuva on juuri saapunut.
- Yksi epäonnistunut Jellyfinin lisäosien, ajastettujen tehtävien tai tallennustilan luenta ei enää tyhjennä kyseistä luetteloa Palvelin-sivulla seuraavaan luentaan asti.
- **Streamystats-toisto pysähtyy jokaisella sivulla siihen, mihin se pysähtyi**: nimikkeen katsojakäyrä ja jakson ”katsottu loppuun” käyttivät sen sijaan sen pituutta ja laskivat sen arvioiduksi.

## [2.1.0] - 2026-09-29 to 2026-09-30

Katselulista

### Added
- **Oma katselulista**: lisää elokuva tai sarja sille sen sivulta, Tulevista, Viimeksi lisätyistä tai haulla (Ctrl+Enter), ja löydä se omasta profiilistasi kohdasta **Katselulista**.
- **Jokainen merkintä kertoo, missä se on nyt**: palvelimella, ”5/26 jaksoa”, pyydetty, ”Tulossa perjantaina”, ei palvelimella, poistunut kirjastosta tai katsottu. Tila lasketaan aina, kun katsot, samalla ”nähdyn” tulkinnalla kuin sarjojesi edistyminen. Katsotut nimikkeet pysyvät omassa ryhmässään, kunnes poistat ne.
- **Nimike, jota palvelimellasi ei vielä ole**, voidaan lisätä listalle suoraan siitä, mitä Sonarr tai Radarr odottaa. Se liitetään nimikkeeseen itsestään, kun se saapuu, ja uudelleennimetty tiedosto vie merkintänsä mukanaan.
- **”Katselulistallasi: nyt saatavilla”**: ilmoitus omiin kohteisiisi, kun jotain odottamaasi saapuu. Pois päältä, kunnes valitset sen.
- **Kukaan muu ei näe katselulistaasi**, ei edes ylläpitäjä, ja sen ilmoitus menee vain omiin kohteisiisi. Jellyfiniin, Seerriin, Sonarriin tai Radarriin ei kirjoiteta mitään.
- Katselulistat kuuluvat varmuuskopioon, ja palautus yhdistää ne kahdentamatta mitään.

### Changed
- `/api/upcoming`-merkinnät sisältävät nimikkeensä TVDB- ja TMDB-tunnisteet, ja `/api/auth/me` kertoo, pitääkö palvelin katselulistoja.

## [2.0.4] - 2026-09-29

### Added
- **Tehtävien ajoitus Jellyfinin tapaan**: jokaisella FinStatsin itsestään tekemällä työllä on laukaisimet (päivittäin tiettyyn aikaan, viikoittain tiettynä päivänä, välein, käynnistyksessä tai Jellyfinin kirjastoskannauksen jälkeen), kullakin valinnainen aikaraja. Napsauta tehtävää nähdäksesi ja muuttaaksesi sen ajoitusta; *Lisää laukaisin* avaa saman valintaikkunan kuin Jellyfinissä.
- **Tehtävät on oma osionsa** Asetuksissa: kunkin työn viimeisin ajo kerrotaan sanoin (”Ajettu viimeksi 38 minuuttia sitten, kesti 6 minuuttia”), ja jokaisella on Aja-painike; ennen se oli Järjestelmä-osion yläosassa.
- **Metatietojen muutokset**: FinStats lukee joka tunti, mitä Jellyfin on tallentanut edellisen tarkistuksen jälkeen (nimet, kuvaukset, lajityypit, arviot, näyttelijät ja tekijät, tiedostojen tiedot ja kuvat), joten Jellyfinissä tehty muokkaus näkyy odottamatta kirjastoskannausta.
- Varmuuskopion ja paikannustietokannan voi ajaa myös Tehtävistä; Esc palaa tehtävän ajoituksesta luetteloon.

### Changed
- **Teema on kolmiasentoinen kytkin**, Vaalea · Laite · Tumma, siinä missä ennen oli niiden läpi kiertävä painike: napsauta sen kolmannesta, vedä nuppia tai käytä nuolinäppäimiä. Nupissa on valinnan kuvake, joka liukuu sisään, kun valinta muuttuu.
- **Uloskirjautuminen on kohdassa Asetukset → Tili**, joka kertoo, kuka tässä on kirjautuneena; sivupalkkiin jää teemakytkin.
- **Vanhat ajoitusasetukset ovat nyt laukaisimia**: *Seuraa Jellyfinin kirjastoskannausta* varaväleineen, *Varmuuskopioi N päivän välein* ja GeoIP:n *Pidä tietokanta ajan tasalla* -kytkin muuttuivat tehtäviensä vastaaviksi laukaisimiksi, sinun arvoillasi. Keräys, Varmuuskopiot ja Tietoturva linkittävät nyt ajoitukseen.
- Käynnistys ei enää aja jokaista työtä uudelleen: se, milloin kukin viimeksi valmistui, säilytetään.
- Jellyfin-tehtävä, jolla ei vielä ole arviota, näyttää ”noin … jäljellä” ja pisteet kiertävät siinä, mihin aika tulee, sen sijaan että se näyttäisi ”ETA…”.

### Performance
- **Tietoturvan kartta ei enää nyi**: vetäminen ja zoomaus siirtävät jo piirrettyä karttaa ja piirtävät sen kerran, kun ele päättyy, eikä nyt toistossa -pulssi enää pakota koko karttaa piirtymään uudelleen 120 kertaa sekunnissa, vaikka kukaan ei koske siihen. Veto nousi hitaalla tietokoneella 34 ruudusta 60 ruutuun sekunnissa.

### Fixed
- **Jellyfinissä korvattu juliste tai muotokuva näkyy FinStatsissa**: kuvat pidettiin välimuistissa viikon palvelimella ja toisen viikon selaimessa, vaikka Jellyfinissä olisi sittemmin ollut uusi. Nyt ne tallennetaan välimuistiin Jellyfinin kuvatunnisteen mukaan, joten uusi kuva on uusi tiedosto, ja selain tarkistaa joka kerta, että sen kopio on ajan tasalla. Muotokuvan korvaaminen tallentaa uudelleen vain henkilön, joten Metatietojen muutokset lukee myös henkilöt.
- **Palvelin → Tehtävät näyttää arvion heti sivun avautuessa**: jäljellä oleva aika mitattiin vain lukemista, jotka sivu itse teki ollessaan auki, joten jokainen käynti alkoi tilasta ”ETA…”, kunnes sivu oli nähnyt prosentin liikkuvan. FinStats mittaa nyt käynnissä olevan Jellyfin-tehtävän ajan itse, 10 sekunnin välein sen ollessa käynnissä, ja sivulle tarjotaan valmis arvio. Tehtävä, jonka Jellyfin ilmoittaa koko ajan olevan 0 %:ssa (kirjastoskannauksen ensimmäinen vaihe), pitää pisteensä, kunnes se liikkuu: mitään ei arvata.
- **Kartan työkaluvihjeet ilmestyvät pisteen kohdalle**: Firefoxissa nyt toistossa -pisteen työkaluvihje saattoi osua kauas, ja päällekkäisissä pisteissä (joku toistaa kotona) näkyvä vihje riippui yhdestä pikselistä. Nyt yksi vihje nimeää kaiken osoittimen alla olevan.
- **Tilapalkilla on yksi osoitusmuoto**: jokainen linkki (toistot, Repo, versio) syttyy samanlaiseksi pieneksi laatikoksi, ja kohteiden väliset pisteet ovat keskellä välejään. Version osoittaminen sytytti ennen myös sitä edeltävän pisteen ja välin.
- **Tiedostopolku vie korttinsa koko leveyden**: nimikkeen Tiedosto-kortissa ja Jellyfinin kansioissa Palvelin-sivulla polku oli puristettu yhteen kapeaan soluun ja valui yhdeksälle riville tyhjän tilan vieressä.
- Häivytettävä valintaikkuna ei enää niele napsautusta sen alla olevalla sivulla.

## [2.0.3] - 2026-09-28

### Security
- Palvelin-sivu antoi jokaisen laitteen tunnisteen kenelle tahansa, jolla oli *Näe palvelin*; tunnisteet vaativat myös oikeuden *Näe verkkotiedot*, kuten kaikkialla muualla.
- Kaksi oikeuskytkintä, joita käännettiin ennen kuin ensimmäinen ehti tallentua, lähettivät saman lähtöluettelon kahdesti, ja toinen kumosi ensimmäisen: poistettu oikeus jäi myönnetyksi ja pois kytketty julkisen profiilin osio julkaistuksi, vaikka molemmat kytkimet näyttivät valitun tilan.

### Stability
- **Vain yksi tuonti tai palautus kerrallaan**: kaksi yhtä aikaa lähetettyä latausta läpäisivät molemmat tarkistuksen, eikä palautus koskaan katsonut tuonteja; jälkimmäinen odotti tietokantaa ja epäonnistui minuuttien latauksen jälkeen.
- **Osoite, jossa on rikkinäinen `%`-koodaus, avaa ei löytynyt -sivun**, sen sijaan että latauskuvake jäisi pyörimään pysyvästi.
- **Suuri sivunumero luettelossa on tyhjä sivu**: se ylivuoti ja palasi taas ensimmäisenä sivuna.

### Fixed
- **Taulukot lajittelevat luvut oikein saksan- tai norjankielisessä selaimessa**: ”1,204” luettiin lukuna 1.204, ja kaikki miljoonaa suuremmat tekstinä.
- **Kaavion Taulukko-näkymä lajittelee päivämäärät, viikonpäivät ja kohdat ajan mukaan**, ei kirjainten mukaan.
- **Lataushistoria nimeää, mistä tuonnit tulivat**: Sonarr ja Radarr nimeävät indeksoijan ja protokollan vain kaappauksessa, joten nuo erittelyt olivat aina tyhjiä. Epäonnistunut kausipaketti on yksi epäonnistuminen eikä yksi jaksoa kohden.
- **Yhdessä katsotun toiston poistaminen** etsii nimikkeen ryhmät uudelleen sen sijaan, että jättäisi yhden hengen ”ryhmän”.
- **Yhden henkilön yhteinen aika ei koskaan ylitä hänen omaa läsnäoloaan**: Yhdessä-sivu hyvitti hänelle koko illan ajan.
- **Seurassa vietetty ilta uudenvuodenaaton keskiyön yli** (tai ”viimeiset N päivää” -jakson rajan yli) kuuluu kokonaan siihen jaksoon, jossa se alkoi, eikä siitä tule kahta iltaa yksin.
- **Tuodut jaksojen toistot nimetään jakson nimellä**, kun Jellystat säilytti vain sarjan nimen.
- **Paikannus löytää kaupunkitietokannan Country- ja ASN-tietokantojen vierestä**, jotka `geoipupdate` kirjoittaa, sen sijaan että kytkeytyisi itse pois.
- **Eniten taaksepäin kelattu kohta noudattaa sivun suodattimia**, kuten sen luvut.
- **”Vuotesi on valmis”** lähetetään vain niille, joiden vuosikatsauksessa on jotain.
- **Mykistetyn hälytyksen ratkaiseminen uudelleen pitää sen parin mykistettynä**; vain uudelleen avaaminen kumoaa mykistyksen.
- **Vanhentunut API-avain ei enää lasketa kahdenkymmenen joukkoon.**
- **Istuntoraja päästää vanhimman istunnon menemään**, vaikka monta kirjautumista osuisi samaan sekuntiin.
- **Tunnistuksen löytämä kotiosoite säilyy**, kun sama osoite poistetaan kirjoitetusta luettelosta.
- **Kuvat tarkastuslokissa ja avainluettelossa**, joissa oli vain nimikirjaimet.
- **Ylläpitäjän ilmoitusluettelo kertoo, kenen henkilökohtainen kohde on**, sen sijaan että jokaisessa lukisi ”Sinun”; **Äskettäin lähetetyt** näyttää viimeisimmän tapahtuman kaikki kohteet.
- **Esc nimikkeen tai henkilön sivulla ei koskaan vie pois FinStatsista**, ja se sulkee vain päällimmäisen valintaikkunan.
- **Sivu piirretään, vaikka tallennus peruisi esihaun, jota se odotti.**
- **Nyt toistossa näyttää taukotilassa kelaavan katsojan** siinä kohdassa, missä hän nyt on, eikä vasta kun hän painaa toistoa.
- **Yhteyden poistaminen päivittää, mitkä sivut ovat olemassa**, kuten yhteyden tallentaminen tekee.
- **Paikannustietokannan koontipäivämäärä säilyttää vuotensa** amerikanenglannissa.

## [2.0.2] - 2026-09-28

### Security
- Välityspalvelimen takana kirjautumisraja laski ensimmäisen `X-Forwarded-For`-osoitteen, jonka asiakas itse lähettää: uusi keksitty osoite jokaisella yrityksellä, eikä raja koskaan pätenyt. Nyt se laskee viimeisen, jonka välityspalvelin kirjoitti.
- *Sisällytä IP-osoitteet ja paikat* lähetti osoitteita henkilökohtaiseen kohteeseen, jonka omistajalla ei ole oikeutta *Näe verkkotiedot* ja jolle mikään sivu ei niitä näyttäisi. Jonkun toisen epäonnistuneet kirjautumiset vaativat nyt myös oikeuden *Näe palvelin*, kuten Tietoturva-sivulla.

### Stability
- **Pysäytystä ei koskaan hukata**: kun kirjaston luenta hylättiin heti käynnistyksen jälkeisinä hetkinä, ennen kuin palvelin kuunteli sitä, hylkäys kirjattiin ja FinStats jatkoi toimintaansa pysähtymisen sijaan.
- **Jellyfin, joka vastaa muodossa, jota FinStats ei osaa lukea, hylätään** eikä sitä uskota: jokainen käyttäjä tai kirjaston jokainen nimike, joka saapui tuntemattomien avainten alla (`id` eikä `Id`), läpäisi suojauksen täydellä määrällä ja merkittiin sitten poistetuksi.
- **Toiston loppua ei koskaan pudoteta**: kun tietokanta oli varattuna odotusaikaansa pidempään (tuonnin pitämänä), sillä hetkellä päättynyt toisto jäi merkityksi toistossa olevaksi seuraavaan uudelleenkäynnistykseen asti. Nyt se kirjoitetaan seuraavalla kierroksella.
- **Asetukset, joita palautus ei osaa lukea, jätetään rauhaan**: varmuuskopio, jonka asetukset eivät enää sopineet tähän versioon, nollasi palautuksessa jokaisen asetuksen: yhdistämisikkunan, oikeudet, kotiosoitteet, ajoituksen.
- **Ohjausmerkki nimessä tai nimikkeessä ei enää riko kortteja**: profiilikortti, jokainen nimikkeen näyttänyt tarinakortti ja niiden ZIP vastasivat virheellä.

### Fixed
- **Ilmoitukset loppuvat Jellyfinissä poistetulta tai käytöstä poistetulta henkilöltä**: hänen henkilökohtaisiin kohteisiinsa lähetettiin edelleen kaikki, mitä hänen vanhat oikeutensa sallivat.
- **Jonossa oleva ilmoitus tarkistetaan uudelleen ennen lähettämistä**: pois kytkettyyn kohteeseen tai oikeutensa menettäneelle omistajalle lähetettiin silti se, mikä odotti uutta yritystä.
- **Kohteesta, joka vastaa aina `Retry-After`, luovutaan** viiden yrityksen jälkeen kuten mistä tahansa muusta, sen sijaan että sitä yritettäisiin kolmenkymmenen päivän ajan.
- **Laitetunnisteet henkilön sivulla** näytetään vain oikeudella *Näe verkkotiedot*, kuten kaikkialla muualla.
- **Julkaistu vuosi odottaa päivän kaikessa**: sen loppuun katsotut ja kesken jätetyt sarjat seurasivat jokaista toistoa, joten kesken jätetyn sarjan toistaminen muutti julkista sivua sekunneissa.
- **Pyydetty sarja lasketaan katsotuksi**, kun jokin sen jaksoista on katsottu: jokainen pyydetty sarja näkyi koskaan katsomattomana Pyynnöt-luvussa ja Pyynnöt-kortilla.
- **Vuosikatsauksen Seurassa-luku jättää suorat TV-lähetykset pois**, kuten muukin vuosi: yhdessä päälle jätetystä kanavasta saattoi tulla kärkinimike, ja osuus saattoi ylittää 100 %.
- **”Viimeisten 12 kuukauden” Joka päivä -kortti päättyy uusimpaan päivään**: ruudukosta puuttuivat viimeisimmät kolme tai neljä viikkoa.
- **Sarjan uusista jaksoista ilmoitetaan kerran illassa**: UTC-keskiyön molemmin puolin julkaistuista jaksoista ilmoitettiin kahdesti palvelimella, joka on sen itä- tai länsipuolella.
- **Seerrin yhdistäminen ei ilmoita vanhoja pyyntöjä katseluvalmiiksi**: pyyntö, joka oli jo saatavilla ensimmäisellä luennalla ilman muuta tietoa siitä, milloin, näkyi juuri sillä hetkellä saapuneena.
- **Jellyfinissä poistetun kirjaston nimikkeet lähtevät sen mukana**: ne jäivät hakuun, Viimeksi lisätyt -hyllylle ja kokonaismääriin.
- **Remux pysyy suoratoistona**, kun lukema menettää hetkeksi transkoodaustietonsa (siirtymä), sen sijaan että siitä tulisi transkoodaus toiston loppuajaksi.
- **”Jellyfin ei vastaa” tarkoittaa peräkkäin epäonnistuneita luentoja**: päivien välein sattuneet yksittäiset aikakatkaisut palvelimella, joka oli joutilaana ja kuunteli socketia, kasautuivat siihen.
- **Streamystats-tuonti näyttää edistymisensä omalla kortillaan** eikä Jellystat-tuonnin viimeisimmän tuloksen päällä.
- **Osoite, jonka isäntänimi alkaa sanalla ”web”** (`http://web:8096`), luetaan isäntänimeksi eikä katkaista Jellyfinin verkkoasiakkaan poluksi.
- **Jonovirhe peittää osoitteen koko kirjautumistiedon**, vaikka salasanassa olisi kauttaviiva.

## [2.0.1] - 2026-09-28

### Changed
- **Nimikkeen kielet pysyvät poissa tieltä**: otsikkoalue nimeää kolme ensimmäistä ääni- ja tekstityskieltä, kumpikin yhdellä rivillä, ja uusi **Kielet**-kortti luettelee ne kaikki: kokonaiset yhdessä ja osittaiset sen kanssa, montako jaksoa niillä on. Pitkän sarjan parikymmentä tekstityskieltä työnsivät ennen taustakuvaa näytön verran alaspäin.

### Fixed
- **Nimikkeen juliste pysyy sen nimen vieressä**: kun viereinen teksti oli julistetta pidempi (pitkä juoniselostus, monta studiota), juliste valui otsikkoalueen pohjalle tyhjän taustakuvan alle.
- **Versio, jota FinStats ei osaa lukea, kerrotaan sellaiseksi**: käsin lukukelvottomaksi muokattu `app_version` sai FinStatsin kieltäytymään käynnistymästä ”vanhempana FinStats 2.0.1:nä”, mitä se ei ollut. Se kieltäytyy edelleen, ja kertoo nyt, mitä arvoa se ei pystynyt lukemaan ja miten se kirjoitetaan.

## [2.0.0] - 2026-09-25 to 2026-09-28

Vuosikatsaus 2026, julkiset profiilit ja yhteiskatselu

### Added
- **Kenen kanssa katsoit**, vuosikatsauksessa: seurassa vietetyt illat, niihin käytetty aika, nimike, joka kokosi ihmiset yhteen useimmin, ja enintään kolme seuralaista, sovelluksessa nimettyinä ja linkitettyinä profiileihinsa.
- **Loppuun katsotut sarjat**, vuosikatsauksessa: kunkin vuoden aikana katsotun sarjan viimeinen jakso, samalla ”nähdyn” tulkinnalla kuin profiilin edistymispalkit, sekä aloitetut ja myöhemmäksi jätetyt, alle puoliksi nähdyt, joista mitään ei ole toistettu kahteen kuukauteen.
- **Mitä pyysit**, vuosikatsauksessa: vuoden aikana tehdyt Seerr-pyynnöt, montako saapui ja montako niistä sitten katsoit. Vain siellä, missä pyyntöjä on.
- **Vuosi verrattuna edelliseen**: tunnit, toistot ja päivät, ja kuinka paljon enemmän tai vähemmän.
- **Vuosi tarinana**: jokainen luku 1080×1920-korttina, siinä muodossa, jota puhelinten tarinat käyttävät (vuosi, luvut, suosikkisarjat, -elokuvat ja -musiikki, lajityypit, katsojatyyppi, tunnit ja päivät, vuoden jokainen päivä, ennätykset, seura, loppuun katsotut sarjat, pyynnöt ja edellinen vuosi). Tallenna yksi, tai kaikki ZIP-tiedostona, kohdasta ”Jaa vuosi kortteina”.
- **Kortti ei koskaan nimeä ketään muuta**: kortit piirretään vuoden kopiosta, jossa ei ole kenttää toiselle henkilölle, sijoitukselle tai sovellukselle. Sovelluksessa kortissa ei ole nimeä lainkaan.
- **Palvelimen vuosi**, Jellyfinin ylläpitäjille: jokainen nimike ja kokonaismäärä sekä talon katsojatyyppi, ketään sijoittamatta tai nimeämättä. Kytkin vuosikatsaussivulla, `scope=server` API:ssa.
- **”Vuosikatsauksesi on valmis”**: ilmoitus joulukuussa, kerran jokaiselle, joka katsoi jotain sinä vuonna.
- **Julkiset profiilit**: henkilö voi julkaista osan profiilistaan linkissä, jonka kuka tahansa voi avata ilman tiliä (kokonaismäärät ja suosikkinimikkeet, putket ja katseluajat, koko vuosi korttien tarinana, viimeiset kymmenen toistoa), jokainen osa pois päältä, kunnes sen omistaja kytkee sen päälle kohdassa Asetukset → Julkinen profiili. Laitteita, osoitteita, sovelluksia, toistotapoja, tiedostopolkuja ja muita ihmisiä ei koskaan julkaista, olipa mitä tahansa kytketty päälle, ja näytetty nimi on omistajan kirjoittama, ei koskaan hänen Jellyfin-käyttäjätunnuksensa.
- **Mikään julkaistulla sivulla ei ole päivää uudempaa**, ei viimeisimmät toistot eikä mikään kokonaismäärä, joten sivu, jota joku seuraa, ei koskaan kerro, kuka katsoo juuri nyt.
- **Jaettavat kortit**: jokaisella julkaistulla profiililla on 1200×630-kuva profiilista ja vuodesta, jonka Discord, Signal, Mastodon ja muut näyttävät, kun linkki liitetään, ja jonka voi ladata julkaistavaksi. Piirretty palvelimella siitä, mitä sivu näyttää, eikä mistään muusta.
- **Linkki, jonka voit perua**: 22 satunnaista merkkiä, vaihdettavissa yhdellä napsautuksella; vanha linkki vastaa sen jälkeen täsmälleen kuin linkki, jota ei koskaan ollut.
- **Julkiset profiilit ovat pois päältä, kunnes ylläpitäjä sallii ne**: yksi kytkin koko palvelimelle, luettelo siitä, kuka julkaisee mitä, ja keino poistaa profiili. Julkaisut, muutokset, vaihdot ja poistot näkyvät tarkastuslokissa, eikä avaimella voi tehdä mitään julkiseksi.
- **Yhdessä**, oma sivunsa: kuka katsoo kenen kanssa, kuinka paljon ja miten se on muuttunut. Yhteinen aika, illat ja seurassa vietetyn katseluajan osuus, kukin verrattuna edelliseen jaksoon; tunnit seurassa ja yksin päivittäin tai viikoittain; jokainen henkilöpari iltoineen, yhteisine aikoineen, viimeisine katselukertoineen ja eniten katsottuine nimikkeineen; eniten yhdessä katsotut nimikkeet; viimeisimmät illat; ja niille, jotka saavat nähdä kaikki, kunkin henkilön aika yksin verrattuna aikaan seurassa.
- Jokainen pari lasketaan: kolmen hengen ilta lasketaan jokaiselle sen kolmesta parista, kun kortti laski ennen vain täsmälleen saman henkilöjoukon. Parin yhteinen aika on kahdesta läsnäolosta lyhyempi.
- Yleiskatsauksen ja profiilin **Katsottu yhdessä** -kortti linkittää sivulle, ja profiilin kortti kertoo, kuinka suuri osa henkilön katseluajasta kului seurassa.
- `GET /api/stats/groups` saa kentät `pairs`, `series`, `people`, `previous`, `bucket` sekä kokonaismääriinsä `watch_s` ja `share`; aiemmat avaimet pysyvät ennallaan.
- **Mihin katsojat lopettavat**, jokaisella elokuvan ja jakson sivulla: kuinka suuri osa katsojista on yhä mukana kullakin minuutilla, alun kaikista loppuun asti päässeisiin, ja niiden toistojen taaksepäin kelaukset ja tekstitysten päälle kytkennät merkittyinä samalle akselille. Kortti kertoo, montako lopetusta mitattiin (FinStatsin tallentamat toistot) ja montako arvioitiin (tuodut toistot, niiden keston perusteella), ja se tarvitsee kolme toistoa ennen kuin piirtää mitään.
- **Kuka jatkaa katsomista**, sarjan sivulla: montako henkilöä aloitti kunkin jakson, järjestyksessä, ja **Katsottu loppuun** -sarake jaksoittain. ”Kaikki lopettavat jaksoon kolme” tarkoittaa ihmisiä, jotka eivät koskaan paina toistoa jaksossa neljä.
- **Tiedostot, jotka eivät koskaan toistu**, Toisto-sivulla: elokuva tai jakso, joka on aloitettu vähintään kolmesti eikä koskaan päässyt kolmeakymmentä sekuntia pidemmälle, ja kuka yritti ja millä sovelluksilla. Jellyfinin loki ei sisällä toistovirheitä, joten toistot ovat todistajia; toiston vähimmäispituus ei päde tässä, koska juuri sitä nämä toistot eivät koskaan saavuta.
- **Eniten kelattu taaksepäin** ja **Tekstitykset päällä**, Toisto-sivulla: nimikkeet järjestettyinä taaksepäin hyppäysten määrän mukaan toistoa kohden sekä minuutin mukaan, johon ne kasautuvat, ja sen osuuden mukaan toistoista, joissa tekstitykset kytkettiin päälle ensimmäisten kymmenen minuutin aikana. Kaikki kolme luetteloa vaativat oikeuden *Näe kaikkien toiminta*. Alle 20 sekunnin hyppyjä ei koskaan tallennettu, joten yhden repliikin lyhyt uusinta on tarkoituksella näkymätön.
- Siirtymä säilyttää lähtökohtansa lukuna nimiönsä vieressä (`from_s` toiston aikajanalla). Jokainen jo tallennettu siirtymä saa lukunsa nimiöstä, samoin tätä vanhempi palautettu varmuuskopio.
- `GET /api/stats/files` kolmea luetteloa varten; `GET /api/items/{id}` saa kentän `insights`, ja sarjan jaksot saavat kentät `users` ja `finished`.
- **API-avaimet**: kuka tahansa kirjautunut voi luoda itselleen avaimia kohdassa Asetukset → API-avaimet skriptiä, kojelautaa tai puhelinta varten. Avain lähetetään muodossa `Authorization: Bearer fs_…`, se näytetään kerran, tallennetaan tiivisteenä, sillä voi olla vanhenemisaika, ja sen voi peruuttaa napsautuksella; ylläpitäjä näkee kaikkien avaimet ja voi peruuttaa ne. Avain toimii saman henkilön nimissä samoin oikeuksin kuin kirjautuminen, ja oikeudet luetaan ajantasaisina jokaisella pyynnöllä, joten kirjautumisoikeuden menettäminen pysäyttää jokaisen avaimen kerralla. Avaimella ei voi luoda eikä peruuttaa avaimia.
- **Kalenterisyöte**: `GET /api/calendar.ics?key=…` on se, mitä Sonarrilla ja Radarrilla on tulossa, iCalendar-muodossa, jonka puhelin voi tilata. Avain, jonka laajuus on *calendar*, avaa syötteen eikä mitään muuta; syöte lukee avaimen osoitteesta, koska tilattu kalenteri ei voi lähettää otsaketta, ei koskaan evästeestä, eikä se nimeä ketään.
- **Tarkastusloki** kohdassa Palvelin → Tarkastusloki, Jellyfinin ylläpitäjille: jokainen kirjautuminen ja epäonnistunut yritys osoitteineen, jokainen muutettu asetus tai oikeus ja mihin se muutettiin, jokainen luotu, käytetty tai peruutettu avain, jokainen yhteys, kohde, varmuuskopio ja tuonti, jokainen poistettu toisto ja ratkaistu hälytys, sekä kuka, mistä, minkä avaimen kautta ja onnistuiko se. Säilytetään vuoden, kuuluu varmuuskopioihin. `GET /api/audit` selaa sitä sivuittain.
- **Tuonti Streamystatsista**, oma osionsa Asetuksissa Jellystatin vieressä. Ota sen **Settings → Backup & Import → Download Backup** -tiedosto ja pudota se tänne: vuosien historia saapuu parissa sekunnissa, eikä mitään jo olemassa olevaa lasketa uudelleen. `finstats import-streamystats <file>` tekee sen ilman käyttöliittymää, `POST /api/import/streamystats` API:n kautta; tuonteja ajetaan yksi kerrallaan, kumpaa lajia tahansa.
- **Käytitkö molempia seurantatyökaluja? Tuo molemmat tiedostot.** Toisto tunnistetaan sellaiseksi, joka FinStatsilla jo on, seurantatyökalun omasta tunnisteesta ja, jos sitä ei ole, siitä, että sama henkilö katsoo samaa kohdetta ja toiston jompikumpi pää on lähellä historiassa olevaa, joten saman illan Jellystat-vienti ja Streamystats-vienti voidaan tuoda molemmat, kummassa järjestyksessä tahansa, eikä FinStatsin itse seuraaman illan päälle tuoda kumpaakaan. ”Lähellä” tarkoittaa yhdistämisikkunaa kohdassa Asetukset → Keräys.
- Tuodut rivit kertovat, mitä ne eivät voi kertoa. Streamystatsin itse seuraama toisto ei säilytä tiedostosta mitään (ei koodekkia, resoluutiota, bittinopeutta tai säiliötä), joten ne jäävät tyhjiksi sen sijaan, että ne täytettäisiin arvauksella, ja kortit, jotka näyttäisivät ne, piilottavat itsensä. Jellystatista otettu toisto sisältää koko istunnon, ja se luetaan sen sijaan: päivättynä hetkeen, jolloin se päättyi, ainoaan hetkeen, jonka kumpikin seurantatyökalu säilytti, ja kuvattuna transkoodaukseksi vain, jos istunto todella oli sellainen.
- Rivit, jotka Streamystats kirjoitti, koska Jellyfin ilmoitti kohteen katsotuksi, katselusta, jota se ei koskaan nähnyt, lasketaan ja ohitetaan: ne ovat koko elokuvan mittaisia ja olisivat katseluaikaa illalta, jona kukaan ei katsonut. Tulos kertoo, montako niitä oli.
- Miten Streamystatsin tiedot tulkitaan kenttä kentältä, opittuna todellisesta viennistä eikä dokumentaatiosta: `docs/streamystats-import.md`.
- **Mistä seurantatyökalusta kukin toisto tuli**, suodattimena Toiminta-sivulla: minkä FinStats tallensi itse, mikä tuli Jellystatista, mikä Streamystatsista. Tarjolla ovat vain ne, joita historiassasi todella on, ja se rajataan kuten jokainen muu luettelo, joten henkilölle, joka saa nähdä vain omat toistonsa, kerrotaan vain, mistä hänen oma historiansa tuli.
- **Toiminta-sivun suodattimet ovat valintaruutuja sisältäviä pudotusvalikoita**, joten voit pyytää elokuvia *ja* jaksoja ilman musiikkia, kahta toistotapaa tai kahta henkilöä kerralla, eikä vain yhtä kutakin tai kaikkia. Mediatyyppi sai vaihtoehdon **Muu**, nyt kun niitä voi yhdistellä.
- Useita henkilöitä kerralla kaikkialla, missä käyttäjäsuodatin on (yleiskatsaus, toisto, kirjastot, tietoturva ja muut), ja kartta sekä jokainen tilasto seuraavat valintaa. Käyttäjäsuodatin vain kaventaa sitä, mitä joku saa nähdä: ilman oikeutta *Näe kaikkien toiminta* pyyntö kiinnitetään pyytäjään, nimesipä osoite kenet tahansa ja kuinka monta tahansa.
- **Vaalea teema**: washi-paperia, mustetekstiä ja punaoranssi korostusväri. FinStats seuraa laitteen vaaleaa tai tummaa asetusta, kunnes valitset jommankumman.
- **Teemapainike Kirjaudu ulos -painikkeen vieressä** kiertää vaihtoehdot Laite, Vaalea ja Tumma. Valinta säilytetään tässä selaimessa ja otetaan käyttöön ennen sivun piirtämistä, joten toinen teema ei koskaan välähdä.
- **Avaa Jellyfinissä**: jokaisella elokuvan, sarjan ja jakson sivulla on painike, joka avaa nimikkeen sivun Jellyfinissä uudessa välilehdessä.
- **Jellyfinin osoite ihmisille** kohdassa Asetukset → Jellyfin: minne nuo painikkeet osoittavat, kun osoite, johon FinStats yhdistää, ei ole selaimen tavoitettavissa. Tyhjänä käytetään osoitetta, johon FinStats yhdistää; vain Jellyfinin ylläpitäjä voi asettaa sen.

### Changed
- **FinStatsilla on uusi koti**: `github.com/finstats/finstats`, ja levykuva on `ghcr.io/finstats/finstats`. Vaihda levykuvan nimi `docker run` -komentoosi tai compose-tiedostoosi, jotta saat päivitykset jatkossakin; datakansio pysyy ennallaan.
- **Asetuksissa on näytöllä yksi osio kerrallaan.** Sivu oli neljätoista korttia yhdessä sarakkeessa ja kuusisataa sanaa ohjetta. Nyt jokaisella osiolla on oma osoitteensa (`/settings/collection`), vasemmalla olevalla luettelolla siirrytään niiden välillä, ja puhelimella luettelo rivittyy siruiksi. Linkit vanhoihin ankkureihin osuvat yhä perille.
- **Asetus on yksi rivi**: mikä se on, yksi rivi, joka kertoo säännön, ja säädin oikealla. Osion luvut tallentuvat yhdessä yhdellä painikkeella alareunassa; kytkimet tallentuvat yhä itsestään. Tilakortit (Tehtävät, Lähtevät yhteydet, Tietokanta) ja lisenssilinkki ovat kohdassa **Järjestelmä**, ja kahdella tuojalla on oma osionsa.
- **Etsi asetus** otsikkoalueen kentästä (`/` kohdistaa siihen) millä tahansa sen nimen sanalla; se kertoo, missä osiossa asetus on, ja Enter vie riville.
- **Asetukset ovat kaikille kirjautuneille**: palvelinta hallinnoivat osiot jäävät niille, jotka saavat hallinnoida sitä, ja henkilö, jolle oli sallittu vain ilmoitukset, pääsee vihdoin omiin kohteisiinsa.
- **Palvelin-sivulla on näytöllä yksi osio kerrallaan**, kuten Asetuksissa: Yleiskatsaus (järjestelmä ja tallennustila), Tehtävät, Laitteet, Lisäosat, Loki ja Tarkastusloki, kukin omassa osoitteessaan. Linkit osoitteeseen `/server#jobs` osuvat yhä perille.
- **Palvelimen loki on Palvelin-sivun osio** (`/server/log`) eikä oma sivunsa valikossa; `/events` ohjaa sinne suodattimineen.
- Ilmoitus epäonnistuneesta työstä, epäonnistuneesta varmuuskopiosta tai yhteydestä, joka lakkasi vastaamasta, linkittää sen sisältävään asetusosioon eikä sivun alkuun.
- Se, mitä kirjastosi voi kertoa tuodusta toistosta (millainen kohde se oli, mikä jakso, sen kesto, mihin kirjastoon se kuuluu), täydennetään **jokaisen** kirjaston luennan jälkeen, ei vain tuonnin hetkellä. Historia tuodaan yleensä ennen kuin FinStats on lukenut kirjastoa kertaakaan, joten nuo toistot jäivät ennen pysyvästi arvatuiksi tai tuntemattomiksi.
- FinStats-varmuuskopion palauttaminen ei enää lisää toistoa, joka saapui seurantatyökalusta eri tunnisteella, ja säilyttää silti kaksi aitoa saman asian katselukertaa minuuttien välein.
- Toiston tiedot nimeävät sen seurantatyökalun, joka toi sen, sen sijaan että kaikkea tuotua kutsuttaisiin Jellystat-toistoksi.

### Performance
- **Uudelleenkäynnistys on lähes välitön historian koosta riippumatta.** Jokainen käynnistys laski koko historiasta, kuka katsoi yhdessä, ja ratkaisi uudelleen, mitkä toistot olivat paikallisia, molemmat muuttamatta mitään: 24 s ja 5,5 s kymmenellä miljoonalla toistolla. Käynnistys tekee nyt vain sen, mitä edellinen ajo jätti tekemättä, ja kaiken uudelleen vain päivityksen jälkeen tai kun taustalla oleva asetus muuttui.
- **Kerralla saapuva joukko tallennetaan sekunneissa.** Jokainen uusi toisto, jokainen edistymisen tallennus ja jokainen laite kirjoitettiin erikseen, ja jokainen päättynyt toisto suljettiin katsomalla erikseen nimikkeen koko historiaa; kierros kirjoittaa nyt kunkin lajin kerralla. 100 000 yhtä aikaa alkavaa toistoa vie 3 s, kun 25 000 yhtä aikaa päättyvää vei ennen yli neljä minuuttia.
- **Nyt toistossa pysyy nopeana tuhansienkin katsellessa.** Yhdessä katsovien etsiminen vertasi jokaista toistoa jokaiseen muuhun ja luetteli jokaisen seuralaisen nimeltä: neljäkymmentä sekuntia 10 000 toistolla ja gigatavuja muistia sen yli. Toistoja verrataan nyt nimikkeen sisällä, ja kukin näyttää muutaman nimen ja montako muuta.
- Yhdessä katsomisen laskeminen kirjoitti historian jokaisen ryhmitellyn toiston uudelleen aina, kun FinStats käynnistyi, toi tietoja tai näki toiston päättyvän; nyt se kirjoittaa vain muuttuneen. Päättyneiden toistojen etsiminen ei enää hidastu toistossa olevien määrän neliössä.
- **Historiassa jo olevan toiston tunnistaminen käyttää indeksiä**, joten 150 000 toiston tuonti vie sekunteja, ja uudelleenlinkitys kirjaston luennan jälkeen käy läpi nimikkeitä sen sijaan, että lukisi jokaisen toiston pitäen tietokantaa varattuna.
- **Toiston oivallukset ja yleiskatsaus ovat nopeampia koko ajalta**: niiden toisistaan riippumattomat osat lasketaan rinnakkain, samanaikaisuuskaavio hakee paikallisen päivän kerran vartissa eikä kerran toistoa kohden, ja lajityypit lasketaan nimikkeittäin. Miljoonalla toistolla: 4,1 s → 2,4 s ja 1,6 s → 1,1 s.

### Stability
- **TV-kirjasto ei enää näytä tyhjennetyltä kirjastolta.** Kirjaston luenta tekee kaksi kierrosta (jokainen kohde, sitten pienempi kierros vain elokuvien ja sarjojen näyttelijöille ja tekijöille), ja tarkistus, joka ratkaisee, voiko luennan perusteella poistaa sen, mitä se ei nähnyt, katsoi jälkimmäistä. TV- tai musiikkikirjastossa nämä kaksi määrää eivät ole lähelläkään toisiaan, joten tarkistus hylkäsi jokaisen luennan ja pysäytti FinStatsin, ja asennus käynnistyi uudelleen ja teki saman taas. Mitään ei koskaan poistettu aiheetta, mutta yksikään kirjaston luenta ei päässyt loppuun. Jos Sarjat- tai Musiikki-kirjastosi ei ole poiminut uusia nimikkeitä, tämä on syy.
- Tuonti saattoi epäonnistua virheeseen ”database is locked”, kun toinen kirjoitus (päivitettävä toisto) osui sen ensimmäisen luennan ja ensimmäisen kirjoituksen väliin. Molemmat tuojat ottavat kirjoituslukon heti alussa, eikä tietokannan lokitiedosto enää pysy yhtä suurena kuin suurin koskaan tehty tuonti.
- **Vaurioitunut, katkennut, jonkun toisen tai vain luku -tilassa oleva tietokantatiedosto hylätään heti**, tiedosto nimeten ja kertoen, ettei mitään muutettu, 30 hiljaisen sekunnin sijaan; toisen ohjelman tietokantaan ei koskaan kirjoiteta.
- Puolivälissä keskeytetty varmuuskopio tai päivitystä edeltävä tilannevedos ei enää jätä tiedostoa jälkeensä, joka voisi viedä hyvän tilannevedoksen paikan kolmen säilytettävän joukossa.
- **Paikkatietokannan korvaaminen FinStatsin ollessa käynnissä ei enää kaada sitä**: FinStats lukee tiedoston yksityistä kopiota.
- Jellyfin-sovellus, joka ilmoitti joka kerta uuden laitetunnisteen, saattoi kasvattaa FinStatsin muistinkäyttöä niin kauan kuin se oli käynnissä; laitteet, joita ei ole nähty viiteen minuuttiin, unohdetaan nyt.
- Yhdellä henkilöllä on enintään 30 kirjautunutta istuntoa; vanhin kirjataan ulos seuraavassa kirjautumisessa.

### Fixed
- **Samaa iltaa ei enää lasketa kahdesti, kun kohde on nimetty uudelleen.** Historian uudelleenlinkitys Jellyfiniin uudelleen lisättyyn nimikkeeseen saattoi tehdä seurantatyökalusta tuodusta toistosta kaksoiskappaleen jo olemassa olevasta, koska tarkistus, joka olisi huomannut sen, ajettiin ennen tunnisteen siirtymistä. Nyt se ajetaan uudelleen aina, kun tunnisteita linkitetään uudelleen. Vain tuotuja toistoja poistetaan, ei koskaan FinStatsin itse tallentamaa, ja poistettujen määrä näkyy lokissa. Version 2.0 ensimmäinen käynnistys käy koko historian kerran läpi jo olemassa olevien varalta.
- **Istunto seuraa nyt henkilöä sellaisena kuin Jellyfin hänet nyt tuntee.** Ylläpitäjä, jonka Jellyfin alensi, säilytti täydet oikeudet FinStatsiin 30 päivän istuntonsa loppuun, ja Jellyfinissä käytöstä poistettu tai poistettu henkilö pysyi kirjautuneena. Nyt hän menettää pääsyn heti, kun FinStats lukee Jellyfinin käyttäjät (15 minuutin välein tai heti Asetuksista), kuten API-avaimille on aina käynyt.
- Kirjautuminen kirjoittaa sen, mitä Jellyfin kertoo henkilöstä, FinStatsin käyttäjäluetteloon, joten tuotu seurantatyökalun varmuuskopio ei enää voi tehdä kenestäkään ylläpitäjää ennen kuin käyttäjien ensimmäinen luenta korjaa sen.
- Jellyfinissä poistettu henkilö merkitään poistetuksi, vaikka kaksi käyttäjien luentaa osuisi samaan sekuntiin.
- **Latausvirheet eivät koskaan näytä latausohjelman salasanaa tai indeksoijan API-avainta**, joita Sonarr ja Radarr saattavat niissä lainata.
- Kirjaston luennan aikana uudelleennimettyyn nimikkeeseen linkitettyjä toistoja ei tarkistettu yhteiskatselun varalta ennen seuraavaa uudelleenkäynnistystä; nyt ne tarkistetaan heti.

## [1.6.5] - 2026-09-24

### Added
- **Täysi varmuuskopio otetaan automaattisesti ennen jokaista päivitystä.** Kun uudempi FinStats avaa tietokantasi ensimmäisen kerran, se kirjoittaa siitä täydellisen kopion (koko historiasi, asetukset ja kaikki) kansioon `data/pre-update-backups/` *ennen* kuin muuttaa mitään. Jos päivitys joskus rikkoo jotain, tietosi ovat turvassa: pysäytä FinStats, laita kopio tiedoston `finstats.db` paikalle, käynnistä aiemmin käyttämäsi versio ja ilmoita virheestä. Mitään ei menetetä. Muutama uusin kopio säilytetään; aseta `FINSTATS_SKIP_PREUPDATE_BACKUP=1` kytkeäksesi tämän pois, jos levytilaa on niukasti.

## [1.6.4] - 2026-09-24

### Added
- **Viisi uutta ilmoituskohdetta**: Telegram, Slack, Pushover, Pushbullet ja sähköposti, jo olemassa olevien webhook-, Discord-, ntfy- ja Gotify-kohteiden lisäksi. Kukin kysyy vain tarvitsemansa (keskustelun tunnisteen, käyttäjäavaimen, postilaatikon), ja ne kolme, jotka toimivat yhden palvelun kautta (Telegram, Pushover, Pushbullet), tavoitetaan aina vain sieltä, joten tunnusta ei voi lähettää samannäköiselle palvelimelle. Sähköposti lähtee TLS-salattuna ensimmäisestä tavusta alkaen tai `STARTTLS`-yhteydellä, eikä kolmatta vaihtoehtoa ole.
- **Lisenssit**, kohdassa **Asetukset**: FinStatsin oma GPL ja jokainen kolmannen osapuolen lisenssi, jonka varaan se on rakennettu (253 Rust-pakettia, kaksi mukana tulevaa fonttia, kartan ääriviivat ja paikannustietokanta), kukin teksti kokonaisuudessaan, sellaisena kuin sen projekti sen kirjoitti. Valitse komponentti, niin sen lisenssi avautuu luettelon viereiseen ikkunaan; useita tiedostoja sisältävän paketin tiedostojen välillä vaihdetaan paikan päällä. Luettelo luodaan niistä paketeista, joista FinStats todella rakennetaan, joten riippuvuutta ei voi lisätä ilman, että sen lisenssi kirjataan.

## [1.6.3] - 2026-09-24

### Fixed
- Turvavahti puuttuu nyt asiaan myös silloin, kun Jellyfin ei luettele *yhtään* kirjastoa tai *yhtään* käyttäjää (rikkinäinen luenta, ei koskaan tavallinen päivä), minkä 1.6.2:n hienosäätö oli päästänyt läpi. Joidenkin kirjastojen katoamista pidetään yhä tavallisena; kaikkien katoamista kerralla ei.

## [1.6.2] - 2026-09-24

### Fixed
- Versiossa 1.6.1 lisätty tietojen eheyden vahti oli liian innokas: kun kirjasto todella menetti suurimman osan kohteistaan (esimerkiksi kourallinen leikkeitä, joiden tiedostot katosivat), sitä luultiin rikkinäiseksi luennaksi, joten FinStats kieltäytyi päivittämästä sitä ja pysähtyi. Nyt se puuttuu vain *selvästi* rikkinäiseen luentaan: merkittävän kokoinen kirjasto, joka luetaan täysin tyhjänä, tai suuri kirjasto, joka on kutistunut lähes olemattomiin. Arkiset muutokset tehdään kuten ennenkin, ja tapaus ”Jellyfin ei palauttanut mitään” huomataan yhä.

## [1.6.1] - 2026-09-24

### Added
- **FinStats ei enää luota Jellyfin-luentaan, joka pyyhkisi historiasi.** Jos kirjasto, jolle FinStatsilla on tuhansia kohteita, luetaan yhtäkkiä tyhjänä tai lähes tyhjänä (Jellyfin-päivitys, joka muutti sen API:a, tyhjäksi tulokseksi naamioitunut virhe), FinStats pitää tietonsa, merkitsee synkronoinnin epäonnistuneeksi (ja ilmoittaa siitä) ja pysähtyy hallitusti selkeällä viestillä sen sijaan, että merkitsisi kaiken poistetuksi. Käynnistä uudelleen, kun Jellyfin on taas oma itsensä, tai aseta `FINSTATS_ALLOW_LIBRARY_SHRINK=1`, jos todella tyhjensit kirjaston. Sama vahti kattaa kirjastoluettelon ja käyttäjäluettelon.

### Security
- Asiakkaan lähettämä `X-Forwarded-Host`-otsake saattoi taata vieraan `Origin`-arvon ja päästää sivustojen välisen kirjoituksen saman alkuperän suojauksen ohi. Otsakkeeseen luotetaan nyt vain luotetun välityspalvelimen takana (`FINSTATS_TRUST_PROXY`), samalla säännöllä, jota asiakkaan osoitteen haku jo käyttää. `SameSite=Lax`-istuntoeväste oli ja on edelleen ensimmäinen puolustuslinja.
- Henkilökohtaisen ilmoituskohteen saattoi suunnata osoitteeseen `0.0.0.0` tai `::`, jotka osuvat tähän koneeseen. Tarkistus, joka pitää henkilökohtaiset kohteet julkisissa osoitteissa, laskee nyt nämä paikallisiksi silmukkaosoitteen ja yksityisten osoitealueiden lisäksi.

### Fixed
- **Toisto**-sivu latautuu nyt myös kirjastolle, jossa on toisto ilman tallennettua toistotapaa (rivi, joka on palautettu saraketta vanhemmasta varmuuskopiosta); tällainen toisto lasketaan yksinkertaisesti suoraksi toistoksi.
- Hyvin pitkä sana haussa tai suodattimessa ei enää aiheuta palvelinvirhettä: SQLite hylkää yli 50 000 merkin `LIKE`-hahmon, joten suodatinsanat lyhennetään järkevän mittaisiksi ennen kyselyä.
- **Asetukset**-sivu ei enää vieri sivuttain puhelimella, kun kortti näyttää kokonaisen lauseen, kuten palautuksen tuloksen.

## [1.6.0] - 2026-09-24

Ilmoitukset

### Added
- **Ilmoitukset**-kortti Asetuksissa: lisää kohde, valitse, mistä sille kerrotaan, ja paina Testaa. Ilman kohdetta mihinkään ei lähetetä pyyntöä.
- Neljä kohdelajia: tavallinen **webhook** (JSON, dokumentoitu tiedostossa `docs/api.md`, valinnainen `Authorization`-otsake), **Discord** (upotus kanavan webhook-osoitteen kautta), **ntfy** (ntfy.sh tai oma palvelin, aiheeseen) ja **Gotify** (oma palvelin, sovellustunnuksella). Nimikkeet kulkevat viestissä eivätkä otsakkeessa.
- Yksitoista tapahtumalajia, kukin omalla valinnallaan: mahdoton matkustus, tili uudessa maassa, epäonnistuneiden kirjautumisten ryöppy, epäonnistunut tehtävä, varmuuskopio, jota ei voitu kirjoittaa, jokin, joka lakkasi vastaamasta (Jellyfin, Sonarr, Radarr tai Seerr) ja kun se vastaa taas, uudet nimikkeet kirjastossa (koottuina sarjoittain ja päivittäin, kuten yleiskatsauksen hylly ne kokoaa), katseltavaksi tullut pyyntö sekä toiston alkaminen tai päättyminen. Kaksi viimeistä ovat oletuksena pois päältä.
- Katseltavaksi tullut pyyntö nimeää nimikkeen, pyytäjän ja sen, kuinka kauan hän odotti.
- Henkilökohtaiset kohteet: myönnä **Vastaanota ilmoituksia**, niin henkilö voi lisätä oman kohteen, johon lähetetään vain se, mitä hän saa jo nähdä: omat pyynnöt ja hälytykset aina, muiden vastaavat oikeudella *Näe kaikkien toiminta*, osoitteet oikeudella *Näe verkkotiedot*. Henkilökohtaisen kohteen on osoitettava julkiseen osoitteeseen; vain ylläpitäjä voi suunnata sellaisen verkon sisälle.
- **Sisällytä IP-osoitteet ja paikat**, kohdekohtainen ja oletuksena pois päältä: muuten viesti mahdottomasta matkustuksesta nimeää vain paikat (”Oslo, Norja ja Lontoo, Yhdistynyt kuningaskunta”).
- **Äskettäin lähetetyt**, samassa kortissa: mitä sanottiin, mihin kohteeseen ja mitä toinen pää vastasi, kun viesti ei mennyt perille. Epäonnistunutta viestiä yritetään uudelleen neljästi noin tunnin aikana, ja nopeutta rajoittavaa kohdetta odotetaan.
- **FinStatsin osoite**, ylläpitäjien asetus, on se, josta ilmoituksen linkki rakennetaan. Tyhjänä viesteissä ei ole linkkiä.
- Jellyfinin omat ajastetut tehtävät Palvelin-sivulla: mitä kukin tekee sanoin eikä koodinsa nimellä, ja onko se raskas. Käynnissä oleva tehtävä näyttää edistymispalkin, prosentin ja jäljellä olevan ajan; muut näyttävät, milloin ne ajetaan seuraavaksi, milloin ne ajettiin viimeksi ja onnistuiko se. Mukana ovat myös tehtävät, jotka Jellyfin piilottaa omalta hallintapaneeliltaan, eikä mikään FinStatsissa voi käynnistää tai pysäyttää tehtävää.
- Tehtävästä jäljellä oleva osuus mitataan siitä tahdista, jolla sen prosentti on todella liikkunut, verrattuna uusimpaan lukemaan, joka on tarpeeksi kaukana merkitäkseen jotain, eikä koko ajon keskiarvosta, joten nopeutuvaa tai hidastuvaa tehtävää kuvataan sen nykyisellä tahdilla.
- Tehtäväluettelot, jotka FinStats lukee jo muista syistä (kirjastoskannauksen tarkistus 5 minuutin välein, palvelimen tiedot 15 minuutin välein), syöttävät samaa mittausta, joten tunnin käynnissä olleella tehtävällä on yleensä arvio heti sivun avautuessa. Jellyfiniltä ei kysytä sitä varten mitään ylimääräistä.
- Käynnissä oleva tehtävä, jolle ei ole vielä mitattu mitään, näyttää luvun sijaan **ETA** ja kiertävät pisteet, ja jos tehtävän prosentti ei ole liikkunut puoleen minuuttiin, sen vieressä lukee ”68 % 4 min ajan”. Mikään ei korvaa arviota, jota ei ole mitattu.
- Luettelo kaikista versioista muutoslokin vieressä: yhdellä napsautuksella pääsee mihin tahansa julkaisuun, se merkitsee luettavana olevan, ja kapealla näytöllä siitä tulee sarjojen rivi merkintöjen yläpuolelle.

### Changed
- Muutosloki ei enää laskostu. Jokainen sarja on auki, ja versioluettelolla liikutaan, sen sijaan että yksi ryhmä olisi kerrallaan auki ja muihin pääsisi napsauttamalla.
- **Asetukset → Lähtevät yhteydet** luettelee jokaisen ilmoituskohteen Jellyfinin ja palveluyhteyksien rinnalla: mikä se on, sen isäntä, onko se kytketty päälle ja milloin se viimeksi otti viestin vastaan.
- Mikään keräyksestä, historiasta tai mistään näytöllä olevasta luvusta ei muutu. FinStats edelleen vain *lukee* Jellyfinistä, Sonarrista, Radarrista ja Seerristä.

## [1.5.2] - 2026-09-22

### Changed
- Jellyfinin istuntoluettelon luenta ja sen pyytäneen kellon nollaus ovat yksi kutsu, joten ne eivät enää voi erota toisistaan. Juuri sen muotoinen oli karkaavien pyyntöjen vika, joka löytyi version 1.5.0 rakentamisen aikana.
- Sen todistaminen vie sekunnin murto-osan seitsemän minuutin sijaan todellista palvelinta vastaan, joten se tarkistetaan aina, kun jokin muuttuu. Samat pyynnöt, sama historia, samat luvut näytöllä.

## [1.5.1] - 2026-09-22

### Fixed
- Transkoodaus tallennetaan kerran, ei kerran sekunnissa. Kun toisto alkaa transkoodaamalla ja palaa suoraksi toistoksi, Jellyfin jättää transkoodaustiedot istuntoon; jokaista lukemaa verrattiin arvoon, jonka FinStats oli juuri kirjoittanut yli, joten jokainen lukema näytti muutokselta ja Toiminta-sivu täyttyi ”Transkoodaus”-riveistä, joiden alla luki ”Suora toisto: …”.
- Jo kirjoitetut rivit siivotaan käynnistyksessä: kun peräkkäiset rivit sanovat saman asian, ensimmäinen säilytetään. Transkoodausta vaatinut toisto lasketaan yhä transkoodaukseksi.

## [1.5.0] - 2026-09-22

Istuntojen seuranta WebSocketilla

### Changed
- FinStats pitää yhden yhteyden auki Jellyfiniin käynnistyksestään lähtien, ja sitä kautta se kuulee toistoista. 1.4.0 toimitti sen asetuksena, oletuksena pois päältä, jotta se osoittaisi toimivuutensa todellisia palvelimia vastaan; nyt se on aina päällä. Samat toistot, tauot ja hypyt, saman koodin laskemina.
- Mitään ei toisteta: FinStats ei kysy Jellyfiniltä yhtään mitään. Ilta, jona kukaan ei katso, ei maksa yhtään pyyntöä.
- Jotain toistetaan: istuntoluettelo luetaan kerran sekunnissa. Siitä tauko, siirtymä tai raidan vaihto saa tarkkuutensa, ja sillä päätetään toisto, jonka asiakas katosi hyvästelemättä.
- Kaikki ladattu on ollut tauolla kolmella peräkkäisellä lukemalla: kysely lakkaa ja yhteys hoitaa asian; ennen tauolla oleva elokuva maksoi saman kysymyksen joka sekunti, noin 16 Mt tunnissa. Jatkamiseen tai siihen, että joku muu aloittaa jotain, vastataan noin sekunnissa, eikä tauolla oltua aikaa edelleenkään lasketa katseluajaksi.
- Jokainen lähtevä luenta pyytää pakattuja vastauksia ja purkaa ne (Jellyfin, Sonarr, Radarr ja Seerr pakkaavat kaikki pyydettäessä), mikä pienentää kahta suurinta luentaa yli puolella: istuntoluetteloa toiston aikana ja kirjaston luentaa. Paikannustietokanta ladataan yhä sellaisena kuin se on tallennettu.
- Asetusten Jellyfin-kortti kertoo, kumpi puolisko on käynnissä, ja reaaliaikainen piste seuraa sitä, välittääkö yhteys tietoa, eikä sitä, kummassa puoliskossa ollaan.
- Vain toinen kahdesta on käynnissä kerrallaan: kun FinStats kysyy, se pyytää Jellyfiniä lopettamaan lähettämisen, ja pyytää sitä aloittamaan uudelleen heti, kun viimeinen toisto päättyy. Yhteys pysyy auki koko ajan.
- Jellyfiniin, joka ei vastaa tilaukseen mitään, luotetaan niin kauan kuin se vastaa keep-alive-viesteihin: FinStats kertoo siitä kerran, lukee sillä välin istuntoluettelon itse ja pysyy tilattuna, kunnes ensimmäinen ilmoitus ratkaisee asian. ”Ei osaa ilmoittaa” ja ”oli käynnistymässä uudelleen, kun kysyimme” näyttävät samalta, joten hiljaisuus ei koskaan ole tuomio.
- Varajärjestely on ennallaan: yhteys, joka sulkeutuu, hiljenee tai ei osoittaudu puhumaan tätä, palaa ajastettuun kyselyyn heti seuraavalla kierroksella ja jatkaa uudelleenyhdistämistä asetusten **Tarkistusväli katselun aikana** ja **Tarkistusväli, kun mitään ei toisteta** mukaisin välein.

### Added
- Mitä keräin tekee, osoitteessa `GET /api/status`: missä neljästä tilasta se on (`idle_socket`, `playing_poll`, `paused_socket`, `fallback`), onko yhteys auki ja tilattu, kuinka usein se kysyy, montako istuntoluentaa lähti viimeisen minuutin aikana ja milloin se viimeksi muuttui. Vastataan muistista (ei pyyntöä, ei kyselyä), eikä siinä ole nimiä, nimikkeitä tai määriä.
- Nämä kentät tarkistetaan toisiaan vasten jokaisella kierroksella, ja pysyvä ristiriita kirjataan lokiin.
- Yksi rajoitin prosessin jokaisen istuntoluennan alla: enintään 2 sekunnissa ja 70 minuutissa, ja kaikki sen yli viivästetään, lasketaan ja kirjataan lokiin enintään kerran minuutissa. Tavallinen katselu pysyy selvästi sen alla.

### Fixed
- Reaaliaikaiset päivitykset todella korvaavat kyselyn. Päälle kytkettynä 1.4.0 pyysi yhä istuntoluetteloa viiden sekunnin välein ja avasi yhteyden uudelleen 20–40 sekunnin välein, koska se arvioi yhteyttä sen mukaan, kuinka äskettäin luettelo oli saapunut, ja Jellyfin lähettää sellaisen, kun jokin *muuttuu*, eikä mitään siinä välissä. Elossaolo tarkoittaa nyt sitä, vastaako Jellyfin ylipäätään, ja se tarkistetaan keep-alive-viestillä noin kahdesti minuutissa.
- Istuntoluettelo, joka saapui FinStatsin ollessa kierrosten välissä, heitettiin pois sillä oletuksella, että seuraava olisi puolentoista sekunnin päässä. Nyt se säilytetään ja käytetään.
- **Viimeksi tarkistettu** Asetuksissa ja Lähtevät yhteydet -kortti eivät enää vanhene yhteyden ollessa hiljaa: ne näyttävät Jellyfinin vastauksen keep-alive-viestiin.

### Removed
- Asetus **Anna Jellyfinin ilmoittaa, mitä toistetaan** kohdasta Asetukset → Keräys, ja `live_socket` asetusten API:sta.
- `socket_enabled` keräimen tilasta: ”pois kytkettyä” ei enää ole.
- `last_reconcile_at` keräimen tilasta, korvattuna kentällä `socket_live`.

## [1.4.1] - 2026-09-22

### Changed
- Seerr luetaan kokonaan vain, kun jokin on muuttunut: viiden minuutin välein FinStats pyytää yhden rivin, viimeksi muuttuneen pyynnön, ja kierros päättyy siihen, jos se tuntee sen jo. Se on noin kilotavu, kun viidenkymmenen rivin sivu on noin kuusikymmentä. Matkalla olevia pyyntöjä tarkastellaan vartin välein, kaikki luetellaan kerran päivässä, ja uusi pyyntö ilmestyy yhä viiden minuutin sisällä.
- Sonarrin ja Radarrin jonot luetaan viiden sekunnin välein, kun sivu näyttää niitä, minuutin välein, kun jonossa on jotain, ja viiden minuutin välein, kun se on tyhjä, sen sijaan että ne luettaisiin minuutin välein ympäri vuorokauden. Sivun avaaminen, palvelun yhdistäminen tai uusi pyyntö Seerrissä päivittää ne heti.

## [1.4.0] - 2026-09-21

Valinnaiset WebSocket-istunnot ja kaikki, mihin FinStats voi yhdistää

### Added
- **Anna Jellyfinin ilmoittaa, mitä toistetaan** kohdassa Asetukset → Keräys, pois päältä, kunnes kytket sen päälle: yksi yhteys kuljettaa istuntoluettelon, ja noin 17 000 päivittäisestä pyynnöstä tulee lähes nolla. Toistot, tauot, hypyt ja ryhmät lasketaan täsmälleen samalla koodilla kuin ennenkin. Kun se on käytössä, FinStats tekee yhä yhden tavallisen pyynnön minuutissa *kun jotain toistetaan*, eikä yhtään, kun mitään ei toisteta. Yhteys, joka sulkeutuu, hiljenee viideksitoista sekunniksi tai ei osoittaudu puhumaan sitä, palaa ajastimeen seuraavalla kierroksella ja jatkaa uudelleenyhdistämistä.
- **Lähtevät yhteydet**, Asetusten kortti Jellyfinin ylläpitäjille: jokainen kohde, johon FinStats voi yhdistää (Jellyfinisi, ”mikä on IP-osoitteeni” -palvelu, DB-IP:n tietokanta sekä jokainen Sonarr, Radarr tai Seerr), sekä mihin se on, onko se kytketty päälle ja milloin se viimeksi vastasi. Koottu siitä, mitä FinStats jo tietää, ja se näyttää vain osoitteet: ei koskaan avainta, ei koskaan peruspolkua.
- **Hae nyt** kohdassa Asetukset → Kotiverkko sitä päivää varten, jona julkinen osoitteesi vaihtuu.

### Changed
- ”Mikä on IP-osoitteeni” -haku ajetaan kerran, kun FinStats ensimmäisen kerran tarvitsee osoitteen, eikä vartin välein.
- Asetusten Jellyfin-kortti kertoo, miten keräin saa tietonsa, ja kun yhteys on kytketty päälle mutta ei välitä tietoa, miksi ei.

## [1.3.0] - 2026-09-20

Sonarr, Radarr ja Seerr yhdellä sivulla

### Added
- **Yhteydet** Asetuksissa Jellyfinin ylläpitäjille: **Sonarr**, **Radarr** ja **Seerr** (tai Jellyseerr/Overseerr), useita kutakin lajia, kukin testattuna ennen tallennusta. Latausohjelmasi ei tarvitse omia asetuksia, koska Sonarr ja Radarr keskustelevat jo sen kanssa ja FinStats lukee, mitä ne tietävät.
- **Pyynnöt**: kuka pyysi mitä, kuinka kauan sen saapuminen kesti ja katsoiko hän sitä koskaan. Ruudut odottavista nimikkeistä ja tyypillisestä odotusajasta, kaavio sen muuttumisesta kuukausittain sekä luettelot siitä, mikä saapui viikkoja sitten eikä sitä ole koskaan toistettu, ja kuka pyytää eniten.
- **Tulevat**: Sonarrin ja Radarrin uudet jaksot ja elokuvajulkaisut päiväkohtaisena asialistana, merkittyinä sen mukaan, katsotko *sinä* sitä sarjaa, ja niille, jotka saavat nähdä kaikki, kuka muu katsoo. Sarja lasketaan katsotuksi, kun joku on toistanut sen jakson viimeisen neljän kuukauden aikana. ”Tulossa”-rivi yleiskatsauksessa, yksi kuhunkin profiiliin sen henkilön katsomista sarjoista, ja nimikkeen omalla sivulla sen seuraava julkaisu.
- **Lataukset**: jokaisen Sonarrin ja Radarrin reaaliaikainen jono (mikä se on, kuinka pitkällä, kuinka nopeasti, mikä meni pieleen tuonnissa) ja sen pyytänyt henkilö vieressä. Kausipaketti on yksi rivi, olipa siinä kuinka monta jaksoa tahansa, ja huomiota vaativat ovat ylimpänä. Sen alla, mitä saapui viikon, kuukauden tai vuoden aikana indeksoijan, laadun ja latausohjelman mukaan, ja mitkä lataukset epäonnistuivat. Yleiskatsaus näyttää saman luettelon lyhennettynä.
- **Näe lataukset**, uusi oikeus. Ilman sitä ihmiset näkevät yhä, kuinka pitkällä heidän *oma* pyyntönsä on ja kauanko sitä on jäljellä, eikä mitään muuta: ei julkaisujen nimiä, ei nopeuksia, ei muita latauksia. Muiden pyynnöt vaativat oikeuden *Näe kaikkien toiminta*.

### Changed
- FinStats lähettää näille palveluille `GET`-pyyntöjä eikä mitään muuta: siinä ei ole koodia, joka voisi hyväksyä pyynnön, käynnistää haun tai lisätä, keskeyttää tai poistaa latauksen.
- Niiden API-avaimet tallennetaan FinStatsin omaan tietokantaan, eikä niitä koskaan lähetetä takaisin selaimelle, kirjoiteta lokiin tai sisällytetä varmuuskopioon. Uudelleenohjauksia ei seurata, joten avain ei voi kulkeutua minnekään, mitä et syöttänyt, ja varmenteet tarkistetaan, ellet kytke sitä pois yksittäiseltä yhteydeltä.

## [1.2.2] - 2026-09-20

### Added
- Ääni- ja tekstityskielet jokaisella elokuvalla ja jaksolla (”Ääni: japani · englanti”).
- Kuinka pitkälle jälkiäänitys ulottuu: sarja tai kausi kertoo, monessako sen jaksossa kukin kieli on (”englanti: 13/26 jaksoa”), ja jaksoluettelossa on **Ääni**-sarake.
- **Äänen kielet** ja **Tekstitysten kielet** kirjastosivuilla: montako tiedostoa kullakin voi toistaa.
- FinStats lukee kirjaston uudelleen kerran tämän päivityksen jälkeen poimiakseen kielet. Se ei koskaan sano ”jälkiäänitetty”, koska Jellyfin ei kerro nimikkeen alkuperäistä kieltä, ja raita ilman kielitunnistetta luetellaan ”Tuntemattomana”.

## [1.2.1] - 2026-09-20

### Changed
- README:n kuvat on otettu uudelleen tästä versiosta, ja mukana on uusi kuva **Tietoturva**-sivusta. Sovelluksessa ei muutu mikään.

## [1.2.0] - 2026-09-20

Mistä katsotaan, kartalla

### Added
- **Tietoturva-sivu** niille, jotka saavat nähdä verkkotiedot ja kaikkien toiminnan: maailmankartta, jossa on piste jokaisesta paikasta, josta käyttäjäsi katsovat, kooltaan sen mukainen, kuinka paljon siellä tapahtuu. Koti on vihreä, muut paikat violetteja, käynnissä olevat toistot sykkivät ja ulkopuolelta tulleet epäonnistuneet kirjautumiset ovat punaisia. Vedä siirtääksesi, zoomaa painikkeilla, näppäimillä tai Ctrl + rullalla, ja päällekkäiset pisteet yhdistyvät, kunnes zoomaat lähemmäs. Sen alla jokainen paikka ihmisineen, toistoineen, katseluaikoineen ja kirjautumisineen sekä maat toistojen mukaan.
- **Mahdoton matkustus**: hälytys, kun sama tili nähdään kahdessa paikassa, joita mikään lento ei yhdistä, tai kahdessa kaukaisessa paikassa yhtä aikaa; hälytys kertoo molemmat havainnot, etäisyyden, niiden välisen ajan ja nopeuden, jota matka olisi vaatinut. **Näytä kartalla** piirtää matkan.
- **Uusi maa**: ensimmäinen kerta, kun joku toistaa tai kirjautuu maasta, jossa häntä ei ole ennen nähty.
- Ratkaise hälytys huomautuksella tai mykistä paikkapari kyseiseltä henkilöltä VPN:n tai puhelimen takia, jonka operaattori sijaitsee pääkaupungissa. Vanhasta historiasta, tuonnin jälkeen tai uudesta tietokannasta löytyneet hälytykset kirjataan ratkaistuiksi sen sijaan, että ne tulvisivat luetteloon. Hälytykset ja niistä tekemäsi päätökset kuuluvat varmuuskopioihin.
- Paikat tulevat paikallisesti luettavasta kaupunkitietokannasta: mitään osoitetta ei koskaan lähetetä minnekään, ja kartta piirretään FinStatsin mukana tulevista ääriviivoista eikä karttapalvelusta. Lataa DB-IP:n ilmainen tietokanta Tietoturva-sivulta tai kohdasta **Asetukset → Tietoturva**, anna FinStatsin päivittää se kuukausittain (pois päältä, kunnes kytket sen päälle) tai pudota oma `.mmdb`-tiedostosi (DB-IP, MaxMind GeoLite2-City) kansioon `data/geoip/`. `FINSTATS_GEOIP_DB` nimeää tiedoston muualla.
- **Asetukset → Tietoturva**: käytössä oleva tietokanta, kuukausittainen päivitys ja kuinka nopeasti (900 km/h) ja kuinka kaukana toisistaan (500 km) kahden havainnon on oltava, jotta ne lasketaan mahdottomaksi matkustukseksi. Kaupunkitietokannat menevät usein muutaman sadan kilometrin verran ohi, ja sitä varten etäisyys on.

## [1.1.3] - 2026-09-20

### Added
- Ctrl+Space hakee myös kaikkien kirjastosi nimikkeiden näyttelijöistä ja tekijöistä kohdassa **Näyttelijät ja tekijät**, kuvineen ja tietoineen siitä, näyttelevätkö he vai ohjaavatko, ja monessako nimikkeessäsi he ovat; valinta avaa heidän sivunsa. Yhtä anteeksiantava kuin nimikehaku (sanat missä järjestyksessä tahansa, tarkkeet ohitetaan, lyöntivirhe sallitaan), ja kun useampi osuu yhtä hyvin, ensimmäisenä on se, joka on useammassa nimikkeessäsi.

## [1.1.2] - 2026-09-20

### Added
- **Viimeksi lisätyt** yleiskatsauksessa, Toiminta-kaavion yläpuolella: 30 uusinta saapunutta julisterivinä, jota voi vierittää sivuttain, tietokoneella nuolilla. Kukin kertoo, milloin se saapui ja mikä se on (elokuva, albumi tai ”Kausi 4 · 3 jaksoa”), ja yksittäinen uusi jakso kertoo, mikä, ja käyttää kauden omaa julistetta, jos sellainen on.
- **Shift + hiiren rulla** ja nuolinäppäimet siirtävät yhden julisteen verran, Page Up ja Page Down näytöllisen, Home ja End vievät kumpaan tahansa päähän, ja pyyhkäisy toimii. Pelkkä rulla vierittää yhä sivua.
- Saman sarjan uudet jaksot, jotka saapuvat samana päivänä, jakavat yhden merkinnän, ja kerralla lisätty kokonainen sarja on yksi merkintä (”3 kautta · 60 jaksoa”), joten yksi iso tuonti ei työnnä kaikkea muuta pois luettelosta.
- Rivi ei välitä yleiskatsauksen aikavälistä ja henkilöstä, koska se koskee kirjastoa eikä toistoja, ja piilottaa itsensä, kun kirjasto on tyhjä.

## [1.1.1] - 2026-09-20

### Added
- Sivut avautuvat heti: hetki FinStatsin latautumisen jälkeen se hakee hiljaa sen, minkä todennäköisesti avaat seuraavaksi (yleiskatsauksen, profiilisi ja aikajanasi, Toiminnan, Käyttäjät, Kirjastot ja jokaisen kirjaston, Toiston, palvelinsivut, vuosikatsauksen ja aktiivisimmat henkilöt), joten sivu näkyy avattaessa heti ja päivittyy silti taustalla.
- Elokuvat, sarjat ja näyttelijät haetaan vasta, kun aiot avata ne, koska niitä on tuhansia: osoittimen pitäminen linkin päällä, sen koskettaminen tai siihen siirtyminen näppäimistöllä hakee kyseisen sivun.
- Se odottaa, kunnes avaamasi sivu on latautunut, käsittelee yhden sivun kerrallaan selaimen ollessa joutilaana, pitää taukoa taustavälilehdellä, ei tee mitään datansäästötilassa tai hyvin hitaalla yhteydellä, ja haettu unohtuu, kun kirjaudut ulos.
- Pieni **Repo**-linkki GitHub-merkillä tilapalkissa version vieressä.

### Fixed
- **Aikajana** unohti, mitkä kirjastot oli kytketty pois, kun sivu ladattiin uudelleen tai avattiin kirjanmerkistä, vaikka valinta oli osoitteessa.

## [1.1.0] - 2026-09-20

Aikajana kaikesta, mitä olet katsonut

### Added
- **Aikajana**, **Yleiskatsauksen** vieressä jokaisessa profiilissa: katselusi yhtenä polkuna tästä päivästä taaksepäin ensimmäiseen FinStatsin tuntemaan toistoon, ja illan jaksot koottuina yhdeksi pysäkiksi. Jokainen pysäkki on juliste siitä, mitä katsottiin ja milloin (”Kausi 2 · Jaksot 3–6”, elokuva ja se, katsottiinko se useammassa istunnossa, tai albumi ja montako kappaletta), ja se avaa nimikkeen.
- Polku mukautuu ikkunan leveyteen: kolme pysäkkiä riville leveällä näytöllä, kaksi kapeammalla, joka toinen rivi kulkee takaperin ja mutka yhdistää sen seuraavaan, puhelimella yhdeksi suoraksi viivaksi asti. Kunkin kuukauden ensimmäinen pysäkki kertoo kuukauden.
- Vanhemmat pysäkit latautuvat itsestään vierittäessä, aina sinne asti, mistä historia alkaa.
- Polun yläpuolella olevat **Kirjastot**-kytkimet jättävät pois sen, mitä et halua nähdä, kuten musiikin, ja valinta on osa osoitetta.
- Yhä toistossa oleva näyttää tekstin **Toistossa nyt**.
- Näet oman aikajanasi; muiden aikajanat vaativat oikeuden ”Näe kaikkien toiminta”.

## [1.0.5] - 2026-09-20

### Added
- Suojaus vanhempaan versioon palaamista vastaan: tietokanta muistaa uusimman FinStats-version, joka on avannut sen, ja vanhempi versio kieltäytyy käynnistymästä sen kanssa sen sijaan, että käsittelisi hiljaa tietoja, joita se ei täysin ymmärrä. Viesti kertoo, mitä tehdä. Versiot 1.0.4:ään asti julkaistiin ennen kuin tätä tarkistusta oli.

## [1.0.4] - 2026-09-20

### Fixed
- Toiston tiedoissa pitkän **Laitetunnisteen** vieressä oleva kopiointipainike oli pudonnut omalle rivilleen.
- Toisto-sivulla **Mitkä sovellukset transkoodaavat** -kortin viimeinen sarake katkesi (”Transk…”); kahden sanan otsikot rivittyvät nyt.

### Changed
- README:n kuvakaappaukset ovat taas ajan tasalla; niissä näkyi vielä versio 0.5.0.

## [1.0.3] - 2026-09-20

### Added
- Jakson sivu näyttää jakson oman kuvan, saman, jonka Jellyfin näyttää jaksoluettelossaan, sarjan julisteen sijaan. Jakso, jolle Jellyfinillä ei ole kuvaa, pitää julisteen.

## [1.0.2] - 2026-09-20

### Fixed
- **Muutoslokissa** v0.9- ja v0.10-ryhmillä ei ollut otsikkoa versionsa vieressä. Ryhmän otsikko on nyt aina ensimmäinen lause siitä, mistä sen ensimmäisessä julkaisussa oli kyse, joten pitkää johdantoa ei katkaista kesken lauseen.

## [1.0.1] - 2026-09-20

### Fixed
- Julkaistu levykuva ei käynnistynyt uudella koneella: ”unable to open database file: /data/finstats.db”, puolen minuutin jumittumisen jälkeen. Docker luo puuttuvan `data`-kansion root-käyttäjänä, ja FinStats toimii tavallisena käyttäjänä. Säiliö ottaa nyt kansion omakseen ja siirtyy sitten kyseiseksi käyttäjäksi, joten `docker run` toimii ensimmäisellä yrityksellä. Jos tiedostot omistaa joku muu kuin käyttäjä 1000, aseta `PUID` ja `PGID`; `--user` toimii kuten ennenkin.
- Datakansio, johon todella ei voi kirjoittaa, ilmoitetaan heti sen korjaavan komennon kera, eikä 30 sekuntia myöhemmin tietokantavirheenä.

## [1.0.0] - 2026-09-20

Valmis kaikille

### Added
- Valmis levykuva, nyt osoitteessa `ghcr.io/finstats/finstats`, 64-bittisille Intel/AMD- ja ARM-koneille (Raspberry Pi 4 tai 5 käy): `:latest` on uusin julkaisu, versiotunniste pysyy paikallaan, ja `:edge` on kehitysversio.
- Jokainen julkaisu ilmestyy projektin GitHub-julkaisusivulle näiden muutoslokimerkintöjen kera.

### Changed
- Versionumerot tarkoittavat nyt sitä, mitä ne sanovat: 1.x-päivitykset eivät riko tietojasi, varmuuskopioitasi tai asetuksiasi.
- Docker Compose -tiedosto ja kaikki ohjeet käyttävät julkaistua levykuvaa. Itse rakennetussa `finstats:latest`-kuvassa tarvitsee vaihtaa vain levykuvan nimi; `data`-kansio siirtyy koskemattomana.

## [0.10.2] - 2026-09-20

### Fixed
- Puhelimella Asetukset ja sarjan sivu saattoivat vieriä sivuttain: vain ruudunlukijoille tarkoitettu nimiö karkasi taulukostaan, ja jaksoluettelot ja näyttelijärivi tekivät samoin. Nyt ne vierivät omassa kehyksessään.
- Vuosikatsaus ei enää vieri sivuttain, kun luvun taustasana on sivua leveämpi.
- Pitkä ”katsoo laitteella…”-rivi kohdan **Nyt toistossa** alla päättyy kolmeen pisteeseen sen sijaan, että se katkeaisi kesken kirjaimen.
- Jokaisen sarjan edessä oleva juliste profiilissa oli ruudunlukijoille toinen, nimetön linkki; nyt se on piilotettu niiltä, ja sen vieressä oleva nimi on linkki.

## [0.10.1] - 2026-09-20

### Fixed
- Varmuuskopion lataaminen epäonnistui selaimessa (Firefoxissa ”the source file could not be read”) ilman, että sivulla kerrottiin syytä: FinStats pakkasi jo valmiiksi pakatun varmuuskopion toiseen kertaan, ja kahdesti pakattu siirto katkesi ennen loppua. Varmuuskopiot lähetetään sellaisinaan ja koko etukäteen ilmoitettuna, joten selain voi näyttää todellisen edistymisen.

## [0.10.0] - 2026-09-20

Varmuuskopiot ja siirtyminen uuteen asennukseen

### Added
- Varmuuskopio joka viikko, viisi uusinta säilytetään: jokainen toisto tauko- ja hyppyaikajanoineen, nähdyksi merkinnät, oikeudet, kotiosoitteet, palvelimen loki ja asetuksesi yhdessä pienessä tiedostossa, muutama sata kilotavua tuhansilla toistoilla. **Asetukset → Varmuuskopiot** luettelee ne kukin **Lataa**-painikkeineen, tekee varmuuskopion pyynnöstä ja määrää, kuinka usein ja kuinka monta. Ne ovat datakansiosi `backups`-kansiossa. Varmuuskopiossa ei ole Jellyfinin API-avainta eikä kirjautumisistuntoa, mutta se on täydellinen katseluhistoria IP-osoitteineen, joten säilytä se yksityisessä paikassa.
- Palautus yhdistää eikä korvaa: jo olemassa olevat toistot ohitetaan, joten sen voi tehdä turvallisesti kahdesti tai asennukseen, joka on ollut käynnissä jo jonkin aikaa. Poista valinta ”Palauta myös asetukset ja oikeudet”, niin palautetaan pelkkä historia. Varmuuskopiot toimivat versioiden välillä molempiin suuntiin. Päätteestä: `finstats backup` ja `finstats restore <file>`.

### Changed
- Keräys tarkistaa joka sekunti, kun joku katsoo, ja 5 sekunnin välein, kun kukaan ei katso (ennen koko ajan 5 sekunnin välein), joten tauot, hypyt ja raidan vaihdot tallennetaan sekunnin tarkkuudella. Molemmat ovat kohdassa **Asetukset → Keräys**; muutettu vanha väli on asetettava uudelleen.

### Fixed
- Toiston tiedoissa IP-osoite katkesi kahdelle riville kesken osoitteen. Paikallinen/Etä-merkintä siirtyy alle, kun tilaa ei ole.

## [0.9.1] - 2026-09-20

### Fixed
- Kotona olevat ihmiset näytettiin etäkäyttäjinä. Oman verkkosi laite, joka tavoittaa Jellyfinin sen julkisen nimen kautta, esimerkiksi käänteisen välityspalvelimen tai verkkotunnuksen kautta, saapuu talouden julkisella IP-osoitteella, ja vain yksityiset osoitteet laskettiin paikallisiksi. FinStats selvittää nyt julkisen osoitteen ja laskee nuo toistot paikallisiksi koko historian ajalta, ja se muistaa aiemmat osoitteet.

### Added
- **Asetukset → Kotiverkko**: osoitteet, joita FinStats pitää kotina, omien lisääminen (aiempi osoite, kakkoskoti, VPN-yhteyden ulostulo) tai haun kytkeminen pois. Haku kysyy tavalliselta ”mikä on IP-osoitteeni” -palvelulta vartin välein ja kokeilee useaa (Amazon, Cloudflare ja muita), koska mainoksia estävä DNS, kuten Pi-hole, estää niitä usein, ja yksi niistä ei tarvitse DNS:ää lainkaan. Pyyntö ei sisällä mitään sinusta tai palvelimestasi, ja kun kytkin on pois päältä, FinStats ei puhu kenellekään muulle kuin Jellyfinille. `FINSTATS_PUBLIC_IP_URL` asettaa oman palvelun.

## [0.9.0] - 2026-09-20

Jokainen taulukko haluamassasi järjestyksessä

### Added
- Jokainen taulukko lajittuu: napsauta otsikkoa lajitellaksesi sen mukaan, toisen kerran kääntääksesi järjestyksen ja kolmannen kerran palataksesi alkuperäiseen järjestykseen. Solujen merkitys ymmärretään, joten ”3 pv 2 t” lajitellaan aikana ja ”1,4 Gt” kokona, ja tyhjät solut tulevat viimeisinä. Se kattaa käyttäjäluettelon, katsojat, jaksot, laitteet, lisäosat, ajastetut tehtävät, jokaisen kaavion taulukkonäkymän, oikeusmatriisin ja palkkiluettelot (koodekit, resoluutiot, sovellukset ja muut), jotka saavat oman kapean otsikkonsa.
- Toiminta ja Palvelimen loki lajittelevat kaikkien sivujensa yli eivätkä vain näytöllä olevia rivejä, ja järjestys säilyy osoitteessa, joten se kestää uudelleenlatauksen tai jaetun linkin.
- Vähintään kymmenen rivin taulukot saavat yläpuolelleen pienen suodatinkentän.

## [0.8.2] - 2026-09-20

### Changed
- Vuosikatsauksen vuodenvalitsin on sama kytkin kuin jokainen muu aikavälin valitsin FinStatsissa (7 pv · 30 pv · 90 pv …).

## [0.8.1] - 2026-09-19

### Added
- Sivut näyttelijöille ja ohjaajille, vuosikatsauksen kohdasta ”Katsotuimmat henkilöt” tai jokaisen elokuvan, sarjan tai jakson uudelta **Näyttelijät ja tekijät** -riviltä: kaikki, missä he ovat palvelimellasi, kuinka paljon siitä on katsottu ja kuka katsoi, ja mikä odottaa vielä. Se noudattaa valitsemaasi aikaväliä, ne, jotka saavat nähdä vain omat tilastonsa, näkevät siellä vain oman katselunsa, ja Esc vie takaisin.

## [0.8.0] - 2026-09-19

Vuosikatsaus sai uuden ilmeen ja enemmän kerrottavaa

### Added
- **Katsotuimmat henkilöt**: näyttelijät, joiden seurassa vietit eniten aikaa, ja ohjaajat, joiden käsialaa katsomasi oli, muotokuvien rivinä. Näyttelijät ja tekijät luetaan kirjaston mukana ja näkyvät tämän päivityksen jälkeisen ensimmäisen kirjaston luennan jälkeen.
- **Vuotesi päivinä**: vuoden jokainen päivä yhtenä ruutuna, sitä kirkkaampana, mitä enemmän katsoit, ja sen yläpuolella pisin putkesi ja suurin päiväsi. Osoita päivää tai kulje kalenterissa nuolinäppäimillä nähdäksesi, mitä siihen mahtui.
- **Lukuina**: toistot, katseluaika, eri nimikkeet, katselupäivät, pisin putki ja kuinka suuri osa oli **uudelleenkatselua**, sekä miten vuosi jakautuu jaksojen, elokuvien ja musiikin kesken.
- Lajityyppiluku kertoo, montako lajityyppiä kosketit ja kuinka suuren osuuden suosituin vei.

### Changed
- Uusi ulkoasu: se avautuu otsikolla vuotesi ”uudelleen toistettuna”, pääotsikko sen täyttäneiden julisteiden vieressä, vuoden aaltomuodon päällä, jossa on yksi palkki viikkoa kohden ja äänekkäin korostettuna, ja vuosi valitaan juuri sen yläpuolelta. Jokaisella luvulla on pääotsikko, jonka avainsana on violetti, lause, joka kantaa luvut, ja luvun sana suurena ja himmeänä sen takana. Se päättyy elokuvien tapaan lopputeksteihin: pääosissa, ohjannut, esitetty, kesto.
- Tunnit, viikonpäivät ja kuukaudet jakavat yhden **Katselutottumukset**-luvun, jossa niiden välillä vaihdetaan kytkimellä, ja pääotsikko seuraa sitä, mitä katsot (”Voiton vei perjantai”).

### Fixed
- **Nyt toistossa** jäi yleiskatsauksessa tyhjäksi, kun kukaan ei katsonut, joten otsikko näytti kuuluvan sen alla oleviin suodattimiin. Sen ”Mitään ei toisteta juuri nyt” -laatikko on palannut.

## [0.7.10] - 2026-09-19

### Added
- Kirjastojen kuvat: jokainen kirjastokortti ja kirjaston oma sivu näyttävät kuvan, jota Jellyfin käyttää kirjastolle, FinStatsin kuvavälityspalvelimen kautta julisteiden tapaan. Kirjasto, jolla kuvaa ei ole, pitää kuvakkeensa.

## [0.7.9] - 2026-09-19

### Changed
- Lajityypit-kortti avautuu taas **luettelona**, ja tutka on kytkimen toinen vaihtoehto. Aiempi valinta säilyy.

## [0.7.8] - 2026-09-19

### Added
- Tutkanäkymä Lajityypit-kortille yleiskatsauksessa ja profiileissa: yksi säde lajityyppiä kohden, vaihdettavissa kortin kulmasta näkymien **Tutka** ja **Luettelo** välillä, ja valinta muistetaan. Pisteen osoittaminen tai nuolinäppäinten käyttö näyttää lajityypin katseluajan ja osuuden.

## [0.7.7] - 2026-09-19

### Fixed
- Kokoelmaan kuuluvat elokuvat näytettiin tilassa ”Ei enää kirjastossa”. Kun *ryhmittele elokuvat kokoelmiksi* on päällä, Jellyfin antaa kokoelman sen sisältämien elokuvien sijaan, joten FinStats ei koskaan nähnyt niitä; yhdellä palvelimella se piilotti 556 kohdetta 1 238:sta Elokuvat-kirjastossa. FinStats pyytää nyt itse elokuvia, ja tämän päivityksen käynnistämä luenta tuo ne takaisin ja liittää niiden toistot uudelleen.

## [0.7.6] - 2026-09-19

### Changed
- Profiileissa **Sarjat**-kortti siirtyi alemmas juuri Lajityyppien yläpuolelle, joten pitkä sarjaluettelo ei enää työnnä katseluaikaa, toimintakaaviota ja suosituimpia nimikkeitä näkyvistä. Päiväputket pysyvät ylhäällä, ja avaamasi sarja pysyy auki, kun vaihdat aikaväliä.

## [0.7.5] - 2026-09-19

### Changed
- Haku täsmää sana sanalta, missä järjestyksessä tahansa ja mitä tahansa välissä, eikä vaadi tarkkaa ilmausta, joten ”alya hides” löytää nimikkeen *Alya Sometimes Hides Her Feelings in Russian*. Kirjainkoolla, tarkkeilla ja välimerkeillä ei ole väliä, alun ”The” ohitetaan, ja lyöntivirhe annetaan anteeksi (”mentalsit” löytää nimikkeen *The Mentalist*); kolmen tai harvemman kirjaimen sanat on yhä kirjoitettava oikein. Paremmat osumat ovat ensin, ja musiikki löytyy myös esittäjän mukaan.
- Toiminnan ja Palvelimen lokin hakukentät täsmäävät sana sanalta kaikista sarakkeistaan: ”alya opera” löytää kyseisen sarjan toistot Operassa.

## [0.7.4] - 2026-09-19

### Changed
- Muutosloki on ryhmitelty sarjoittain: jokainen `0.x`-sarja on yksi laskos, joka sisältää sen julkaisut, sekä sen, mistä sarjassa oli kyse, montako julkaisua siinä on ja milloin. Vain uusin on aluksi auki, ja suljettu laskos näyttää silti, mitä versiota käytät.

## [0.7.3] - 2026-09-19

### Fixed
- Nyt toistossa -kello takelteli muutaman sekunnin välein: 5 sekunnin päivitys piirsi myös sen uudelleen, joten aina kun palvelin oli hitusen edellä, luku liikkui liian aikaisin ja seisoi sitten paikallaan sekunnin. Nyt sitä liikuttaa vain kerran sekunnissa tapahtuva tikitys, täsmälleen yhden sekunnin verran (mitattu 999–1001 ms välein), ja päivitys korjaa yhä tauon, hypyn tai todellisen ajautumisen jälkeen, tikitykseen osuen.

## [0.7.2] - 2026-09-19

### Added
- Yhteiskatselu reaaliajassa: jokainen Nyt toistossa -kortti kertoo ”Yhdessä: maria”, kun ihmiset katsovat samaa asiaa yhdessä; ennen ryhmä tunnistettiin vasta toistojen päätyttyä. Reaaliajassa ihmiset lasketaan yhdessä katsoviksi, kun he ovat samassa nimikkeessä ja joko aloittivat ryhmäikkunan sisällä tai ovat lähes samassa kohdassa, mikä toimii silloinkin, kun FinStats käynnistettiin uudelleen kesken toiston.

## [0.7.1] - 2026-09-19

### Changed
- Nyt toistossa laskee jokaisen sekunnin: kello tikittää ja palkki liukuu välissä sen sijaan, että hyppäisi aina, kun palvelimelta kysytään. Palvelimelta kysytään yhä 5 sekunnin välein, ja se pysyy totuuden lähteenä: näyttö synkronoituu uudelleen tauon, hypyn tai palvelimen ollessa edellä, mutta ei kohdasta, joka on vain muutaman sekunnin vanha, koska asiakkaat raportoivat vain noin kymmenen sekunnin välein. Tauolla olevat toistot seisovat paikallaan.

## [0.7.0] - 2026-09-19

Kuka katsoo yhdessä

### Added
- Yhteiskatselu: kun eri ihmiset aloittavat saman nimikkeen samaan aikaan, FinStats laskee sen yhdessä katsomiseksi. Yleiskatsauksen **Katsottu yhdessä** -kortti näyttää ryhmät, niiden yhteiset nimikkeet ja yhteiset tunnit, jokainen profiili näyttää, kenen kanssa henkilö katsoo eniten, yhteisissä toistoissa on Toiminta-sivulla pieni ihmismerkki, ja toiston tiedot kertovat, kenen kanssa sitä katsottiin. Se toimii koko historiassasi, myös tuoduissa toistoissa.
- Asetus sille, kuinka lähellä toisiaan aloitusten on oltava, oletuksena 60 sekuntia: noin kolmannes aidoista ryhmäistunnoista alkaa 6–60 sekunnin välein. Ihmisten on myös katsottava rinnakkain pari minuuttia, joten kaksi samaa jaksoa sattumalta avaavaa ihmistä ei ole ryhmä.

### Changed
- Haku avautuu näppäinyhdistelmällä **Ctrl + Space** eikä enää Ctrl/⌘ + K.

## [0.6.1] - 2026-09-19

### Added
- Ylläpitäjät voivat avata toisen henkilön vuosikatsauksen Vuosikatsaus-sivun yläosasta tai hänen profiilinsa **Avaa vuosikatsaus** -painikkeella. Se kertoo silloin hänestä nimeltä eikä ”sinusta”.

### Changed
- Kaikki muut näkevät yhä vain oman vuosikatsauksensa oikeuksistaan riippumatta, eikä koko palvelimen vuosikatsausta vieläkään ole.

## [0.6.0] - 2026-09-19

Profiilisi ja missä olet jokaisessa sarjassa

### Added
- Sarjojen edistyminen profiileissa: jokainen sarja, johon olet tarttunut, palkkina, jossa on yksi lohko jaksoa kohden (nähty, aloitettu tai ei vielä), avattavissa kausi kerrallaan, järjestettynä sen mukaan, kuinka lähellä olet loppua, ja erilliset näkymät loppuun katsotuille sarjoille ja kaikelle.
- **Merkitse nähdyksi** omassa profiilissasi, jaksoittain, kausittain tai sarjoittain, sitä varten, mitä katsoit, kun mikään ei tallentanut. Myös Jellyfinin omia toistettu-merkintöjä käytetään. Merkinnät pysyvät FinStatsissa eivätkä koskaan muuta mitään Jellyfinissä; tallennetun toiston merkintää ei voi poistaa, omat merkintäsi voi.
- Päiväputket profiileissa: pisin peräkkäisten toistopäivien sarjasi, nykyinen putkesi ja kuinka monena päivänä olet katsonut jotain.
- **Oma profiili** sivupalkissa.
- Toiminta näyttää edistymispalkin alla, mihin toisto pysähtyi (esimerkiksi `44:15 / 45:00`), ja päivämäärän ja kellonajan ”3 t sitten” -merkinnän alla. Jellystatista tuoduilla toistoilla ei ole pysähtymiskohtaa, koska Jellystat ei koskaan tallentanut sitä.

### Changed
- Puuttuvia tai vielä esittämättömiä jaksoja ei enää käsitellä osana kirjastoasi. Jellyfin luettelee ne paikkamerkkeinä ilman tiedostoa, ja ne laskettiin ennen mukaan jaksojen kokonaismääriin.

## [0.5.1] - 2026-09-19

### Added
- Logo: kolmeksi kaaviopalkiksi viipaloitu toistopainike sivupalkissa ja kirjautumisnäkymissä, terävänä SVG:nä, sekä monikokoinen `favicon.ico` selaimille ja kirjanmerkkipalkeille, jotka sellaisen haluavat, ja aloitusnäytön kuvake puhelimille ja tableteille.

## [0.5.0] - 2026-09-19

Sinä päätät, kuka näkee mitä

### Added
- Oikeudet kohdassa Asetukset → Käyttöoikeudet, kaikille kerralla tai henkilö kerrallaan: **Näe kaikkien toiminta** (muiden tilastot ja historia, Käyttäjät-sivu, jokainen käynnissä oleva toisto), **Näe verkkotiedot** (IP-osoitteet, laitetunnisteet, paikallinen vai etä), **Näe palvelin** (Palvelin-sivu, palvelimen loki, epäonnistuneet kirjautumiset, tiedostopolut) ja **Hallinnoi FinStatsia** (asetukset, tehtävät, tuonti, toistojen poistaminen).
- Kirjautumisoikeus henkilökohtaisesti, jotta muutaman ihmisen voi päästää sisään avaamatta FinStatsia jokaiselle Jellyfin-käyttäjälle.
- Muutokset tulevat voimaan heti, myös pääsyn poistaminen, ja palvelin valvoo kaikkea sen sijaan, että painikkeita vain piilotettaisiin.

### Changed
- Jellyfinin ylläpitäjillä on yhä kaikki, ja vain he voivat muuttaa oikeuksia; FinStatsia hallinnoimaan oikeutettu ei voi myöntää mitään, ei itselleen eikä kenellekään muulle.
- Vuosikatsaus pysyy henkilökohtaisena, myönnettiinpä mitä tahansa.
- Olemassa olevissa asennuksissa mikään ei muutu, ennen kuin myönnät jotain.

## [0.4.5] - 2026-09-19

### Changed
- FinStats on lisensoitu **GNU General Public License v3.0** -lisenssillä. Aiempaa versiota ei koskaan julkaistu, joten mitään ei julkaistu muilla ehdoilla. Mukana tulevien fonttien lisenssit toimitetaan niiden rinnalla.

## [0.4.4] - 2026-09-19

### Added
- Esc vie askeleen taaksepäin: avatussa kirjastossa, käyttäjässä tai nimikkeessä se palauttaa sinut sinne, mistä tulit, luettelo täsmälleen sellaisena kuin jätit sen. Kaikki auki oleva sulkeutuu ensin (toiston tiedot, haku, pudotusvalikko, mobiilivalikko), ja Esc jätetään rauhaan, kun kirjoitat tai käytät kaaviota näppäimistöllä.

## [0.4.3] - 2026-09-19

### Changed
- FinStats muistaa, mitä kukin sivu viimeksi näytti: palaaminen profiiliin, kirjastoon tai aikaväliin, jota olet jo katsonut, piirtää sen heti profiilikuvineen kaikkineen, kun tuoreet luvut latautuvat taustalla, ja sivu piirretään uudelleen vain, jos jokin muuttui. Muistettu säilyy selaimen välilehdessä ja pudotetaan, kun kirjaudut ulos tai muutat jotain.
- Latauksen paikkamerkit näkyvät vain, kun lataus on todella hidas. Ennen jokainen sivu näytti ne vähintään kolmanneksen sekunnista.

### Fixed
- Kirjastoja, jotka poistettiin Jellyfinissä eikä niissä koskaan ollut toistoa, ei enää luetella. Poistettu kirjasto, jolla on historiaa, näytetään yhä poistetuksi merkittynä.

## [0.4.2] - 2026-09-19

### Fixed
- Käyttäjät-sivulla **Ylläpitäjä**-tunniste oli omalla rivillään ja teki rivistä muita korkeamman. Tunnisteet ovat nyt nimen vieressä.

## [0.4.1] - 2026-09-19

### Fixed
- Sana ”null” ei enää näy yleiskatsauksen otsikon alla joulu- ja tammikuun ulkopuolella, kun vuosikatsauksen banneria ei ole näytettävänä.

## [0.4.0] - 2026-09-19

Näe, mikä muuttui, poistumatta sovelluksesta

### Added
- **Muutosloki**-välilehti, joka näyttää jokaisen julkaisun ja merkitsee käyttämäsi version.
- Tilapalkin versio linkittää siihen, ja välilehdessä näkyy päivityksen jälkeen piste, kunnes olet katsonut.

### Changed
- Historian liittäminen uudelleennimettyihin kohteisiin on paljon nopeampaa: 18 sekunnista kolmannekseen sekunnista 666 orvolla nimikkeellä, joten käynnistys pysyy välittömänä historian kasvaessa.

## [0.3.3] - 2026-09-19

### Changed
- Kirjasto luetaan uudelleen heti, kun Jellyfinin **Skannaa mediakirjasto** -tehtävä päättyy, ei koskaan erillisellä ajastimella eikä koskaan skannauksen ollessa käynnissä. FinStats on aina vain lukenut Jellyfiniä eikä koskaan käynnistä siellä skannausta.
- Viikoittainen varmistusluenta kattaa palvelimet, jotka luottavat reaaliaikaiseen valvontaan ja joilla skannaustehtävää ei ehkä koskaan ajeta.
- Vanha väli on nyt vain varajärjestely sitä varten, että seuraaminen on kytketty pois tai palvelin ei ilmoita skannaustehtävää.

### Added
- **Seuraa Jellyfinin kirjastoskannausta** -kytkin kohdassa Asetukset → Keräys, oletuksena päällä.

## [0.3.2] - 2026-09-19

### Fixed
- Historia säilyy uudelleennimeämisissä. Jellyfin antaa uudelleennimetylle tiedostolle uuden tunnisteen, mikä jätti ennen sen aiemmat toistot raa'an kansionimen alle (`Title (2010) [tmdbid-…] [imdbid-…]`) ilman julistetta. Nuo toistot liitetään nyt nykyiseen kirjaston kohteeseen: TMDB-, IMDb- tai TVDB-tunnisteella, kun vanhassa nimessä sellainen on, muuten siistityllä nimellä ja vuodella, ja jaksot sarjan sekä kauden ja jakson numeron mukaan. Osuman on oltava yksiselitteinen, joten oikeasti poistetut nimikkeet pysyvät ennallaan.
- Jellystatista tuodut suorat TV-kanavat laskettiin elokuviksi, koska Jellystat ei tallenna kohteen tyyppiä. Ne tunnistetaan nyt suoraksi TV:ksi, ja uudet suoran TV:n toistot tallennetaan Jellyfinin omalla tyypillä.

### Changed
- Suora TV jätetään vuosikatsauksesta kokonaan pois: kärkilistat, kokonaismäärät, katsojatyyppi, ennätykset ja sijoitus.

### Added
- `finstats relink` liittää historian uudelleen pyynnöstä. Se ajetaan myös käynnistyksessä ja jokaisen kirjaston luennan jälkeen.

## [0.3.1] - 2026-09-19

### Changed
- Vuosikatsaus on tiukasti henkilökohtainen: jokainen, ylläpitäjät mukaan lukien, näkee vain omansa, ja ”Kaikki”-näkymä, käyttäjänvaihdin ja palvelinluku on poistettu.
- Vuosikatsaus avautuu ”valmiiseen” vuoteen: joulukuussa kuluvaan vuoteen, muuten edelliseen. Muut vuodet ja ”Viimeiset 12 kuukautta” ovat yhden napsautuksen päässä.

## [0.3.0] - 2026-09-19

Vuotesi katsauksena

### Added
- **Vuosikatsaus**-välilehti, vieritettävä tarina vuodesta: katsotut tunnit, suosituimmat sarjat, elokuvat, kappaleet ja lajityypit sekä katsojatyyppi (yökyöpeli, aamuvirkku, viikonloppusoturi, sarjojen ahmija, elokuvafriikki, musiikin ystävä, tapojensa orja) vuorokaudenajan ja viikonpäivän kaavioineen.
- Vuosi kuukausittain, kunkin kuukauden määrittäneen nimikkeen kera.
- Ennätykset: suurin päivä, suurin ahmimisrupeama, pisin päivittäinen putki, pisin yksittäinen toisto, eniten uudelleen katsottu nimike, vuoden ensimmäinen toisto ja vanhin katsottu nimike.
- Löydöt: aloitetut uudet sarjat, loppuun katsotut elokuvat ja jaksot sekä sarjat, joista katsottiin täsmälleen yksi jakso.
- Yhden rivin banneri yleiskatsauksessa joulu- ja tammikuussa, kun vuosikatsaus on valmis.

## [0.2.0] - 2026-09-19

Enemmän kuin Jellystat koskaan tallensi

### Added
- Aikajana jokaiselle toistolle: aloitus, tauko, jatkaminen, hyppy, ääni- ja tekstitysvaihdot, suoran toiston muuttuminen transkoodaukseksi ja pysäytys. Lisäksi taukojen ja hyppyjen määrät, mistä toisto jatkui ja oliko katsoja lähiverkossa.
- **Palvelin**-välilehti: Jellyfinin versio, odottavat päivitykset ja uudelleenkäynnistykset, levyn käyttö kirjastoittain (Jellyfin 10.11+), lisäosat, ajastettujen tehtävien tulokset ja jokainen rekisteröity laite.
- **Toiston** oivallukset: samanaikaiset toistot ajan mittaan, mitkä sovellukset pakottavat transkoodauksen, kuinka pitkälle ihmiset pääsevät ennen lopettamista, paikalliset ja etätoistot sekä katselutottumukset.
- Mistä kirjastosi koostuu: resoluutiot, koodekit, dynamiikka-alue, säiliöt, nimikkeet vuosikymmenittäin, lisäykset kuukausittain, suurimmat nimikkeet ja nimikkeet, joita kukaan ei ole koskaan katsonut, tarkistettuna sekä FinStatsin historiasta että Jellyfinin omista toistettu-merkinnöistä.
- Yleiskatsaus: eniten samanaikaisia toistoja, arvioitu siirretyn datan määrä, etätoistojen osuus, lajityypit katseluajan mukaan ja epäonnistuneet kirjautumiset.
- Nimikesivut: IMDb- ja TMDB-linkit, studiot, bittisyvyys ja kuvataajuus sekä ketkä ovat merkinneet nimikkeen toistetuksi Jellyfinissä.
- Käyttäjäsivut: lajityypit sekä elokuvat, jaksot ja suosikit, jotka Jellyfinillä on tallessa.

### Fixed
- Nyt toistossa -kortin aikanimiö ei enää valu kortin ulkopuolelle.
- Toisto-sivun pitkät nimiöt katkesivat liian aikaisin.
- Kirjastosta poistunut sarja näyttää yhä kausi- ja jaksoluettelonsa.

## [0.1.0] - 2026-09-19

Ensimmäinen julkaisu

### Added
- Reaaliaikainen keräys Jellyfinin istunnoista: käyttäjä, nimike, sovellus, laite, IP-osoite, toistotapa, transkoodauksen syyt ja laitteistokiihdytys, video-, ääni- ja tekstitystiedot sekä katseluaika verrattuna taukoaikaan.
- Kirjautuminen Jellyfin-tililläsi. Kaksivaiheinen käyttöönotto luo FinStatsin oman API-avaimen, eikä salasanoja koskaan tallenneta. Muut kuin ylläpitäjät voivat halutessaan kirjautua nähdäkseen vain omat tilastonsa.
- Yleiskatsaus-, Toiminta-, Käyttäjät-, Kirjastot-, Toisto- ja Palvelimen loki -sivut, komentopaletti (Ctrl/⌘ K) ja reaaliaikainen tilapalkki.
- Jellystat-tuonti: suuret varmuuskopiot virtaavat levylle ja tuodaan sekunneissa, ja saman tiedoston tuominen kahdesti on turvallista.
- Yksi pieni binääri, jossa on upotettu verkkokäyttöliittymä ja yksi SQLite-tiedosto, sekä Docker-levykuva.
