# Endringslogg

Alt som e endra i FinStats, d nyaste fyrst. Dinna fila vert kompilert inni
programfila og e d som fana **Endringslogg** i appen viser.

Format: `## [version] - date`, elle `## [version] - first day to release day` for ei utgjeving som vart laga
over fleire dagar, deretter `### Added`, `### Changed`, `### Performance`, `### Stability`, `### Fixed`
elle `### Removed` (Nytt, Endra, Yting, Stabilitet, Retta, Fjerna) me eitt ponkt per endring. Yting e nåke som
vart raskare elle lettare, Stabilitet nåke som ikkje lenger kan krasje, stanse, lekke elle miste data, Retta nåke
som gidde feil svar. Ei `x.y.0`-utgjeving he ei kort tittellinje under overskrifta si: den tittelen bruka appen
som overskrift for heile serien.

## [2.2.0] - 2026-10-01 to 2026-10-07

Bibliotekhelse og FinUI

### Added
- **En eie meny på datamaskina**: Innstillingar → Utsjånad tilbyr åtte: sidepanele sann d va, gruppert ette ka sidene e til, ei skjene me ikon som opnar sej til sidepanele under peikari, en sidepanel som starta me søk, ei dokk, ei kommandolinje, sider du festar øvst, og farga fliser. Sidepanela kan stå til høgre. Dokka glir bort menst du rullar nedover og kjøme att når du rullar oppover elle peikar mot botni av vindauge; me ho flyttar statuslinja sej til toppi.
- **En eie meny på telefoni**: ei fanelinje me Meir (den nye standardi), en ark som opnar sej halvvegs, heile skjermi, sidene spreidde i ei vifte rundt tommeli, elle ei adresselinje som veks til menyen. Kvar av dei søkjer i sidene sine og sender alle andre ord vidare til søke.
- Begge vert hugsa av nettlesari dei vart valde i, so telefoni og datamaskina helde kvar på sitt; begge e FinUI-komponenta, `desktop-nav` og `mobile-nav`.
- **En kontekstmeny**: høgreklikk på en tittel, en person elle en bibliotek kor som helst i FinStats (en plakat på Oversikt, ei rad i ei liste, en namn i en tabell, en film i Kjøme snart som endå ikkje e i biblioteke), hald fingeri på d, elle trykk på menytasti, og en meny veks ut ifrå ponkte du trykte på: opne, opne i ny fane, legg en film elle en serie til i Lista mi, opne i Jellyfin elle på TMDB og TVDB, kopier lenkja. Shift me høgreklikk gir framleis menyen til nettlesari. `context-menu` ifrå FinUI.
- **Søk utan dialog**: søke veks ut ifrå menyen din (en sidepanel vert breiare, dokka stig opp, namne i kommandolinja vert til søkjefelte, menyen på telefoni helde d) elle, om du vel d i Innstillingar → Utsjånad, tek plassi til sida menst menyen vert verande. Ctrl+Space opnar d som før; vindauge over alt saman e borte. `search` ifrå FinUI.
- **Alt i en bibliotek**: korte Nyleg lagt til for en bibliotek tilbyr *Alt i det*: alle filmane, seriane elle albuma d he, som plakata, ette namn, år, dato lagt til elle storleik, avgrensa menst du skriv. Fleire kjøme til menst du rullar.
- **Server → Bibliotekhelse**: FinStats samanliknar kvar fil me naboane hennar og listar d som skil sej ut, kvart funn me tala bak. D rapporterer bære; ingenting vert endra, sletta elle skanna på nytt i Jellyfin.
- **Hol**: episoda som manglar myllå to filer i en sesong («Sesong 2: episode 4 og 5 manglar myllå 3 og 6»). Ei fil som helde to episoda, tel som begge; spesialepisoda og slutti av en sesong vert ikkje rørte.
- **Sesonga som skil sej** ifrå resti av serien i oppløysing, HDR elle kodek, og **avvikande episoda**: ei fil som ikkje liknar resti av sesongi, til dømes en SD-episode i en 1080p-sesong. Oppløysinga går ette breidd elle høgd, so en beskoren film på 1920×800 e 1080p.
- **Kopia og versjona**: den samme filmi elle episoden to gonga, ette dei leverandør-ID-ane som Lista mi og På veg alt går ette. To kopia i samme oppløysing viser plassi som går me to gonga; en 4K-kopi ved sida av en 1080p-kopi vert lista som to versjona og aldri rekna som sløsing.
- **Tynne filer**: en bitrate langt under d oppløysinga og kodeki vanlegvis krev («4K HEVC på 1,1 Mbps»); dei tynne episodane i en sesong e eitt funn.
- **Dubbing som stoppar**: en lydspråk som nokren sesonga he på kvar episode og andre på ingen.
- **Aldri identifisert**: filma og seria Jellyfin he kjent att utan TMDB-, TVDB- elle IMDb-ID.
- **Avvis** en funn som e i orden me vilje, me en merknad. D kjøme att av sej sjølv når filene endrar sej. Dei som kan administrere FinStats og sjå serveri, kan avvise; avvisingar står i revisjonsloggi og e me i en tryggingskopi.
- Sida til en bibliotek seier kor mykje d e å sjå på der, og leier til d.
- **FinUI**, komponentane FinStats e bygd av, e en eie prosjekt (`github.com/finstats/finui`): kvar knapp, kvart kort og kvar kontroll i alle tilstanda og begge tema på `finui.finstats.no`, og **FinUI create** for å endre farga, hjørneradius, tettleik og meir menst du ser på, og ta d me heim me éin kommando. **Innstillingar → System** leier dit.
- **Innstillingar → Utsjånad**: alle vel korleis FinStats ser ut for dei, me koden til en førehandsval ifrå FinUI: grunnfarge, aksentfarge, diagramfarga, kontrast, hjørneradius, tettleik, kanta, kort, knappa, felt, tabella, utheving, rørsle, ikon, menyen, sida bak alt saman, fokusringi, korleis overskrifter e sette, og skriftene for tekst, overskrifter og tal. Vale ditt gjeld bære dej; sida viser ka koden tyde og leier til FinUI create for å endre han.
- **Stila**: elleve heile utsjånada å starta ifrå me eitt val i FinUI create, og Innstillingar → Utsjånad tek imot koden. Dei e Washi (FinStats sann d vert levert), Studio, Noir, Matinee, Console, Pastille, Ledger, Gazette, Arcade, Canopy og Harbour. FinStats sjølv kjøme me éin utsjånad og tilbyr inga liste over stila.
- **Ikon som rører sej**: kvart ikon i ei lenkje, en knapp elle ei fane gjær si eie rørsle eingong når du peikar på d (oppdater snurrar, pila i last ned fell ned i korga si, papirkorga lyftar lokke), og ei oppdatering du he trykt på, helde fram me å snurre til svare e kome. Står stille me redusert rørsle og me Motion: Off.
- **Rørsle, me FinMotion**: komponentane i FinUI rører sej no: knotti på en brytar vert kasta over, streki under ei fane krabbar bort til neste fane, en felt som vert avvist, ristar på hovude, en kopi vert kryssa av me eitt strøk, meldingar ligge i bunke som korta på ei hand og breier sej ut når du peikar på dei, en ark på telefoni gir ette som en strikk når du dreg i d, radene i en sortert tabell flyttar sej til plassane sine, en tal som endrar sej, rullar som en kilometerteljar, og diagram teiknar sej sjølve første gongi dei vert viste. Alt saman følgjer vale Motion i Innstillingar → Utsjånad og står stille me redusert rørsle. FinMotion e en eie prosjekt (`github.com/finstats/finmotion`), lagt oppå FinUI.
- **Fleire ikon i FinUI**: 239 no, opp ifrå 71, kvart me si eie rørsle (volum opp og ned, en hovudsett og en dempa hovudsett, en blyant som skriv, ei bjølle som ringjer, en telefon som surrar), og alle kan lastast ned ifrå gallerie til FinUI som SVG, stille elle animerte.
- **Skrifter**: trettifire skriftfamilia me open lisens å velje myllå (Geist, Roboto, Manrope, IBM Plex Sans, Montserrat, Lora, Playfair Display, Fira Code og fleire), pakka me sann Inter e, aldri henta ifrå andre stada, og lasta bære når dei e valde.
- **Fleire grunnfarga og aksentfarga**: Nord, Solarized, Gruvbox, Catppuccin, Rosé Pine, Tokyo Night, Dracula, Everforest og Ocean; seksten nye aksentfarga, ti av dei me éin farge om dagi og en anna om natta. Kvar grunnfarge he no sitt eie papir om dagi: dei grå såg like ut i d lyse temae.
- **Sletting kan angrast i 30 daga**: ei sletta avspeling elle en sletta tryggingskopi venta i papirkorga, og **Angre** i meldinga som kjøme etterpå, hentar d tilbake. Aktivitet og Innstillingar → Tryggingskopiar viser kvar for sej d som nyleg vart sletta, og når d forsvinn for godt, so sant d finst nåke. Ei avspeling i papirkorga e halden utanfor alle summa, diagram, tilbakeblikk og profila, og en import elle ei gjenoppretting hentar ho ikkje tilbake som ny; den som såg ho saman me nokon, vert rekna som åleine til ho kjøme tilbake.

### Changed
- Dark Reader og liknande utvidingar lèt sidene til FinStats være (`darkreader-lock`): FinStats teiknar begge tema sjølv, og ei side som vart måla om, kjempa mot dei.
- På telefoni glir ikkje menyen lenger inn ifrå venstre: fanelinja står nedst om ikkje en anna meny e vald.
- FinStats les biblioteke ditt eingong ette dinna oppdateringa for å lære kor filer som helde fleire episoda, sluttar.
- På sida til en tittel e **Legg til i Lista mi** like høg som **Opne i Jellyfin** ved sida av; han va en storleik mindre.
- Filtera for tidsrom og persona vert verande øvst på skjermi menst sida rullar, so 7 d vert Alt ifrå botni av ei lang side; d samme gjær filtera for førespurnada og Kjøme i På veg.
- Knappi for å rulle til toppi helde sej unna den menyen som he botni av vindauge: lågt i hjørne ved sida av dokka, rett over fanelinja, tommelknappi elle adresselinja på telefoni, og elles over statuslinja. Han e `to-top` ifrå FinUI.
- Ei avspeling elle en tryggingskopi som vert sletta, går til papirkorga istade for å verte fjerna me eingong (`PUT` me `{"deleted": true}`; `DELETE` e fjerna ifrå API-et). Tryggingskopia ut over tale som vert teke vare på, vert framleis fjerna for godt.
- **Endringslogg** e eitt ark istade for en stabel me kort: kvar merknad e ei eie linje me typi sin (Nytt, Endra, Retta og resti) og ikone for typi i ei kolonne framfor, under ei overskrift for kvar serie. Lista over versjona står framleis til høgre.

### Performance
- Når du opnar FinStats, vert bære d som e endra, lasta ned: en stilark, en skript elle ei skrift som nettlesari alt he, vert stadfesta me nokren få byte istade for å verte sendt på nytt. Før henta kvart besøk alle stilarka og modulane til FinStats i heilskap, sjølv om kvar av dei hadde en tagg som skilde dei ifrå kvarandre.

### Stability
- Gjenoppretting av en tryggingskopi les innstillingane bære ifrå rada for innstillingar: ei fil ifrå en nyare FinStats som he meir der, fekk innstillingane sine forkasta utan en ord.
- Ei open fane veks ikkje lenger for kvar side du he vore på: kvar side du gjekk ifrå, vart liggjande i minne bak den på skjermi (om lag 2 600 element per besøk), halden fast av lista FinMotion he over d som ska rørast, og av rullelyttari til filterlinja, til fana vart lukka.
- En feilskriven kommando (`finstats bakcup`), elle `restore` utan fila si, lagar elle oppgraderer ikkje lenger en database før FinStats seier at d ikkje veit ka d ska gjære.

### Fixed
- Den vesle grafi i ei flis går ikkje lenger ut over flisa når tale ved sida av e breitt («56 d 23 t»), elle når fire fliser deler ei smalare rad: grafi gir ette, og tale helde breidda si.
- I d lyse temae vart en primærknapp fiolett under peikari; no helde han fargen til segle.
- I Innstillingar står en brytar på linje me namne og hjelpeteksti til innstillinga; han hang øvst i rada.
- I ei rangert liste utan nåke under namna, som Toppbrukarar på Oversikt, stod namne øvst i rada ved sida av en sentrert bilete; no e d sentrert.
- Ei rad me statistikkfliser i ei breidd myllå telefon og datamaskin (fire fliser, tre i breidda) la att den siste åleine på ei eie rad; fire fliser står no fire, to elle ei i breidda, og tre står tre elle ei.
- Lesinga av serveri talde me ei eining som ingen hadde logga inn på, og som FinStats ikkje tek vare på: oppgåva sa tre einingar der Server-sida lista opp to.
- Eitt sekund, eitt minutt, éin time: varsla sa «1 seconds apart» og «in 1 minutes», og ei Jellyfin-oppgåve me intervall på eitt minutt køyrde «every 1 minutes».
- `finstats --help` nemner kvar variabel FinStats les; `FINSTATS_PUBLIC_IP_URL`, `FINSTATS_ALLOW_LIBRARY_SHRINK` og `FINSTATS_SKIP_PREUPDATE_BACKUP` mangla.
- Oppsettsvegvisari sa «kunne ikkje kople til» om nåke som ikkje va ei adresse, i ei setning som rann inni den neste; no seier han at d ikkje e ei adresse.
- I Innstillingar → Tryggingskopiar forsvann en spørsmål som vart stilt menst en tryggingskopi vart skriven, når tryggingskopien va ferdig.
- Me menyen som ei skjene som opnar sej, synte d å lukke søke både d opne søke og den lukka skjena samstundes, og so stod skjena open til peikari kom og gjekk. Esc gir no fokuse til tastature tilbake dit d va då søke opna sej, og slepp fokuse en klikk gidde.
- På telefoni heldt sida sej dempa til søke va lukka heilt, og hoppa so til full lysstyrke i eitt bilete; no tonar dempinga ut saman me søke.
- Stega for Tautulli-importi sa at tryggingskopien vert lasta ned ette **Backup Database**. Tautulli lagrar han på si eie maskin: stega seier no kor (mappa `backups`, `/config` me Docker), ka fil du ska ta, og at du ska kopiere ho over.
- Søk som vart lukka før d va ferdig me å opne sej, tok animasjoni ifrå skjena som opnar sej: ifrå då av opna og stengde ho sej me eitt.
- Skjena som opnar sej, vart ståande open ette at ei side va vald ifrå ho, til en klikk en anna stad, og i nokren nettlesara, elle me zoom, teikna ho en rullefelt under ikona. No opnar ho sej bære for peikari og tastature, og teiknar ikkje nåke rullefelt når ho e lukka.
- På Tryggleik-sida tel ei adresse som databasen plasserer i en land, men ikkje på karte, me for d lande, istade for blant adressene utan kjend stad.

## [2.1.2] - 2026-10-01

### Added
- **Import ifrå Tautulli**: ta me dej historikki ifrå Plex. Last opp databasetryggingskopien ifrå Tautulli (`.db`-fila, elle `.zip`-fila som helde ho) under **Innstillingar → Import**; avspelingane får sin eie plass i filtere *Registrert av* i Aktivitet.
- **En koplingsbrett for å seie kven som e kven**: Plex-brukara på den eine sida, Jellyfin-brukara på den andre. Dra en leidning ifrå kvar Plex-brukar til den dei e no, elle klikk på den eine og so den andre, elle bruk tastature. Den som står utan leidning, vert ikkje importert, og to Plex-kontoa kan gå inni éin person.
- Filma og episoda vert kopla til biblioteke ditt ette namn (en film ette tittel og år, en episode ette serie og nummer), og ein som endå ikkje e på serveri, vert teken vare på og kopla på når han kjøme. Ei vising som vart teken opp att, e éi avspeling; musikk vert ikkje importert.
- Tryggingskopien helde tilgangstokena til Plex: FinStats les dei aldri, og fjernar fila so snart du importerer elle starta på nytt.
- **Ukopla media** under **Innstillingar → Data**: kvar tittel ifrå Tautulli, Jellystat elle Streamystats som ikkje passar me nåke i biblioteke ditt, me kor han kom ifrå og kor han truleg alt finst. **Finn** koplar han ifrå dei beste gjetningane elle ifrå en søk (en film kan plasserast som spesialepisode i en serie), og FinStats huska vale, so den samme historikki importert på nytt koplar sej sjølv. Vala e me i en tryggingskopi.

### Performance
- **Tilbakeblikke opnar tre til fem gonga raskare**: dei mest sette skodespelarane og regissørane i åre vart rekna ut ved å gå gjennom åre på nytt eingong for kvar person i d. Målt på en verkeleg historikk gjekk en år ifrå 0,29 s til 0,09 s og dei siste 12 månadene ifrå 0,53 s til 0,12 s.

### Fixed
- **Importert historikk finn titla som går under en anna namn**: ei avspeling passar også me namne på originalspråke, so en serie Plex elle Tautulli kjende som 오징어 게임, vert kopla til Squid Game, og *Im Westen nichts Neues* til *All Quiet on the Western Front*. Originaltitlane i biblioteke ditt vert lesne ved neste skanning.
- Namn vert samanlikna sann søke samanliknar dei (store og små bokstava, aksenta, teiknsetjing, en årstal skreve inni namne, en innleiande «The»), og utgjevingsåre kan være eitt år feil, so «Star Wars: Episode I - The Phantom Menace» finn «… – The Phantom Menace», og en Kingsman ifrå 2014 finn den ifrå 2015. En tittel som e kjend under en heilt anna namn, vert framleis aldri gjetta.
- En mislukka Streamystats-import elle ei mislukka gjenoppretting vart meldt i varsla me d interne namne sitt.
- Knappi *Del året som kort* i Tilbakeblikke og sjåarplasseringa ved sida av hadde ikkje mellomrom myllå sej.

## [2.1.1] - 2026-10-01

### Changed
- **Lista mi vert tilboden overalt der en tittel e**: på korta i Kjøme snart på Oversikt, sann som i På veg → Kjøme, og på sida til en episode elle en sesong, som set serien på lista.
- På en plakat på Oversikt viser knappi for Lista mi sej menst du peikar på korte elle når d me tastature, istade for å dekkje hjørne på kvar plakat; på en trykkskjerm e han alltid der.

### Stability
- Å lagre ei Sonarr-, Radarr- elle Seerr-tilkopling menst nokon hadde **Innstillingar → Utgåande** open, kunne la begge vente på kvarandre for godt.
- **Innstillingar som ikkje kan lesast, stoppar FinStats ved oppstart** og nemner innstillinga d gjeld, istade for å stille alle innstillingane tilbake til standard i d stille.
- **To endringar i innstillingane samstundes** (til dømes innstillingssida og tidsplani for ei oppgåve) opphevar ikkje lenger kvarandre.
- En pause, ei spoling elle en sporbyte som vart sett menst databasen va oppteken, vert skreve ved neste runde istade for å gå tapt.
- Ei lesing av biblioteke feilar ikkje lenger når Lista mi til nokon he fleire oppføringar som viser sej å være samme tittel.

### Fixed
- **Ei utbytt fil ekje «no på serveri»**: å oppgradere fila til en film på Lista mi til nokon seier ikkje lenger at filmi nettopp he kome.
- Éi mislukka lesing av programtillegga, dei planlagde oppgåvene elle lagringa i Jellyfin tømmer ikkje lenger den lista på Server-sida fram til neste lesing.
- **Ei Streamystats-avspeling stoppar der ho stoppa, på kvar side**: fråfallskurva for en tittel og «fullført» for en episode gjekk istade ette lengda og rekna ho som anslått.

## [2.1.0] - 2026-09-29 to 2026-09-30

Lista mi

### Added
- **Ei liste som e di eie**: set en film elle en serie på ho ifrå sida til titteli, ifrå Kjøme, ifrå Nyleg lagt til elle ifrå søke (Ctrl+Enter), og finn ho under **Lista mi** på din eie profil.
- **Kvar oppføring seier kor ho står no**: på serveri, «5 av 26 episoda», førespurd, «Kjøme fredag», ikkje på serveri, forsvann ifrå biblioteke, elle sett. D vert rekna ut kvar gong du ser på lista, me den samme tydinga av «sett» som framdrifta di i seria. Sette titla vert verande, i ei eie gruppe, til du tek dei bort.
- **En tittel som endå ikkje e på serveri**, kan kome på lista rett ifrå d Sonarr elle Radarr venta på. Oppføringa vert kopla til titteli av sej sjølv når han kjøme, og ei fil som får nytt namn, tek oppføringa me sej.
- **«På Lista mi: no tilgjengeleg»**: en varsel til dine eigne mottakara når nåke du venta på, kjøme. Av til du kryssa av for d.
- **Ingen andre ser lista di**, ikkje eingong en administrator, og varsele går bære til dine mottakara. Ingenting vert skreve til Jellyfin, Seerr, Sonarr elle Radarr.
- Listene e me i en tryggingskopi, og ei gjenoppretting slær dei saman utan å doble nåke.

### Changed
- `/api/upcoming`-oppføringar he TVDB- og TMDB-ID-ane til titteli sin, og `/api/auth/me` seier om serveri he Lista mi.

## [2.0.4] - 2026-09-29

### Added
- **Planlegging av oppgåver, sann Jellyfin gjær d**: kvar jobb FinStats gjær på eie hand, he utløysara (dagleg på en klokkeslett, kvar veke på en bestemt dag, me en intervall, ved oppstart elle ette biblioteksskanninga i Jellyfin), kvar me ei valfri tidsgrense. Klikk på ei oppgåve for å sjå og endre timeplani hennar; *Legg til utløysar* opnar den samme dialogi som Jellyfin he.
- **Oppgåver e en eie seksjon** i Innstillingar, me siste køyring for kvar jobb i ord («Køyrde for 38 minutt sida, tok 6 minutt») og en Køyr-knapp; før lå han øvst i System.
- **Metadataendringar**: kvar time les FinStats d Jellyfin he lagra sida sist (namn, omtala, sjangra, vurderingar, medverkande, fildetalja og bilete), so ei endring i Jellyfin kjøme fram utan å vente på ei biblioteksskanning.
- Tryggingskopi og geolokaliseringsdatabasen kan også køyrast ifrå Oppgåver; Esc går tilbake ifrå timeplani til ei oppgåve til lista.

### Changed
- **Temae e en brytar me tre stopp**, Lyst · Eining · Mørkt, der knappi som bladde gjennom dei, va før: klikk på en tredjedel av han, dra knotti elle bruk piltastane. Knotti ber ikone for vale, og ikone glir inn når vale vert endra.
- **Utlogging ligge under Innstillingar → Konto**, som seier kven som e innlogga her; sidepanele helde på temabrytari.
- **Dei gamle planinnstillingane e no utløysara**: *Følg biblioteksskanninga i Jellyfin* me reserveintervalle sitt, *Ta tryggingskopi kvar N. dag* og GeoIP-brytari *Hald databasen oppdatert* vart til dei tilsvarande utløysarane på oppgåvene sine, me verdiane du hadde. Innsamling, Tryggingskopiar og Tryggleik lenkjer til timeplani istade.
- En oppstart køyrer ikkje lenger alle jobbane på nytt: d vert teke vare på når kvar av dei sist vart ferdig.
- En Jellyfin-jobb som ikkje he nåke overslag endå, viser «om lag … att», me prikka som går rundt der tida ska stå, istade for «ETA …».

### Performance
- **Karte under Tryggleik heng ikkje lenger ette**: å dra og zoome flyttar karte som alt e teikna, og d vert teikna på nytt eingong når rørsla he stilna, og pulsi for d som spelast no, får ikkje lenger heile karte til å teikne sej 120 gonga i sekunde menst ingen rører d. Ei draging gjekk ifrå 34 til 60 bilete i sekunde på ei treg datamaskin.

### Fixed
- **En plakat elle en portrett som e bytt ut i Jellyfin, kjøme fram i FinStats**: bilete vart mellomlagra ei veke på serveri og ei veke til i nettlesari, samme ka Jellyfin hadde fått sida. No vert dei mellomlagra under biletmerke til Jellyfin, so en nytt bilete e ei ny fil, og nettlesari sjekkar at kopien hans e gjeldande kvar gong han viser eitt. Når en portrett vert bytt ut, vert bære personi lagra på nytt, so Metadataendringar les personane også.
- **Server → Jobba viser en overslag me eingong sida opnar sej**: tida som va att, vart bære målt ut ifrå avlesingar sida sjølv gjor menst ho va open, so kvart besøk starta på «ETA …» til sida hadde sett prosenti flytte sej. No tek FinStats sjølv tida på en Jellyfin-jobb som køyrer, kvart 10. sekund menst han køyrer, og sida får overslage servert. En jobb som Jellyfin rapporterer på 0 % heile tida (den første fasen av ei biblioteksskanning), helde på prikkane til han flyttar sej: ingenting vert gjetta.
- **Verktøytips på karte kjøme på prikki**: i Firefox kunne verktøytipse til prikki for d som spelast no, hamne langt unna, og der prikka ligge over kvarandre (nokon som spelar av heime), va d en piksel som avgjorde ka verktøytips som kom fram. No namngjev eitt verktøytips alt som ligge under peikari.
- **Statuslinja he éi form når peikari e over**: kvar lenkje (straumane, Repo, versjoni) lyser opp som den samme vesle boksi, og prikkane myllå elementa sitte midt i mellomromma. Me peikari over versjoni lyste før også prikki og mellomromme framfor opp.
- **En filsti tek heile breidda av korte sitt**: på Fil-korte til en tittel, og for mappene til Jellyfin på Server-sida, vart en sti klemd inni éi smal celle og rann nedover ni liner ved sida av tomrom.
- En dialog som tonar ut, slukar ikkje lenger en klikk på sida under.

## [2.0.3] - 2026-09-28

### Stability
- **Bære éin import elle gjenoppretting om gongi**: to opplastingar sende samstundes kom begge gjennom kontrolli, og ei gjenoppretting såg aldri ette importa; den andre venta ut databasen og feila ette fleire minutt me opplasting.
- **Ei adresse me ei øydelagd `%`-koding opnar sida for «ikkje funne»**, istade for å late en lasteindikator gå for alltid.
- **En stort sidetal i ei liste gir ei tom side**: tale flaut over og kom tilbake som den første sida igjen.

### Fixed
- **Tryggleik:** Server-sida gidde ID-en til kvar eining til alle me *Sjå server*; ID-ar krev *Sjå nettverk* i tillegg, som alle andre stada.
- **Tryggleik:** to løyvebrytara som vart slegne om før den første va lagra, sende den samme startlista to gonga, og den andre gjor om den første: en løyve som va teke bort, vart verande gjeve, og en seksjon av den offentlege profili som va slegen av, vart verande publisert, menst begge brytarane viste d som va valt.
- **Tabella sorterer tal rett i en tysk elle norsk nettlesar**: «1,204» vart lese som desimaltale 1,204 og ikkje som 1 204, og alt over en million som tekst.
- **Tabellvisinga til en diagram sorterer datoa, vekedaga og posisjona ette tid**, ikkje ette bokstavane.
- **Nedlastingshistorikki seier kor importane kom ifrå**: Sonarr og Radarr namngjev indekserari og protokolli bære når ei utgjeving vert henta, so desse fordelingane va alltid tomme. En mislukka sesongpakke e éin feil, ikkje éin per episode.
- **Å slette ei avspeling som vart sett saman,** finn gruppene til titteli på nytt, istade for å late att ei «gruppe» på éin.
- **Tid saman e for éin person aldri meir enn kor lenge personi sjølv va me**: Saman-sida godskreiv personi heile kveldi.
- **En kveld i lag me andre over midnatt nyttårsaftan** (elle over kanti av «siste N daga») høyrer heilt til tidsromme han starta i, istade for å verte to kvelda åleine.
- **Importerte episodeavspelingar får namne til episoden** der Jellystat bære tok vare på namne til serien.
- **Geolokalisering finn bydatabasen ved sida av Country- og ASN-databasane** som `geoipupdate` skriv, istade for å slå sej sjølv av.
- **Ponkte d oftast vert spola tilbake til, følgjer filtera på sida**, sann tala gjær.
- **«Åre ditt e klart»** går bære til folk som he nåke i tilbakeblikke sitt.
- **Å løyse en dempa varsel på nytt helde stadpare dempa**; bære å opne varsele att opphevar ei demping.
- **En utgått API-nøkkel tel ikkje lenger me blant dei tjue.**
- **Grensa for økter lèt den eldste økta gå** sjølv når monge innloggingar kjøme i samme sekund.
- **Ei heimeadresse som oppslage fann, vert verande** når den samme vert teken bort ifrå lista du he skreve inn.
- **Bilete i revisjonsloggi og i nøkkellista**, der d før bære stod initiala.
- **Varsellista til en administrator seier kven en personleg mottakar høyrer til**, istade for «Din» på alle; **Nyleg sendt** viser alle mottakarane av den siste hendinga.
- **Esc på en tittel elle en person tek dej aldri ut av FinStats**, og lukkar bære dialogi som ligge øvst.
- **Ei side vert teikna sjølv når ei lagring avbryt førehandslastinga ho venta på.**
- **Spelast no viser en sjåar som he pausa og spolar,** der sjåari e no, ikkje fyrst når avspelinga helde fram.
- **Å fjerne ei tilkopling oppdaterer ka sider som finst**, sann d gjær å lagre ei.
- **Byggjedatoen til geolokaliseringsdatabasen helde på årstale** i amerikansk engelsk.

## [2.0.2] - 2026-09-28

### Stability
- **En stans går aldri tapt**: når ei lesing av biblioteke vart avvist i dei første augneblinkane ette oppstart, før serveri lytta ette d, vart avvisinga notert, og FinStats køyrde vidare istade for å stanse.
- **En Jellyfin som svarar i ei form FinStats ikkje kan lese, vert avvist**, ikkje trudd: alle brukara, elle alle titla i en bibliotek, som kom under nøkla FinStats ikkje kjenner (`id` for `Id`), slapp forbi verne me folt tal og vart so merkte som fjerna.
- **Slutti på ei avspeling vert aldri mista**: når databasen va oppteken lenger enn ventetida (en import som heldt på han), vart ei avspeling som slutta akkurat då, ståande som i gang til neste omstart. No vert slutti skriven i neste runde.
- **Innstillingar som ei gjenoppretting ikkje kan lese, vert ikkje rørte**: en tryggingskopi der innstillingane ikkje lenger passa dinna versjoni, nullstilte alle innstillingane ved gjenoppretting: samanslåingsvindauge, løyve, heimeadresser, timeplan.
- **En kontrollteikn i en namn elle en tittel øydelegg ikkje lenger korta**: profilkorte, kvart historiekort som viste den titteli, og ZIP-fila me dei svarte me en feil.

### Fixed
- **Tryggleik:** bak en proxy talde innloggingsgrensa den første `X-Forwarded-For`-adressa, som e den klienti sjølv sender: ei ny oppdikta adresse for kvart forsøk, og grensa slo aldri inn. No e d den siste, den proxyen skreiv.
- **Tryggleik:** *Ta med IP-adresser og stader* sende adresser til en personleg mottakar der eigari manglar *Sjå nettverk*, sann at eigari fekk d inga side vilde vist han. Mislukka innloggingar for andre krev no *Sjå server* også, som på Tryggleik-sida.
- **Varsel stoppar for nokon som e deaktivert elle sletta i Jellyfin**: dei personlege mottakarane deira fekk framleis alt dei gamle løyva tillét.
- **En varsel i kø vert sjekka på nytt før d vert sendt**: en mottakar som va slått av, elle en eigar som hadde mista en løyve, fekk framleis d som venta på en nytt forsøk.
- **En mottakar som alltid svarar `Retry-After`, vert gjeven opp** ette fem forsøk som alle andre, istade for å verte prøvd på nytt i tretti daga.
- **Eining-ID-ar på sida til en person** vert bære viste me *Sjå nettverk*, som alle andre stada.
- **En publisert år venta eitt døgn me alt**: dei fullførte og droppa seriane i d følgde kvar avspeling, so å trykkje play på en droppa serie endra den offentlege sida i løpe av sekund.
- **En serie nokon he bede om, tel som sett** når ein av episodane e sett: kvar serie d va bede om, vart lesen som aldri sett, i kapitle Førespurnader og på korte Bede om.
- **Kapitle Saman i tilbakeblikke helde direkte-TV utanfor**, som resti av åre: en kanal som stod på menst folk såg saman, kunne verte den største titteli, og deli kunne gå over 100 %.
- **Dagar-korte for «dei siste 12 månadene» sluttar på den nyaste dagi**: rutenette mista dei siste tre elle fire vekene.
- **Nye episoda av en serie vert kunngjorde eingong per kveld**: episoda på kvar si side av midnatt UTC vart kunngjorde to gonga på en server aust elle vest for han.
- **Å kople til Seerr kunngjer ikkje gamle førespurnada som klare til å sjåast**: en førespurnad som alt va tilgjengeleg då han vart lesen første gong, og som ikkje hadde nåke anna som sa når, vart lesen som om han kom i samme augneblinki.
- **Titlane i en bibliotek som e sletta i Jellyfin, forsvinn me d**: dei vart verande i søk, på hylla Nyleg lagt til og i totalane.
- **En remux helde sej som direktestrøyming** når ei avlesing mistar omkodingsdetaljane en augneblink (ved ei spoling), istade for å verte omkoding resti av avspelinga.
- **«Jellyfin e nede» tyde at fleire lesingar feila på rad**: einskilde tidsavbrot me dagars mellomrom, på en server som lå roleg på socketi, vart lagde saman til d.
- **En Streamystats-import viser framdrifta si på sitt eie kort**, ikkje over d siste resultate ifrå Jellystat-importi.
- **Ei adresse der vertsnamne starta me «web»** (`http://web:8096`) vert lesen som en vertsnamn, ikkje kutta av som om d va stien til nettklienti til Jellyfin.
- **En køfeil gøymer heile innloggingsdeli** av ei adresse sjølv når passorde inneheld en skråstrek.

## [2.0.1] - 2026-09-28

### Changed
- **Språka til en tittel står ikkje i vegi**: toppfelte namngjev dei tre første lyd- og undertekstspråka, éi line for kvart slag, og en nytt **Språk**-kort listar kvart einaste eitt: dei fullstendige saman, dei delvise me kor monge episoda som he dei. Dei to dusin undertekstspråka til en lang serie skuva før bilete en heil skjerm nedover.

### Fixed
- **Plakati til en tittel helde sej ved sida av namne**: når teksti ved sida av vart lengre enn plakati (en langt samandrag, monge studio), sokk plakati ned til botni av toppfelte, under tomt bakgrunnsbilete.
- **En versjon FinStats ikkje kan lese, vert kalla d han e**: en `app_version` som va handredigert til nåke ulesleg, fekk FinStats til å nekte å starte fordi databasen skulle være opna av «den eldre FinStats 2.0.1», nåke som ikkje stemde. FinStats nektar framleis, men seier no ka verdi som ikkje kunne lesast, og korleis han ska skrivast.

## [2.0.0] - 2026-09-25 to 2026-09-28

Tilbakeblikk 2026, offentlege profila og å sjå saman

### Added
- **Kven du såg saman me**, i tilbakeblikke: kvelda i lag me andre, tida brukt sann, titteli som oftast samla folk, og opptil tre følgjesvena, namngjevne i appi og lenkja til profilane deira.
- **Seria sett til endes**, i tilbakeblikke: den siste episoden av kvar serie, sett d åre, ette den samme tydinga av «sett» som framdriftslinjene på profili, og òg dei som vart starta på og lagde til sides, under halvvegs sett og utan avspeling på to månada.
- **D du bad om**, i tilbakeblikke: Seerr-førespurnada gjorde i løpe av åre, kor monge som kom, og kor monge av dei du so såg. Bære der d finst førespurnada.
- **Åre mot åre før**: tima, avspelingar og daga, og kor mykje meir elle mindre.
- **Åre som ei historie**: kvart kapittel som en kort på 1080×1920, formate som historier på mobili bruka (åre, tala, toppseria, filma og musikk, sjangra, personlegdomi, tima og daga, kvar dag i åre, rekorda, selskap, fullførte seria, førespurnada og åre før). Lagre eitt, elle alle som en ZIP, ifrå «Del åre som kort».
- **En kort namngjev aldri nokon annan**: korta vert teikna ifrå en kopi av åre som hekje felt for en anna person, ei rangering elle en app. I appi he en kort ikkje nåke namn i d heile.
- **Åre til serveri**, for Jellyfin-administratora: kvar tittel og kvar total og personlegdomi til huse, utan at nokon vert rangert elle namngjeven. En brytar på tilbakeblikksida, `scope=server` i API-et.
- **«Tilbakeblikke ditt for åre e klart»**: en varsel i desember, eingong til kvar person som såg nåke d åre.
- **Offentlege profila**: en person kan publisere dela av profili sin på ei lenkje alle kan opne utan konto (totala og topptitla, rekkjer og ka tid på døgne d vert sett, heile åre som historiekort, dei ti siste avspelingane), kvar del slått av til eigari slær han på under Innstillingar → Offentleg profil. Einingar, adresser, appa, avspelingsmåta, filstia og andre persona vert aldri publiserte, samme ka som e slått på, og namne som vert vist, e eit eigari skriv inn, aldri brukarnamne i Jellyfin.
- **Ingenting på ei publisert side e nyare enn eitt døgn**, både nylege avspelingar og alle totala, so ei side nokon helde auge me, seier aldri kven som ser akkurat no.
- **Kort til deling**: kvar publisert profil kjøme me en bilete på 1200×630, av profili og av åre, som Discord, Signal, Mastodon og resti viser når lenkja vert limt inn, og som kan lastast ned og leggjast ut. Teikna på serveri ifrå d sida viser, og ikkje nåke meir.
- **Ei lenkje du kan ta tilbake**: 22 tilfeldige teikn, nullstilt me eitt klikk; den gamle lenkja svarar då nøyaktig som ei som aldri he funnest.
- **Offentlege profila e slått av til en administrator tillèt dei**: éin brytar for heile serveri, ei liste over kven som publiserer ka, og en måte å ta ned en profil på. Publisering, endringar, nullstillingar og nedtakingar står i revisjonsloggi, og en nøkkel kan ikkje gjære nåke offentleg.
- **Saman**, ei eie side: kven som ser me kven, kor mykje, og korleis d he endra sej. Tid saman, kvelda og deli av visingstida i lag me andre, kvar mot tidsromme før; tima i lag mot tima åleine per dag elle veke; kvart par av persona me kveldane, tida saman, når dei sist såg og ka dei ser mest; titlane som oftast vert sett saman; dei siste kveldane; og, for alle som kan sjå alle, tida kvar person ser åleine mot tida i lag me andre.
- Kvart par tel: en kveld me tre tel for kvart av dei tre para, der korte før bære talde den nøyaktige samansetjinga av persona. Tida saman for en par e d kortaste av dei to opphalda.
- **Sett saman**-korte på Oversikt og på en profil lenkjer til sida, og korte på en profil seier kor stor del av visingstida til personi som va i lag me andre.
- `GET /api/stats/groups` får `pairs`, `series`, `people`, `previous`, `bucket`, og `watch_s` og `share` i totalane; nøklane d hadde ifrå før, e uendra.
- **Kor folk stoppar**, på kvar film- og episodeside: deli av avspelingane som framleis helde på ved kvart minutt, ifrå alle ved starti til dei som nådde slutti, me tilbakespolingane og undertekstpåslaga i desse avspelingane merkte på samme aksi. Korte seier kor monge stopp som e målte (avspelingar FinStats registrerte) og kor monge som e rekna ut (importerte avspelingar, ifrå kor lenge dei varte), og treng tre avspelingar før d teiknar nåke.
- **Kven som helde fram**, på ei serieside: kor monge som starta på kvar episode, i rekkjefølgje, og ei **Fullført**-kolonne per episode. «Alle gir opp ette episode tre» e folk som aldri trykkjer play på episode fire.
- **Filer som aldri vert spela av**, under Avspeling: en film elle episode som e starta tre gonga elle meir og aldri kom forbi tretti sekund, me kven som prøvde og i ka appa. Loggi til Jellyfin he ingen avspelingsfeil, so avspelingane e vitne; minste avspelingslengd gjeld ikkje her, sida d e nettopp den desse avspelingane aldri når.
- **Mest tilbakespola** og **Underteksta slått på**, under Avspeling: titla rangerte ette hopp bakover per avspeling, me minutte hoppa samlar sej i, og ette deli av avspelingar der underteksta vart slått på innan dei første ti minutta. Alle tre listene krev *Sjå alle*. Hopp kortare enn 20 sekund vart aldri registrerte, so å høyre éin replikk eingong til e usynleg me vilje.
- Ei spoling tek vare på kor ho kom ifrå, som en tal ved sida av merkelappi (`from_s` i tidslinja til ei avspeling). Kvar spoling som alt e registrert, får tale sitt ifrå merkelappi, og d samme gjær en gjenoppretta tryggingskopi ifrå før dette.
- `GET /api/stats/files` for dei tre listene; `GET /api/items/{id}` får `insights`, og episodane til en serie får `users` og `finished`.
- **API-nøkla**: alle som e innlogga, kan lage nøkla til sej sjølve under Innstillingar → API-nøkla, til en skript, en dashbord elle en telefon. En nøkkel vert send som `Authorization: Bearer fs_…`, vert vist eingong, vert lagra som hash, kan ha en utløpsdato og vert trekt tilbake me eitt klikk; en administrator ser nøklane til alle og kan trekkje dei tilbake. Nøkkeli høyrer til den samme personi me dei samme løyva som ei innlogging, lese på nytt ved kvar førespurnad, so å miste retti til å logge inn stoppar alle nøklane me eingong. En nøkkel kan verken lage elle trekkje tilbake nøkla.
- **En kalender å abonnere på**: `GET /api/calendar.ics?key=…` e d Sonarr og Radarr he på veg, som en iCalendar en telefon kan abonnere på. En nøkkel me omfange *kalender* opnar kalenderi og ingenting anna; kalenderi les nøkkeli ifrå adressa, aldri ifrå informasjonskapseli, fordi en abonnert kalender ikkje kan sende nokon header, og han namngjev ingen.
- **En revisjonslogg** under Server → Revisjonslogg, for Jellyfin-administratora: kvar innlogging og kvart mislukka forsøk me adressa, kvar innstilling elle kvart løyve som vart endra og til ka, kvar nøkkel som vart laga, brukt elle trekt tilbake, kvar tilkopling, mottakar, tryggingskopi og import, kvar avspeling som vart sletta og kvart varsel som vart løyst, me kven, ifrå kor, gjennom ka nøkkel og om d lukkast. Vert teken vare på i eitt år og e me i tryggingskopia. `GET /api/audit` blar gjennom han.
- **Import ifrå Streamystats**, en eie seksjon under Innstillingar ved sida av Jellystat. Ta fila ifrå **Settings → Backup & Import → Download Backup** der og slepp na inn: år me historikk kjøme på en par sekund, og ingenting du alt he, vert talt på nytt. `finstats import-streamystats <file>` gjær d utan grensesnitt, `POST /api/import/streamystats` gjennom API-et; éin import køyrer om gongi, samme ka slag.
- **He du brukt begge sporingsverktøya? Importer begge filene.** Ei avspeling vert kjend att som ei FinStats alt he, fyrst på ID-en sporingsverktøye sjølv gav na, og elles på at samme person ser samme element me ein av endane av avspelinga nær ei i historikki, so en Jellystat-eksport og en Streamystats-eksport av dei samme kveldane kan begge importerast, i ka rekkjefølgje som helst, og en kveld FinStats registrerte sjølv, vert ikkje importert over heller. «Nær» e Samanslåingsvindauge under Innstillingar → Innsamling.
- Importerte rader seier ikkje meir enn dei veit. Ei avspeling Streamystats registrerte sjølv, tek ikkje vare på nåke om fila (ingen kodek, oppløysing, bitrate elle container), so desse felta vert ståande tomme istade for å fyllast me ei gjetting, og korta som vilde vist dei, gøymer sej. Ei avspeling Streamystats hadde henta ifrå Jellystat, he me sej heile økta, og den vert lesen istade: datert ette augneblinki ho slutta, den einaste augneblinki nokon av sporingsverktøya tok vare på, og skildra som omkoding bære når økta verkeleg va d.
- Rader Streamystats skreiv fordi Jellyfin melde en element som sett, for ei sjåing Streamystats aldri såg, vert talde og hoppa over: dei e like lange som heile filmi og vilde vore visingstid en kveld ingen såg. Resultate seier kor monge d va.
- Korleis Streamystats-data vert tolka, felt for felt, lært ifrå en verkeleg eksport og ikkje ifrå dokumentasjon: `docs/streamystats-import.md`.
- **Ka sporingsverktøy kvar avspeling kom ifrå**, som filter under Aktivitet: d FinStats registrerte sjølv, d som kom ifrå Jellystat, d som kom ifrå Streamystats. Bære dei som historikki din faktisk he, vert tilbydde, og filtere e avgrensa som alle andre lister, so nokon som bære kan sjå sine eie avspelingar, får bære vite kor den eie historikki kom ifrå.
- **Filtera under Aktivitet e nedtrekkslister du kryssa av i**, so du kan be om filma *og* episoda utan musikk, to avspelingsmåta elle to persona samstundes, istade for éin av kvar elle alle. Medietype fekk **Anna** no som typane kan kombinerast.
- Fleire persona samstundes, overalt der brukarfiltere finst (Oversikt, Avspeling, Bibliotek, Tryggleik og resti), og karte og all statistikk følgjer me. En brukarfilter snevrar framleis bære inn d nokon kan sjå: utan *Sjå alle* vert en førespurnad låst til den som spør, samme kven adressa namngjev og kor monge.
- **En lyst tema**: washipapir, me blekktekst og en vermilionraud aksentfarge. FinStats følgjer den lyse elle mørke innstillinga på eininga til du vel en tema sjølv.
- **En temaknapp ved sida av Logg ut** går gjennom Eining, Lyst og Mørkt. Vale vert teke vare på i dinna nettlesari og teke i bruk før sida vert teikna, so ho blinkar aldri me d andre temae.
- **Opne i Jellyfin**: kvar film-, serie- og episodeside he en knapp som opnar sida til titteli i Jellyfin, i ei ny fane.
- **Adressa til Jellyfin for folk** under Innstillingar → Jellyfin: dit desse knappane peikar, for når adressa FinStats koplar sej til, ikkje e ei som en nettlesar kan nå. Tomt felt helde på adressa FinStats koplar sej til; bære en Jellyfin-administrator kan setje na.

### Changed
- **FinStats he fått en ny heim**: `github.com/finstats/finstats`, og Docker-bilete e `ghcr.io/finstats/finstats`. Endre namne på bilete i `docker run` elle compose-fila di for å helde fram me å få oppdateringar; datamappa e som ho e.
- **Innstillingar viser éin seksjon på skjermi om gongi.** Sida va fjorten kort i éi kolonne og seks hundre ord me hjelpetekst. No he kvar seksjon si eie adresse (`/settings/collection`), ei liste til venstre flyttar myllå dei, og på en telefon bryt lista om til brikker. Lenkjer til dei gamle ankera kjøme framleis fram.
- **Ei innstilling e éi rad**: ka ho e, éi line som seier regeli, og kontrolli til høgre. Tal i en seksjon vert lagra saman me éin knapp nedst; brytara lagrar sej sjølve som før. Statuskorta (Oppgåver, Utgåande tilkoplingar, Database) og lenkja til lisensane ligge under **System**, og dei to importverktøya he en eie seksjon.
- **Finn ei innstilling** ifrå søkjefelte øvst (`/` set fokus på d) me ka ord som helst i namne; d seier ka seksjon innstillinga ligge i, og Enter tek dej til rada.
- **Innstillingar e for alle som e innlogga**: seksjonane som styrer serveri, e framleis for dei som får styre han, og en person som bære hadde løyve til varsel, kan endeleg nå sine eie mottakara.
- **Server-sida viser éin seksjon på skjermi om gongi**, sann Innstillingar gjær: Oversikt (system og lagring), Jobba, Einingar, Programtillegg, Logg og Revisjonslogg, kvar på si eie adresse. Lenkjer til `/server#jobs` kjøme framleis fram.
- **Serverloggi e en seksjon på Server-sida** (`/server/log`) istade for ei eie side i menyen; `/events` sender vidare dit me filtera sine.
- En varsel om en jobb som feila, en tryggingskopi som feila elle ei tilkopling som slutta å svare, lenkjer til den innstillingsseksjoni d gjeld, istade for til toppi av sida.
- D biblioteke ditt kan fortelje om ei importert avspeling (ka slags ting d va, ka episode, speletida, ka bibliotek ho høyrer til), vert fylt ut ette **kvar** lesing av biblioteke, ikkje bære i den augneblinki en import skjer. Historikk vert somregel importert før FinStats nokon gong he lese biblioteke, so desse avspelingane vart før verande gjetta elle ukjende for godt.
- Å gjenopprette en FinStats-tryggingskopi legg ikkje lenger til ei avspeling som kom ifrå en sporingsverktøy under en anna ID, og helde framleis på to ekte sjåingar av samme ting me nokren minutt imyllå.
- Detaljane til ei avspeling namngjev d sporingsverktøye som henta na inn, istade for å kalle alt som e importert, ei Jellystat-avspeling.

### Performance
- **En omstart går nesten me eingong, samme kor stor historikki e.** Kvar oppstart rekna ut kven som såg saman over heile historikki og avgjorde på nytt ka avspelingar som va lokale, bære for å ikkje endre nåke: 24 s og 5,5 s på ti milliona avspelingar. No gjær en oppstart bære d den førre køyringa lét att, og alt på nytt bære ette ei oppdatering elle når innstillinga bak e endra.
- **Monge som starta samstundes, vert registrerte på sekund.** Kvar ny avspeling, kvar lagring av framdrift og kvar eining vart skriven kvar for sej, og kvar avspeling som slutta, vart lukka me sin eie gjennomgang av heile historikki til titteli; en runde skriv no kvart slag samla. 100 000 avspelingar som starta samstundes, tek 3 s, der 25 000 som slutta samstundes, før tok over fire minutt.
- **Spelast no helde sej rask me tusenvis som ser.** Å finne kven som ser saman, samanlikna kvar straum me alle andre og lista kvar følgjesvein me namn: førti sekund ved 10 000 strauma, og gigabyte me minne i tillegg. No vert strauma samanlikna innanfor samme tittel, og kvar viser nokren få namn og kor monge andre.
- Å rekne ut kven som såg saman, skreiv om kvar gruppert avspeling i historikki kvar gong FinStats starta, importerte elle såg ei avspeling slutte; no skriv d bære d som e endra. Å finne ka avspelingar som he slutta, vert ikkje lenger tregare me kvadrate av tale som spelar.
- **Å kjenne att ei avspeling historikki alt he, går gjennom en indeks**, so en import av 150 000 avspelingar tek sekund, og kopling på nytt ette ei lesing av biblioteke går ifrå tittel til tittel istade for å lese kvar avspeling menst databasi e låst.
- **Innsikt i avspeling og oversikta e raskare over heile tida**: dei uavhengige delane vert rekna ut side om side, diagramme for samtidige strauma slær opp den lokale dagi eingong per kvarter istade for eingong per avspeling, og sjangra vert talde per tittel. På ein million avspelingar: ifrå 4,1 s til 2,4 s, og ifrå 1,6 s til 1,1 s.

### Stability
- **En TV-bibliotek vert ikkje lenger lese som en bibliotek som e tømt.** Lesing av en bibliotek går i to runda (alle element, so en mindre runde bære for medverkande i filma og seria), og kontrolli som avgjer om ei lesing e til å stole på når ho ska fjerne d ho ikkje såg, såg på den andre runden. I en bibliotek me TV elle musikk e dei to tala langt ifrå kvarandre, so kontrolli avviste kvar lesing og stoppa FinStats, og installasjoni starta på nytt og gjorde d samme igjen. Ingenting vart nokon gong feilaktig fjerna, men inga lesing av biblioteke kunne fullførast. Dersom serie- elle musikkbiblioteke ditt hekje fått me sej nye titla, e dette grunni.
- En import kunne feile me «database is locked» når ei anna skriving (ei avspeling som vart oppfriska) kom myllå den første lesinga og den første skrivinga. Begge importverktøya tek skrivelåsi me eingong dei starta, og journali til databasi helde sej ikkje lenger like stor som den største importi som nokon gong e gjord.
- **Ei databasefil som e skadd, avkorta, nokon annan sin elle skriveverna, vert avvist me eingong**, me namne på fila og beskjed om at ingenting vart endra, istade for 30 stille sekund; databasi til en anna program vert aldri skriven i.
- En tryggingskopi elle en kopi før oppdatering som vert broten av halvvegs, lèt ikkje lenger ei fil liggje att, ei fil som kunne ta plassi til en god kopi blant dei tre som vert tekne vare på.
- **Å skreve over staddatabasi menst FinStats køyrer, får ikkje lenger FinStats til å krasje**: FinStats les ifrå en privat kopi av fila.
- En Jellyfin-app som rapporterte en ny eining-ID kvar gong, kunne få minne til FinStats til å vekse so lenge programme køyrde; einingar som ekje sett på fem minutt, vert no gløymde.
- Éin person he høgst 30 innlogga økter; den eldste vert logga ut ved neste innlogging.

### Fixed
- **Den samme kveldi vert ikkje lenger talt to gonga når en element he fått nytt namn.** Å kople historikk på nytt til en tittel som va lagd til på nytt i Jellyfin, kunne gjære ei avspeling importert ifrå en sporingsverktøy til en duplikat av ei som alt va der, fordi kontrolli som vilde ha fanga d, køyrde før ID-en flytta sej. No køyrer han igjen overalt der ID-ar vert kopla på nytt. Bære importerte avspelingar vert nokon gong fjerna, aldri ei FinStats registrerte sjølv, og kor monge som forsvann, står i loggi. Den første oppstarti av 2.0 går gjennom heile historikki eingong for dei som alt e der.
- **Ei økt følgjer no personi sann Jellyfin he han no.** En administrator som Jellyfin tok administratorrettane ifrå, heldt på full tilgang til FinStats resti av den 30 daga lange økta si, og nokon som va deaktivert elle sletta i Jellyfin, vart verande innlogga. No mistar dei tilgangi so snart FinStats les brukarane i Jellyfin (kvart 15. minutt, elle me eingong ifrå Innstillingar), sann d alltid he vore me API-nøkla.
- Innlogging skriv d Jellyfin seier om personi, inni brukarlista til FinStats, so en importert tryggingskopi ifrå en sporingsverktøy kan ikkje lenger gjære nokon til administrator før den første lesinga av brukarane rettar d.
- Nokon som e sletta i Jellyfin, vert merkt som fjerna sjølv når to lesingar av brukarane fell i samme sekund.
- **Nedlastingsfeil viser aldri passorde til en nedlastingsklient elle API-nøkkeli til en indekserar**, som Sonarr og Radarr kan sitere i feilmeldingane.
- Avspelingar som vart kopla på nytt til en tittel me nytt namn under ei lesing av biblioteke, vart ikkje sjekka for å være sett saman før neste omstart; no vert dei d, me eingong.

## [1.6.5] - 2026-09-24

### Added
- **En full tryggingskopi vert teken automatisk før kvar oppgradering.** Første gongi en nyare FinStats opnar databasi din, skriv han en fullstendig kopi av databasi (heile historikki, innstillingane og alt) til `data/pre-update-backups/` *før* nåke vert endra. Dersom ei oppgradering nokon gong øydelegg nåke, e dataa dine trygge: stopp FinStats, legg kopien tilbake istade for `finstats.db`, start versjoni du va på, og meld ifrå om feili. Ingenting går tapt. Dei nyaste kopiane vert tekne vare på; set `FINSTATS_SKIP_PREUPDATE_BACKUP=1` for å slå d av dersom d e lite diskplass.

## [1.6.4] - 2026-09-24

### Added
- **Fem nye stada å sende varsel til**: Telegram, Slack, Pushover, Pushbullet og e-post, ved sida av mottakarane webhook, Discord, ntfy og Gotify som fanst ifrå før. Kvar ber bære om d han treng (en chat-ID, en brukarnøkkel, ei postkasse), og dei tre som helde til hos éi teneste (Telegram, Pushover, Pushbullet), kan bære nåast der, so en token kan ikkje postast til en vert som bære liknar. E-post går ut over TLS ifrå første byte elle me `STARTTLS`, og d finst ikkje nåke tredje val.
- **Lisensa**, under **Innstillingar**: GPL-lisensi til FinStats sjølv og kvar tredjepartslisens programme byggjer på (253 Rust-crates, dei to medfølgjande skrifttypane, kartomrissa og geolokaliseringsdatabasi), kvar tekst i heilskap, sann prosjekte sjølv skreiv han. Vel en komponent, so opnar lisensi sej i vindauge ved sida av lista; en crate som he fleire filer, byter myllå dei på samme stadi. Lista vert laga ut ifrå dei cratene FinStats faktisk e bygd av, so en avhengnad kan ikkje leggjast til utan at lisensi hans vert registrert.

## [1.6.3] - 2026-09-24

### Fixed
- Verne slær no òg inn når Jellyfin listar *ingen* bibliotek elle *ingen* brukara i d heile (ei øydelagd lesing, aldri en vanleg dag), nåke justeringa i 1.6.2 hadde sleppt gjennom. Å miste nokren bibliotek vert framleis rekna som vanleg; å miste alle på eingong vert d ikkje.

## [1.6.2] - 2026-09-24

### Fixed
- Verne for dataintegritet som kom i 1.6.1, va for ivrig: en bibliotek som verkeleg mista fleste elementa sine (til dømes ei handfull klipp der filene forsvann), vart teke for ei øydelagd lesing, so FinStats nekta å oppdatere d og stoppa. No slær verne bære inn ved ei *tydeleg* øydelagd lesing: en bibliotek som e stort nok til å bety nåke og kjøme heilt tomt tilbake, elle en stort bibliotek som e tømt til nesten ingenting. Vanlege endringar vert tekne i bruk som før, og tilfelle «Jellyfin returnerte ingenting» vert framleis fanga.

## [1.6.1] - 2026-09-24

### Added
- **FinStats stolar ikkje lenger på ei lesing ifrå Jellyfin som vilde slette historikki din.** Dersom en bibliotek som FinStats he tusenvis av element for, plutseleg kjøme tomt elle nesten tomt tilbake (ei Jellyfin-oppgradering som endra API-et, en feil forkledd som en tomt resultat), helde FinStats på dataa sine, merkjer synkroniseringa som feila (og varslar), og stoppar på en ryddig måte me ei klar melding istade for å merkje alt som fjerna. Start på nytt når Jellyfin e sej sjølv igjen, elle set `FINSTATS_ALLOW_LIBRARY_SHRINK=1` dersom du verkeleg tømde en bibliotek. D samme verne dekkjer bibliotekslista og brukarlista.

### Fixed
- **Tryggleik:** en `X-Forwarded-Host`-header sendt av en klient kunne gå god for en framand `Origin` og få ei skriving på tvers av nettstada forbi verne for samme opphav. Den headeri vert no bære stolt på bak en klarert proxy (`FINSTATS_TRUST_PROXY`), samme regel som oppslage av klientadressa alt følgjer. Øktinformasjonskapseli me `SameSite=Lax` va, og e framleis, d første forsvare.
- **Tryggleik:** en personleg varselmottakar kunne peikast mot `0.0.0.0` elle `::`, som når dinna maskina. Kontrolli som helde personlege mottakara retta mot offentlege adresser, reknar no desse som lokale, saman me loopback og dei private adresseområda.
- **Avspeling**-sida lastar no òg for en bibliotek som inneheld ei avspeling utan registrert avspelingsmåte (ei rad gjenoppretta ifrå en tryggingskopi som e eldre enn kolonna); ei sann avspeling tel bære som direkteavspeling.
- En svært langt ord i en søk elle en filter gir ikkje lenger serverfeil: SQLite avviser en `LIKE`-mønster på over 50 000 teikn, so filterord vert korta ned til ei fornuftig lengd før spørjinga.
- **Innstillingar**-sida rullar ikkje lenger sidelengs på en telefon når en kort melder tilbake ei heil setning, til dømes resultate av ei gjenoppretting.

## [1.6.0] - 2026-09-24

Varsel

### Added
- **Varsel**-kort under Innstillingar: legg til en mottakar, kryss av for ka han ska få vite om, og trykk Test. Ingen mottakar tyde at d ikkje vert sendt nokon førespurnad til nåke.
- Fire slag mottakara: en vanleg **webhook** (JSON, dokumentert i `docs/api.md`, valfri `Authorization`-header), **Discord** (en embed gjennom ei webhook-nettadresse for en kanal), **ntfy** (ntfy.sh elle din eie, til en emne) og **Gotify** (din eie server, me en programtoken). Titla vert sende i meldinga og ikkje i en header.
- Elleve slag hendingar, kvar me sitt eie kryss: umogleg reise, en konto sett i en nytt land, ei bølgje av mislukka innloggingar, ei oppgåve som feila, en tryggingskopi som ikkje kunne skrivast, nåke som slutta å svare (Jellyfin, Sonarr, Radarr elle Seerr) og når d svarar igjen, nye titla i biblioteke (samla per serie og dag, sann hylla på Oversikt samlar dei), en førespurnad som vart klar til å sjåast, og ei avspeling som starta elle sluttar. Dei to siste e av som standard.
- En førespurnad som vart klar til å sjåast, namngjev titteli, kven som bad om han, og kor lenge dei venta.
- Personlege mottakara: gje **Varsle**, so kan nokon leggje til sin eie mottakar, som bære får d personi alt kan sjå: eie førespurnada og varsel alltid, d som gjeld andre me *Sjå alle*, adresser me *Sjå nettverk*. En personleg mottakar må peike på ei offentleg adresse; bære en administrator kan rette ein inni nettverke.
- **Ta me IP-adresser og stada**, per mottakar og av som standard: ei melding om umogleg reise namngjev elles bære stada («Oslo, Noreg og London, Storbritannia»).
- **Nyleg sendt**, i samme korte: ka som vart sagt, til ka mottakar, og ka den andre sida svarte når meldinga ikkje kom fram. Ei melding som feila, vert prøvd på nytt fire gonga over om lag en time, og en mottakar som avgrensar kor mykje han tek imot, vert venta på.
- **Adressa til FinStats**, ei innstilling for administratora, e d lenkja i en varsel vert bygd ifrå. Tomt felt tyde at meldingane ikkje he nåke lenkje.
- Dei planlagde jobbane til Jellyfin på Server-sida: ka kvar av dei gjær, i ord istade for namne på koden, og om han e ein av dei tunge. En jobb som køyrer, viser ei framdriftslinje, en prosent og kor mykje som e att; dei andre viser når dei køyrer neste gong, når dei køyrde sist og om d gjekk bra. Jobba Jellyfin gøymer ifrå sitt eie kontrollpanel, e me, og ingenting i FinStats kan starte elle stoppe en jobb.
- Kor mykje som e att av en jobb, vert målt ut ifrå farti prosenti faktisk he flytta sej me, mot den nyaste avlesinga som ligge langt nok tilbake til å seie nåke, istade for snitte over heile køyringa, so en jobb som går raskare elle saktare, vert skildra ut ifrå tempoe han he no.
- Jobblistene FinStats alt les av andre grunna (sjekki av biblioteksskanning kvart 5. minutt, serverdetaljane kvart 15.), gir data til samme målinga, so en jobb som he køyrt i en time, he somregel en overslag me eingong sida opnar sej. Jellyfin vert ikkje spurt om nåke ekstra for dette.
- En jobb som køyrer utan at nåke e målt endå, viser **ETA** og prikka som går rundt istade for en tal, og ein der prosenti ikkje he flytta sej på en halvt minutt, viser «68 % i 4 min» ved sida av. Ingenting står istade for en overslag som ikkje e målt.
- Ei liste over alle versjona ved sida av endringsloggi: eitt klikk tek dej til ka utgjeving som helst, lista markerer den du les, og på en smal skjerm vert ho ei rad me versjonsseria over notata.

### Changed
- Endringsloggi fell ikkje lenger saman. Kvar versjonsserie står open, og versjonslista e måti du flyttar dej på, istade for éi gruppe open om gongi og en klikk for å nå kvar av dei andre.
- **Innstillingar → Utgåande tilkoplingar** listar kvar varselmottakar ved sida av Jellyfin og tenestetilkoplingane: ka han e, verti, om han e slått på, og når han sist tok imot ei melding.
- Ingenting ved innsamlinga, historikki elle nåke tal på skjermi endrar sej. FinStats *les* framleis bære ifrå Jellyfin, Sonarr, Radarr og Seerr.
## [1.5.2] - 2026-09-22

### Changed
- Ei lesing av øktlista i Jellyfin og nullstillinga av klokka som bad om na e no eitt og samme kall, so dei to kan ikkje lenger kome ut av takt. D va forma på feili me løpske førespurnada som vart funnen menst 1.5.0 vart bygd.
- Å prove d tek no en brøkdel av en sekund istade for sju minutt mot en ekte server, so d vert sjekka kvar gong nåke vert endra. Dei samme førespurnadene, den samme historikki, dei samme tala på skjermi.

## [1.5.1] - 2026-09-22

### Fixed
- Ei omkoding vert registrert eingong, ikkje eingong i sekunde. Når ei avspeling starta me omkoding og går tilbake til direkteavspeling, lèt Jellyfin omkodingsdetaljane liggje att på økta; kvar avlesing vart samanlikna me en verdi FinStats nettopp hadde skreve over, so kvar avlesing såg ut som ei endring, og Aktivitet-sida fylte sej me «Omkoding»-linjer me «Direkteavspeling: …» under.
- Linjer som alt e skrivne, vert rydda opp ved oppstart: der fleire ette kvarandre seier d samme, vert bære den første behalden. Ei avspeling som trong omkoding, tel framleis som ei omkoding.

## [1.5.0] - 2026-09-22

Øktsporing over WebSocket

### Changed
- FinStats helde éi tilkopling open til Jellyfin ifrå d starta, og d e sann d får vite om ei avspeling. 1.4.0 kom me dette som ei innstilling, av som standard, for at d skulle prove sej mot ekte servera; no e d alltid på. Dei samme avspelingane, pausane og hoppa, rekna ut av den samme koden.
- Ingenting vert spela av: FinStats spør ikkje Jellyfin om nåke i d heile. En kveld då ingen ser på, kosta ingen førespurnada.
- Nåke vert spela av: øktlista vert lesen eingong i sekunde. D e d som gir en pause, ei spoling elle en sporbyte skarpleiki, og d e d som avsluttar ei avspeling der klienti forsvann utan å seie farvel.
- Alt som e lasta, he stått på pause i tre avlesingar på rad: spørjinga stoppar, og tilkoplinga tek over, der en film på pause før kosta d samme spørsmåle kvart sekund, rundt 16 MB i timen. Helde nokon fram, elle starta nokon andre nåke, vert d fanga opp innan om lag eitt sekund, og tid på pause tel framleis aldri som visingstid.
- Kvar utgåande lesing ber om komprimerte svar og pakkar dei ut (Jellyfin, Sonarr, Radarr og Seerr komprimerer alle når ein ber om d), og d kuttar meir enn halvparti av storleiki på dei to største lesingane: øktlista under ei avspeling, og lesinga av biblioteke. Geolokaliseringsdatabasi vert framleis lasta ned sann han e lagra.
- Jellyfin-korte i Innstillingar seier ka for ein av dei to halvdelane som køyrer, og direkteprikki følgjer om tilkoplinga faktisk fører data, ikkje ka halvdel du e i.
- Bære éin av dei to køyrer om gongi: menst FinStats spør, ber d Jellyfin slutte å sende, og ber Jellyfin starta igjen samme augneblinki den siste avspelinga sluttar. Tilkoplinga står open heile tida.
- En Jellyfin-server som svarar på abonnemente me ingenting, vert trudd so lenge han svarar på livsteikn: FinStats seier ifrå om d eingong, les øktlista sjølv i mellomtida og helde på abonnemente til den første sendinga avgjer saka. «Kan ikkje sende» og «heldt på å starte på nytt då vi spurde» ser like ut, so stilla vert aldri teken som en svar.
- Reserveløysinga e urørt: ei tilkopling som vert lukka, vert stille elle viser sej ikkje å snakke dette, fell tilbake til å spørje på tidtakar alt ved neste runde og helde fram me å kople til på nytt, me intervalla **Menst nokon ser på, sjekk kvart** og **Menst ingenting vert spela av, sjekk kvart**.

### Added
- Ka innsamlari helde på me, på `GET /api/status`: ka for ein av dei fire tilstandane han e i (`idle_socket`, `playing_poll`, `paused_socket`, `fallback`), om tilkoplinga e open og abonnerer, kor ofte han spør, kor monge øktlesingar som gjekk ut d siste minutte, og når dette sist endra sej. Svare kjøme ifrå minne (ingen førespurnad, inga databasespørjing), og d inneheld ingen namn, titla elle tal.
- Desse felta vert kontrollerte mot kvarandre ved kvar runde, og ei usemje som varer, vert skriven til loggi.
- Éi sperre under kvar øktlesing i prosessi: høgst 2 i sekunde og 70 i minutte; alt over d vert utsett, talt og logga høgst eingong i minutte. Vanleg sjåing helde sej godt under.

### Fixed
- Direkteoppdateringane erstattar no verkeleg spørjinga. Me innstillinga slått på spurde 1.4.0 framleis ette øktlista kvart femte sekund og opna tilkoplinga på nytt kvart tjuande til førtiande sekund, fordi tilkoplinga vart vurdert ette kor nyleg ei liste hadde kome, og Jellyfin sender ei når nåke *endrar sej* og ingenting i mellomtida. Om tilkoplinga lever, vert no avgjort av om Jellyfin svarar i d heile, sjekka me en livsteikn om lag to gonga i minutte.
- Ei øktliste som kom menst FinStats va myllå to runda, vart kasta, ut ifrå at ei ny lå halvanna sekund bak. Ho vert no teken vare på og brukt.
- **Sist sjekka** under Innstillingar og korte Utgåande tilkoplingar vert ikkje lenger utdaterte menst tilkoplinga e stille: d e svare ifrå Jellyfin på en livsteikn dei viser.

### Removed
- Innstillinga **La Jellyfin sende d som vert spela av** under Innstillingar → Innsamling, og `live_socket` ifrå innstillings-API-et.
- `socket_enabled` ifrå innsamlarstatusi: d finst ikkje lenger nåke «slått av».
- `last_reconcile_at` ifrå innsamlarstatusi, erstatta av `socket_live`.

## [1.4.1] - 2026-09-22

### Changed
- Seerr vert bære lesen i heilskap når nåke he endra sej: kvart femte minutt ber FinStats om éi rad, den sist endra førespurnadi, og runden sluttar der dersom FinStats alt kjenner til han. D e om lag en kilobyte mot rundt seksti for ei side på femti. Førespurnada som framleis e på veg, vert kontrollerte kvart kvarter, alt vert lista opp eingong i døgne, og en ny førespurnad dukkar framleis opp innan fem minutt.
- Køane i Sonarr og Radarr vert lesne kvart femte sekund menst ei side viser dei, kvart minutt menst nåke ligge i køen og kvart femte minutt menst han e tom, istade for kvart minutt heile døgne. Når sida vert opna, ei teneste vert kopla til elle d kjøme en ny førespurnad i Seerr, vert dei oppdaterte me eingong.

## [1.4.0] - 2026-09-21

Valfrie WebSocket-økter, og alt FinStats kan nå

### Added
- **La Jellyfin sende d som vert spela av** under Innstillingar → Innsamling, av til du slær d på: éi tilkopling ber øktlista, og om lag 17 000 førespurnada i døgne vert til nesten ingen. Avspelingane, pausane, hoppa og gruppene vert rekna ut av nøyaktig den samme koden som før. Menst ho e aktiv, sender FinStats framleis éin vanleg førespurnad i minutte *menst nåke vert spela av*, og ingen når ingenting vert d. Ei tilkopling som vert lukka, e stille i femten sekund elle viser sej ikkje å snakke dette, fell tilbake til tidtakari ved neste runde og helde fram me å kople til på nytt.
- **Utgåande tilkoplingar**, en kort i Innstillingar for Jellyfin-administratora: kvart mål FinStats kan nå (Jellyfin-serveri din, «ka e IP-en min»-tenesta, databasi til DB-IP, og kvar Sonarr, Radarr elle Seerr), me ka d e til, om d e slått på, og når d sist svarte. Korte vert bygt av d FinStats alt veit, og viser bære adresser: aldri en nøkkel, aldri en basesti.
- **Slå opp no** under Innstillingar → Heimenettverk, for dagi den offentlege adressa di endrar sej.

### Changed
- Oppslage «ka e IP-en min» køyrer eingong, første gongi FinStats treng ei adresse, istade for kvart kvarter.
- Jellyfin-korte i Innstillingar seier korleis innsamlari får vite ting, og, når tilkoplinga e slått på men ikkje fører data, kvifor ikkje.

## [1.3.0] - 2026-09-20

Sonarr, Radarr og Seerr, på éi side

### Added
- **Tilkoplingar** under Innstillingar, for Jellyfin-administratora: **Sonarr**, **Radarr** og **Seerr** (elle Jellyseerr/Overseerr), fleire av kvart slag, kvar av dei testa før ho vert lagra. Nedlastingsklienti din treng ikkje nåke eie oppsett, fordi Sonarr og Radarr alt snakkar me han, og FinStats les d dei veit.
- **Førespurnada**: kven som bad om ka, kor lang tid d tok før d kom, og om dei nokon gong såg d. Ruter for titla som venta og den vanlege ventetida, en diagram over korleis ho he endra sej månad for månad, og lister over d som kom for fleire veker sida og aldri vart spela av, og over kven som spør om mest.
- **Kjøme**: nye episoda og filmutgjevingar ifrå Sonarr og Radarr som ei oversikt dag for dag, merkte me om *du* ser den serien og, for dei som får sjå alle, kven andre som gjær d. Nokon vert rekna for å sjå en serie når dei he spela av en episode av han dei siste fire månadene. Ei «Kjøme snart»-rad på Oversikt, éi per profil for seriane den personi ser, og ka som e d neste for en tittel, på sida til titteli.
- **Nedlastingar**: den direkte køen i kvar Sonarr og Radarr (ka d e, kor langt d he kome, kor fort d går, ka som gjekk gale ved import), me personi som bad om d ved sida. Ei sesongpakke e éi linje samme kor monge episoda ho helde, og d som treng merksemd, ligge øvst. Under d: ka som kom inn over ei veke, en månad elle en år ette indeksera, kvalitet og nedlastingsklient, og ka nedlastingar som feila. Oversikt viser den samme lista, forkorta.
- **Sjå nedlastingar**, en nytt løyve. Utan d ser folk framleis kor langt deira *eie* førespurnad he kome og kor lang tid som står att, og ingenting anna: ingen utgjevingsnamn, ingen fart, ingen andre nedlastingar. Førespurnadene til andre krev *Sjå alle*.

### Changed
- FinStats sender `GET` til desse tenestene og ingenting anna: d finst ingen kode i FinStats som kan godkjenne en førespurnad, starte en søk elle leggje til, setje på pause elle fjerne ei nedlasting.
- API-nøklane deira vert lagra i den eie databasen til FinStats, vert aldri sende tilbake til nettlesari, aldri skrivne til en logg og e aldri me i en tryggingskopi. Ingen omdirigering vert følgd, so en nøkkel kan ikkje hamne en stad du ikkje skreiv inn, og sertifikat vert kontrollerte me mindre du slær d av for éi tilkopling.

## [1.2.2] - 2026-09-20

### Added
- Lyd- og undertekstspråk på kvar film og episode («Lyd: japansk · engelsk»).
- Kor langt en dubbing går: en serie elle en sesong seier kor monge av episodane som he kvart språk («Engelsk: 13 av 26 episoda»), og episodelista he ei **Lyd**-kolonne.
- **Lydspråk** og **Undertekstspråk** på bibliotekssidene: kor monge filer som kan spelast av på kvart språk.
- FinStats les biblioteke på nytt eingong ette dinna oppdateringa for å hente språka. D seier aldri «dubba», fordi Jellyfin ikkje oppgjev originalspråke til en tittel, og en spor utan språkmerke vert lista som «Ukjend».

## [1.2.1] - 2026-09-20

### Changed
- Bileta i README-en e tekne på nytt i dinna versjoni, me eitt nytt av **Tryggleik**-sida. Ingenting i appi endrar sej.

## [1.2.0] - 2026-09-20

Kor folk ser ifrå, på en kart

### Added
- **Tryggleik-sida**, for dei som får sjå nettverksdetalja og aktiviteti til alle: en verdskart me en prikk for kvar stad brukarane dine ser ifrå, me storleik ette kor mykje som skjer der. Heime vert vist i grønt, stada borte i lilla, pågåande strauma pulserer, og mislukka innloggingar utanfrå vert raude. Dra for å flytte, zoom me knappane, tastane elle Ctrl + hjul, og prikka som overlappar, vert slegne saman til du zoomar inn. Under karte: kvar stad me personane, avspelingane, visingstida og innloggingane, og landa ette tal på avspelingar.
- **Umogleg reise**: en varsel når éin konto dukkar opp på to stada inga flyrute bind saman, elle på to stada langt ifrå kvarandre samstundes, me begge observasjonane, avstandi, tida myllå dei og farti d vilde ha kravd. **Vis på kart** teiknar reisa.
- **Nytt land**: første gongi nokon spelar av elle loggar inn ifrå en land FinStats hekje sett dei i før.
- Løys en varsel me en notat, elle demp en par stada for den personi, for en VPN elle en telefon der operatøri helde til i hovudstadi. Varsel som vert funne i gammal historikk, ette en import elle i en fersk database, vert arkiverte som løyste istade for å fløyme over lista. Varsla og ka du avgjorde om dei, e me i tryggingskopiane.
- Stadene kjøme ifrå en bydatabase som vert lesen lokalt: ingen adresse vert nokon gong send nokon stad, og karte vert teikna ifrå omriss som følgjer me FinStats istade for ifrå ei karteneste. Last ned den gratis databasen til DB-IP ifrå Tryggleik-sida elle **Innstillingar → Tryggleik**, la FinStats fornye han kvar månad (av til du slær d på), elle legg di eie `.mmdb`-fil (DB-IP, MaxMind GeoLite2-City) i `data/geoip/`. `FINSTATS_GEOIP_DB` peikar på ei fil en anna stad.
- **Innstillingar → Tryggleik**: databasen som e i bruk, den månadlege oppdateringa, og kor fort (900 km/h) og kor langt ifrå kvarandre (500 km) to observasjona må være for å telje som umogleg reise. Bydatabasa bommar ofte me nokren hundre kilometer, og d e d avstandi e til for.

## [1.1.3] - 2026-09-20

### Added
- Ctrl+Space søkjer òg i dei medverkande i alt i biblioteke ditt, under **Medverkande**, me bilete, om dei spelar elle regisserer, og kor monge av titlane dine dei e me i; vel du ein, opnar sida deira. Like tilgjevande som tittelsøke (ord i ka rekkjefølgje som helst, aksenta vert ignorerte, en feiltast e lov), og når fleire treffer like godt, kjøme den som e me i flest av titlane dine, fyrst.

## [1.1.2] - 2026-09-20

### Added
- **Nyleg lagt til** på Oversikt, over Aktivitet-diagramme: dei 30 siste tilskota som ei rad me plakata du kan rulle sidelengs, me piler på ei datamaskin. Kvar plakat seier når titteli kom og ka han e (en film, en album elle «Sesong 4 · 3 episoda»), og en enkelt ny episode seier ka for ein, me plakati til sesongi når sesongi he ein.
- **Shift + musehjul** og piltastane flyttar éin plakat, Page Up og Page Down en skjerm, Home og End går til kvar sin ende, og sveiping fungerer. D vanlege hjule rullar framleis sida.
- Nye episoda av samme serie som kjøme samme dag, deler éi oppføring, og en heil serie som vert lagd til på eingong, e éi oppføring («3 sesonga · 60 episoda»), so éin stor import skubbar ikkje alt anna ut av lista.
- Rada bryr sej ikkje om tidsromme elle personi på Oversikt, sida ho handlar om biblioteke og ikkje om avspelingar, og ho gøymer sej menst biblioteke e tomt.

## [1.1.1] - 2026-09-20

### Added
- Sider opnar me eingong: en augneblink ette at FinStats e lasta, hentar d i d stille d du truleg opnar neste gong (Oversikt, profili og tidslinja di, Aktivitet, Brukara, Bibliotek og kvart bibliotek, Avspeling, serversidene, tilbakeblikke og dei mest aktive personane), so ei side syner sej straks når du opnar na, og ho vert likevel oppdatert i bakgrunni.
- Filma, seria og medverkande vert bære henta når du viser at du vil opne dei, sida d finst tusenvis av dei: når peikari kviler på ei lenkje, når du rører ved na elle når du når na me tastature, vert den eine sida henta.
- D venta til sida du opna, e ferdig lasta, tek éi side om gongi menst nettlesari e ledig, tek pause i ei fane i bakgrunni, gjær ingenting på ei datasparande elle svært treg tilkopling, og vert gløymt når du loggar ut.
- Ei lita **Repo**-lenkje me GitHub-merke i statuslinja, ved sida av versjoni.

### Fixed
- **Tidslinje** gløymde ka bibliotek som va slått av når sida vart lasta på nytt elle opna ifrå en bokmerke, sjølv om vale stod i adressa.

## [1.1.0] - 2026-09-20

Ei tidslinje over alt du he sett

### Added
- **Tidslinje**, ved sida av **Oversikt** på kvar profil: sjåinga di som éin sti ifrå i dag og tilbake til den første avspelinga FinStats kjenner til, der en kveld me episoda e falda saman til eitt stopp. Kvart stopp e en plakat me ka som vart sett og når («Sesong 2 · Episode 3–6», en film og om han tok meir enn éi økt, elle en album og kor monge spor), og opnar titteli.
- Stien følgjer breidda på vindauge: tre stopp i rada på en brei skjerm, to på en smalare, kvar andre rad går baklengs me en boge som bind na til den neste, ned til éi rett linje på en telefon. D første stoppe i kvar månad he namne på månadi.
- Eldre stopp vert lasta av sej sjølv når du rullar, heilt tilbake til der historikki starta.
- **Bibliotek**-brytarane over stien utelèt d du ikkje vil sjå, til dømes musikk, og vale e ein del av adressa.
- Nåke som framleis vert spela av, seier **Spelast no**.
- Du ser di eie tidslinje; tidslinjene til andre krev løyve «Sjå alle».

## [1.0.5] - 2026-09-20

### Added
- Vern mot nedgradering: databasen huska den nyaste FinStats-versjoni som he opna han, og en eldre versjon nektar å starte på han istade for å arbeide i d stille me data han ikkje folt ut forstår. Meldinga seier ka du ska gjære. Versjona tillåme 1.0.4 kom ut før dinna kontrolli fanst.

## [1.0.4] - 2026-09-20

### Fixed
- I detaljane til ei avspeling hadde kopiknappi ved en lang **Einings-ID** falle ned på ei eie linje.
- På Avspeling-sida vart den siste kolonnen i **Ka klienta kodar om** kutta («Omko…»); overskrifter på to ord bryt no over to linjer.

### Changed
- Skjermbileta i README-en e oppdaterte igjen; dei viste framleis versjon 0.5.0.

## [1.0.3] - 2026-09-20

### Added
- Ei episodeside viser bilete til den episoden, d Jellyfin viser i episodelista si, istade for plakati til serien. En episode Jellyfin hekje nåke bilete av, helde på plakati.

## [1.0.2] - 2026-09-20

### Fixed
- I **Endringslogg** hadde gruppene v0.9 og v0.10 inga overskrift ved sida av versjoni. Overskrifta til ei gruppe e no alltid den første setninga om ka den første utgjevinga i gruppa handla om, so ei lang innleiing vert ikkje kutta midt i ei setning.

## [1.0.1] - 2026-09-20

### Fixed
- D publiserte Docker-bilete vilde ikkje starte på ei ny maskin: «unable to open database file: /data/finstats.db», ette å ha hengt i en halvt minutt. Docker lagar ei manglande `data`-mappe som root, og FinStats køyrer som en vanleg brukar. Containeri gjær no mappa til si eie og byter so til den brukari, so `docker run` fungerer på første forsøk. For filer som nokon annan enn brukar 1000 eig, set `PUID` og `PGID`; `--user` fungerer som før.
- Ei datamappe som verkeleg ikkje kan skrivast til, vert rapportert me eingong, me kommandoen som rettar d, istade for 30 sekund seinare som en databasefeil.

## [1.0.0] - 2026-09-20

Klar for alle

### Added
- En ferdig Docker-bilete, no på `ghcr.io/finstats/finstats`, for 64-bits Intel/AMD- og ARM-maskiner (en Raspberry Pi 4 elle 5 fungerer): `:latest` e den nyaste utgjevinga, en versjonstagg står der han står, og `:edge` e utviklingsversjoni.
- Kvar utgjeving kjøme på utgjevingssida til prosjekte på GitHub me dinna endringsloggi.

### Changed
- Versjonsnummera tyde no d dei seier: 1.x-oppdateringar øydelegg ikkje dataa dine, tryggingskopiane elle innstillingane.
- Docker Compose-fila og all rettleiing bruka d publiserte bilete. En `finstats:latest` du he bygt sjølv, treng bære at namne på bilete vert endra; `data`-mappa vert me urørt.

## [0.10.2] - 2026-09-20

### Fixed
- På en telefon kunne Innstillingar og sida til en serie rulle sidelengs: en etikett meint bære for skjermlesara slapp ut av tabelli sin, og episodelistene og rada me medverkande gjor d samme. Dei rullar no inne i sej sjølve.
- Tilbakeblikke rullar ikkje lenger sidelengs når bakgrunnsorde i en kapittel e breiare enn sida.
- Ei lang «ser på …»-linje under **Spelast no** sluttar no me ellipse istade for å verte kutta midt i en bokstav.
- Plakati framfor kvar serie på en profil va ei ekstra, namnlaus lenkje for skjermlesara; han e gøymd for dei, og titteli ved sida e lenkja.

## [0.10.1] - 2026-09-20

### Fixed
- Nedlasting av en tryggingskopi feila i nettlesari («Kjeldefila kunne ikkje lesast» i Firefox) utan nåke på sida som sa kvifor: FinStats komprimerte en tryggingskopi, som alt e komprimert, eingong til, og den dobbeltpakka overføringa braut av før slutti. Tryggingskopia vert sende som dei e, me storleiki oppgjeven på førehand, so nettlesari kan vise reell framdrift.

## [0.10.0] - 2026-09-20

Tryggingskopia, og flytting til en ny installasjon

### Added
- En tryggingskopi kvar veke, og dei fem nyaste vert tekne vare på: kvar avspeling me tidslinja for pausa og hopp, sett-merke, løyve, heimeadresser, serverloggi og innstillingane dine, i éi lita fil, nokren hundre KB for tusenvis av avspelingar. **Innstillingar → Tryggingskopia** listar dei me en **Last ned**-knapp kvar, lagar ein når du vil, og styrer kor ofte og kor monge. Dei ligge i `backups`-mappa i datamappa di. En tryggingskopi inneheld ingen API-nøkkel til Jellyfin og inga innloggingsøkt, men han e en komplett sjåhistorikk me IP-adresser, so hald han en privat stad.
- Gjenoppretting slær saman istade for å skrive over: avspelingar som alt finst, vert hoppa over, so d e trygt å gjære d to gonga elle inni en installasjon som he køyrt ei stond. Fjern haken for «Gjenopprett òg innstillingar og løyve» for bære historikki. Tryggingskopia fungerer på tvers av versjona begge vega. Ifrå en terminal: `finstats backup` og `finstats restore <file>`.

### Changed
- Innsamlinga sjekkar kvart sekund menst nokon ser på og kvart 5. sekund menst ingen gjær d, der ho før sjekka kvart 5. sekund heile tida, so pausa, hopp og sporbyte vert registrerte på sekunde. Begge ligge under **Innstillingar → Innsamling**; en gammalt intervall du he endra, må setjast på nytt.

### Fixed
- Ei IP-adresse i detaljane til ei avspeling vart broten over to linjer midt i adressa. Merke Lokal/Utanfrå flyttar sej under når d ekje plass.

## [0.9.1] - 2026-09-20

### Fixed
- Folk heime vart viste som utanfrå. Ei eining på ditt eie nettverk som når Jellyfin gjennom d offentlege namne, til dømes via en omvend proxy elle en domene, kjøme me den offentlege IP-adressa til husstandi, og bære private adresser vart rekna som lokale. FinStats lærer no den offentlege adressa og reknar dei avspelingane som lokale for heile historikki, og d huska tidlegare adresser.

### Added
- **Innstillingar → Heimenettverk**: adressene FinStats reknar som heime, der du kan leggje til dine eie (ei tidlegare adresse, en heim nummer to, en VPN-utgang) elle slå av oppslage. Oppslage spør ei enkel «ka e IP-en min»-teneste kvart 15. minutt og prøver fleire (Amazon, Cloudflare og andre) fordi DNS me reklameblokkering, som Pi-hole, ofte blokkerer dei, og éi av dei treng ingen DNS i d heile. Førespurnadi inneheld ingenting om dej elle serveri din, og me brytari av snakkar FinStats ikkje me nåke anna enn Jellyfin. `FINSTATS_PUBLIC_IP_URL` vel ei teneste du sjølv he.

## [0.9.0] - 2026-09-20

Kvar tabell, i den rekkjefølgja du vil

### Added
- Alle tabella kan sorterast: klikk på ei overskrift for å sortere ette na, eingong til for å snu, og en tredje gong for den opphavlege rekkjefølgja. Innhalde i cellene vert forstått, so «3 d 2 t» vert sortert som tid og «1,4 GB» som storleik, og tomme celler kjøme sist. D gjeld brukarlista, sjåarane, episodane, einingane, programtillegga, dei planlagde oppgåvene, tabellvisinga av kvart diagram, løyvematrisa og stolpelistene (kodeka, oppløysingar, klienta og resti), som får ei eie, slank overskriftsrad.
- Aktivitet og serverloggi vert sorterte på tvers av alle sidene sine, ikkje bære radene på skjermi, og rekkjefølgja vert halden i adressa, so ho overlever ei ny lasting elle ei delt lenkje.
- Tabella me ti rader elle fleire får en lite filterfelt over sej.

## [0.8.2] - 2026-09-20

### Changed
- Årsveljari i tilbakeblikke e den samme brytari som alle andre tidsromveljara i FinStats (7 d · 30 d · 90 d …).

## [0.8.1] - 2026-09-19

### Added
- Sider for skodespelara og regissøra, ifrå «Personane du såg mest av» i tilbakeblikke elle den nye **Medverkande**-rada på kvar film, serie elle episode: alt dei e me i på serveri din, kor mykje som e sett og av kven, og ka som framleis venta. Sidene følgjer tidsromme du vel, dei som bære får sjå sin eie statistikk, ser bære si eie sjåing der, og Esc tek dej tilbake.

## [0.8.0] - 2026-09-19

Tilbakeblikke he fått ny drakt, og meir å fortelje

### Added
- **Personane du såg mest av**: skodespelarane du broka mest tid me, og regissørane bak d du såg, som ei rad me portrett. Medverkande vert lesne saman me biblioteke og dukkar opp ette den første lesinga av biblioteke ette dinna oppdateringa.
- **Åre ditt i daga**: kvar dag i åre som ei rute, lysare jo meir du såg, me den lengste rekkja di og den største dagi din over. Hald peikari over en dag, elle gå gjennom kalenderi me piltastane, for å sjå ka han inneheldt.
- **I tal**: avspelingar, visingstid, ulike titla, daga me sjåing, lengste rekkje og kor mykje som va **gjensyn**, pluss korleis åre fordeler sej myllå episoda, filma og musikk.
- Sjangerkapitle seier kor monge sjangra du va innom, og kor stor del den største tok.

### Changed
- Ei ny utforming: d opnar me åre ditt «spela av på nytt», overskrifta ved sida av plakatane som fylte d, over ei bølgjeform av åre me éin stolpe per veke og dei høgaste lyste opp, og du vel åre rett over. Kvart kapittel he ei overskrift me nøkkelorde i fiolett, ei setning som ber tala, og orde til kapitle stort og svakt bak. D sluttar sann filma gjær, me rulletekstane: i hovudrollene, regi, vist på, speletid.
- Tima, vekedaga og månada deler eitt **Aktivitetsmønster**-kapittel me en brytar myllå dei, og overskrifta følgjer d du ser på («Fredagi tok krona»).

### Fixed
- **Spelast no** på Oversikt stod tom når ingen såg på, so overskrifta såg ut til å høyre til filtera under na. Boksi «Ingenting vert spela av akkurat no» e tilbake.

## [0.7.10] - 2026-09-19

### Added
- Bibliotekbilete: kvart bibliotekkort, og sida til biblioteke, viser bilete Jellyfin bruka for d, gjennom biletproxyen til FinStats sann som plakatane. En bibliotek utan bilete helde på ikone sitt.

## [0.7.9] - 2026-09-19

### Changed
- Sjangra-korte opnar som **liste** igjen, me radari som d andre vale på brytari. En val du alt he gjort, vert halde på.

## [0.7.8] - 2026-09-19

### Added
- Ei radarvising for Sjangra-korte, på Oversikt og på profila: éi eike per sjanger, en brytar myllå **Radar** og **Liste** i hjørne av korte, og vale vert huska. Når du helde peikari over en ponkt elle bruka piltastane, ser du visingstida og deli til den sjangeri.

## [0.7.7] - 2026-09-19

### Fixed
- Filma som høyrer til ei samling, vart viste som «Ikkje lenger i biblioteke». Me *grupper filmar i samlingar* slått på gir Jellyfin ut samlinga istade for filmane inni, so FinStats såg dei aldri; på éin server gøymde d 556 av 1 238 element i Movies-biblioteke. FinStats ber no om filmane sjølve, og lesinga dinna oppdateringa set i gang, hentar dei tilbake og koplar avspelingane deira til på nytt.

## [0.7.6] - 2026-09-19

### Changed
- På profila he **Seria**-korte flytta ned til rett over Sjangra, so ei lang liste me seria ikkje lenger skubbar visingstida, aktivitetsdiagramme og topptitlane ut av syne. Dagsrekkjene står att øvst, og en serie du he opna, helde sej open når du endrar tidsromme.

## [0.7.5] - 2026-09-19

### Changed
- Søke treffer ord for ord, i ka rekkjefølgje som helst, me ka som helst imyllå, istade for å krevje den nøyaktige frasen, so «alya hides» finn *Alya Sometimes Hides Her Feelings in Russian*. Store og små bokstava, aksenta og teiknsetjing spelar inga rolle, en innleiande «The» vert ignorert, og en feiltast vert tilgjeven («mentalsit» finn *The Mentalist*); ord på tre bokstava elle færre må framleis skrivast rett. Betre treff kjøme fyrst, og musikk vert òg funnen ette artist.
- Søkjefelta på Aktivitet og i serverloggi treffer ord for ord på tvers av alle kolonnane: «alya opera» finn avspelingar av den serien i Opera.

## [0.7.4] - 2026-09-19

### Changed
- Endringsloggi e gruppert ette serie: kvar `0.x`-serie e éi gruppe som kan faldast saman og helde utgjevingane sine, me ka serien handla om, kor monge d e og når. Bære den nyaste starta open, og ei lukka gruppe viser likevel ka for ein du køyrer.

## [0.7.3] - 2026-09-19

### Fixed
- Klokka under Spelast no nølte me nokren sekunds mellomrom: oppdateringa kvart 5. sekund teikna na òg på nytt, so kvar gong serveri va en brøkdel føre, flytte tale sej for tidleg og stod so stille i en sekund. No flyttar bære tikke eingong i sekunde na, me nøyaktig eitt sekund (målt 999 til 1 001 ms ifrå kvarandre), og oppdateringa rettar framleis opp ette en pause, en hopp elle verkeleg avdrift, og landar på tikke.

## [0.7.2] - 2026-09-19

### Added
- Gruppesjåing, direkte: kvart Spelast no-kort seier «Me maria» menst folk ser d samme saman, der ei gruppe før bære vart kjend att når avspelingane va ferdige. Direkte tel folk som saman når dei e på samme tittel og anten starta innanfor gruppevindauge elle e på nesten samme posisjon, nåke som òg fungerer om FinStats vart starta på nytt midt i en straum.

## [0.7.1] - 2026-09-19

### Changed
- Spelast no tel kvart sekund: klokka tikkar og framdriftslinja glir imyllå, istade for å hoppe kvar gong serveri vert spurd. Serveri vert framleis spurd kvart 5. sekund og e framleis fasiti: visinga synkroniserer på nytt ved en pause, en hopp elle når serveri e føre, men ikkje for en posisjon som bære e nokren sekund gammal, sida klienta bære melder ifrå om lag kvart tiande sekund. Strauma på pause står stille.

## [0.7.0] - 2026-09-19

Kven ser saman

### Added
- Gruppesjåing: når ulike persona starta samme tittel samstundes, reknar FinStats d som sjåing saman. En **Sett saman**-kort på Oversikt viser gruppene, titlane dei deler og timane dei he hatt saman, kvar profil viser kven den personi ser mest saman me, delte avspelingar he en lite personmerke i Aktivitet, og detaljane til ei avspeling seier kven andre som såg me. D fungerer på heile historikki din, importerte avspelingar òg.
- Ei innstilling for kor tett startane må liggje, 60 sekund som standard: om lag en tredjedel av ekte gruppeøkter starta 6 til 60 sekund ifrå kvarandre. Folk må òg halde fram me å sjå ved sida av kvarandre i en par minutt, so to persona som tilfeldigvis opnar samme episode, ekje ei gruppe.

### Changed
- Søke opnar me **Ctrl + Space** istade for Ctrl/⌘ + K.

## [0.6.1] - 2026-09-19

### Added
- Administratora kan opne tilbakeblikke til en anna person, øvst på Tilbakeblikk-sida elle me **Opne tilbakeblikk** på profili til personi. D vert då skreve om dei me namn istade for «du».

### Changed
- Alle andre ser framleis bære sitt eie tilbakeblikk, samme ka løyve dei he, og d finst framleis ikkje nåke tilbakeblikk for heile serveri.

## [0.6.0] - 2026-09-19

Profili din, og kor du e i kvar serie

### Added
- Framdrift i seria på profila: kvar serie du he vore innom, som en stolpe me eitt segment per episode (sett, påbyrja elle ikkje endå), som kan opnast sesong for sesong, sortert ette kor nær du e å verte ferdig, me eie visingar for ferdige seria og for alt.
- **Merk som sett** på din eie profil, per episode, sesong elle serie, for d du såg menst ingenting registrerte. Merka for «spela av» i Jellyfin vert òg broka. Merka vert verande i FinStats og endrar aldri nåke i Jellyfin; ei registrert avspeling kan ikkje avmerkast, merka du he lagt inn sjølv, kan d.
- Dagsrekkjer på profila: den lengste rekkja di av daga på rad me ei avspeling, den noverande rekkja di, og kor monge daga du he sett nåke.
- **Profili min** i sidepanele.
- Aktivitet viser kor avspelinga stoppa (til dømes `44:15 / 45:00`) under framdriftslinja, og dato og klokkeslett under «for 3 t sida». Avspelingar importerte ifrå Jellystat he ingen stoppposisjon, fordi Jellystat aldri registrerte nokon.

### Changed
- Episoda som manglar elle ekje sende endå, vert ikkje lenger rekna som ein del av biblioteke ditt. Jellyfin listar dei som plasshaldara utan fil, og dei talde før me i tale på episoda.

## [0.5.1] - 2026-09-19

### Added
- En logo: en avspelingsknapp skoren i tre diagramstolpa, i sidepanele og på innloggingsskjermane, som en skarp SVG me en `favicon.ico` i fleire storleika for nettlesara og bokmerkelinjer som vil ha ein, pluss en ikon for heimeskjermi på telefona og nettbrett.

## [0.5.0] - 2026-09-19

Du avgjer kven som ser ka

### Added
- Løyve under Innstillingar → Tilgang, for alle på eingong elle person for person: **sjå aktiviteti til alle** (statistikki og historikki til andre, Brukara-sida, kvar pågåande straum), **sjå nettverksdetalja** (IP-adresser, einings-ID-ar, lokal elle utanfrå), **sjå serveri** (Server-sida, serverloggi, mislukka innloggingar, filstia) og **administrere FinStats** (innstillingar, oppgåver, import, sletting av avspelingar).
- Innlogging per person, for å sleppe inn nokren få utan å opne FinStats for kvar Jellyfin-brukar.
- Endringar gjeld me eingong, òg når tilgang vert teken bort, og alt vert handheva av serveri istade for ved å gøyme knappa.

### Changed
- Jellyfin-administratora he framleis alt og e dei einaste som kan endre løyve; ein som får administrere FinStats, kan ikkje gje nåke, verken til sej sjølv elle andre.
- Tilbakeblikke for åre e framleis personleg, samme ka som e gjeve.
- Ingenting endrar sej for eksisterande installasjona før du gir nåke.

## [0.4.5] - 2026-09-19

### Changed
- FinStats e lisensiert under **GNU General Public License v3.0**. Ingen tidlegare versjon vart nokon gong publisert, so ingenting kom ut under andre vilkår. Lisensane til dei medfølgjande skrifttypane ligge ved sida av dei.

## [0.4.4] - 2026-09-19

### Added
- Esc går eitt steg tilbake: på en opna bibliotek, en brukar elle en tittel tek Esc dej tilbake dit du kom ifrå, me lista nøyaktig sann du forlét na. D som e ope, vert lukka fyrst (avspelingsdetalja, søk, en nedtrekksmeny, mobilmenyen), og Esc gjær ingenting menst du skriv elle bruka en diagram me tastature.

## [0.4.3] - 2026-09-19

### Changed
- FinStats huska ka kvar side sist viste: går du tilbake til en profil, en bibliotek elle en tidsrom du alt he sett på, vert d teikna me eingong, profilbilete og alt, menst ferske tal vert lasta bak, og sida vert bære teikna på nytt om nåke he endra sej. D som vert huska, ligge i fana i nettlesari og vert forkasta når du loggar ut elle endrar nåke.
- Plasshaldara for lasting dukkar bære opp når lastinga faktisk e treg. Før viste kvar side dei i minst en tredjedel av en sekund.

### Fixed
- Bibliotek som vart sletta i Jellyfin og aldri hadde ei avspeling, vert ikkje lenger lista. En sletta bibliotek me historikk vert framleis vist, merkt som fjerna.

## [0.4.2] - 2026-09-19

### Fixed
- På Brukara-sida stod **Admin**-merke på ei eie linje og gjor den rada høgare enn dei andre. Merka står no ved sida av namne.

## [0.4.1] - 2026-09-19

### Fixed
- Orde «null» dukkar ikkje lenger opp under titteli på Oversikt utanom desember og januar, når d ekje nåke tilbakeblikk-banner å vise.

## [0.4.0] - 2026-09-19

Sjå ka som e endra utan å forlate appi

### Added
- **Endringslogg**-fana, som viser kvar utgjeving, me versjoni du køyrer merkt.
- Versjoni i statuslinja lenkjer dit, og fana viser en prikk ette ei oppdatering til du he sett på na.

### Changed
- Å kople historikk til omdøypte element på nytt går mykje fortare: ifrå 18 sekund til en tredjedel av en sekund me 666 foreldrelause titla, so oppstarti helde sej augneblinkleg når historikki veks.

## [0.3.3] - 2026-09-19

### Changed
- Biblioteke vert lese på nytt rett ette at jobbi **Scan Media Library** i Jellyfin e ferdig, aldri på en eie tidtakar og aldri menst ei skanning køyrer. FinStats he alltid bære lese ifrå Jellyfin, og d starta aldri ei skanning der.
- Ei vekentleg lesing som tryggleiksnett dekkjer servera som nyttar sanntidsovervaking, der skanningsjobbi kanskje aldri køyrer.
- D gamle intervalle e no bære ei reserveløysing, for når følgjinga e slått av elle serveri ikkje melder om nokon skanningsjobb.

### Added
- Brytari **Følg biblioteksskanninga i Jellyfin** under Innstillingar → Innsamling, på som standard.

## [0.3.2] - 2026-09-19

### Fixed
- Historikki overlever namnebyte. Jellyfin gir ei fil som he fått nytt namn, en ny ID, og d gjor at dei tidlegare avspelingane vart ståande att under d rå mappenamne (`Title (2010) [tmdbid-…] [imdbid-…]`) utan plakat. Desse avspelingane vert no kopla til den noverande oppføringa i biblioteke: via TMDB-, IMDb- elle TVDB-ID når d gamle namne he ein, elles via reinska tittel og år, og episoda via serie pluss sesong- og episodenummer. En treff må være eintydig, so titla som verkeleg vart sletta, vert ståande som dei e.
- Direkte-TV-kanala importerte ifrå Jellystat vart rekna som filma, fordi Jellystat ikkje registrerer nokon elementtype. Dei vert no kjende att som direkte-TV, og nye avspelingar av direkte-TV vert registrerte me typen ifrå Jellyfin.

### Changed
- Direkte-TV vert heilt halde utanfor tilbakeblikke: topplister, totala, personlegdom, rekorda og plassering.

### Added
- `finstats relink` for å kople historikk på nytt når du vil. D køyrer òg ved oppstart og ette kvar lesing av biblioteke.

## [0.3.1] - 2026-09-19

### Changed
- Tilbakeblikke e strengt personleg: alle, administratora òg, ser bære sitt eie, og «Alle»-visinga, brukarveljari og serverkapitle e borte.
- Tilbakeblikke opnar på åre som e «klart»: inneverande år i desember, elles åre før. Andre år og «Siste 12 månada» e framleis eitt klikk unna.

## [0.3.0] - 2026-09-19

Åre ditt i tilbakeblikk

### Added
- **Tilbakeblikk**-fana, ei historie å rulle gjennom om eitt år: tima me sjåing, toppseria, filma, spor og sjangra, og en sjåarpersonlegdom (nattugle, morgonfugl, helgekrigar, maratonsjåar, filmelskar, musikkelskar, vanedyr) me diagram over tid på døgne og vekedag.
- Åre månad for månad, me titteli som prega kvar månad.
- Rekorda: største dag, største maraton, lengste rekkje daga på rad, lengste enkeltavspeling, titteli du såg oftast om att, første avspeling i åre og den eldste titteli du såg.
- Oppdagingar: nye seria påbyrja, filma og episoda fullførte, og seriane som fekk nøyaktig éin episode.
- En banner på éi linje på Oversikt i desember og januar når en tilbakeblikk e klart.

## [0.2.0] - 2026-09-19

Meir enn Jellystat nokon gong registrerte

### Added
- Ei tidslinje for kvar avspeling: start, pause, framhald, hopp, byte av lyd og underteksta, direkteavspeling som vert til omkoding, og stopp. I tillegg tal på pausa og hopp, kor avspelinga heldt fram ifrå, og om sjåari va på d lokale nettverke.
- **Server**-fana: Jellyfin-versjon, ventande oppdateringar og omstarta, diskbruk per bibliotek (Jellyfin 10.11+), programtillegg, resultat ifrå planlagde oppgåver og kvar registrerte eining.
- Innsikt i **avspeling**: samtidige strauma over tid, ka klienta som tvingar fram omkoding, kor langt folk kjøme før dei stoppar, lokale avspelingar mot avspelingar utanfrå, og sjåvana.
- Ka biblioteke ditt e laga av: oppløysingar, kodeka, dynamisk område, containera, titla per tiår, tilskot per månad, dei største titlane, og titlane ingen nokon gong he sett, kontrollert mot både historikki i FinStats og avspelingsmerka i Jellyfin.
- Oversikt: flest samtidige strauma, estimert mengd strøymde data, deli avspelingar utanfrå, sjangra ette visingstid og mislukka innloggingar.
- Elementsider: lenkjer til IMDb og TMDB, studio, bitdjupn og biletfrekvens, og kven som he titteli merkt som spela av i Jellyfin.
- Brukarsider: sjangra, og filmane, episodane og favorittane Jellyfin he registrert.

### Fixed
- Tidsetiketti på en Spelast no-kort renn ikkje lenger ut av korte.
- Lange etiketta på Avspeling-sida vart kutta for tidleg.
- En serie som he forlate biblioteke, viser framleis sesong- og episodelista si.

## [0.1.0] - 2026-09-19

Første utgjeving

### Added
- Direkte innsamling ifrå øktene i Jellyfin: brukar, tittel, klient, eining, IP-adresse, avspelingsmetode, grunna til omkoding og maskinvareakselerasjon, detalja om bilete, lyd og underteksta, og visingstid mot tid på pause.
- Innlogging me Jellyfin-kontoen din. En oppsett i to steg lagar en eie API-nøkkel for FinStats, og passord vert aldri lagra. Brukara som ekje administratora, kan om ønskjeleg logge inn for å sjå bære sin eie statistikk.
- Sidene Oversikt, Aktivitet, Brukara, Bibliotek, Avspeling og Serverlogg, en kommandopalett (Ctrl/⌘ K) og ei statuslinje som oppdaterer sej direkte.
- Import ifrå Jellystat: store tryggingskopia vert strøymde til disk og importerte på sekund, og d e trygt å importere samme fila to gonga.
- Éi lita binærfil me innebygd nettgrensesnitt og éi SQLite-fil, pluss en Docker-bilete.
