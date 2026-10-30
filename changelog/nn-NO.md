# Endringslogg

Alt som er endra i FinStats, det nyaste først. Denne fila vert kompilert inn i
programfila og er det som fana **Endringslogg** i appen viser.

Format: `## [version] - date`, eller `## [version] - first day to release day` for ei utgjeving som vart laga
over fleire dagar, deretter `### Added`, `### Changed`, `### Security`, `### Performance`, `### Stability`, `### Fixed`
eller `### Removed` (Nytt, Endra, Tryggleik, Yting, Stabilitet, Retta, Fjerna) med eitt punkt per endring. Tryggleik er
noko nokon kunne ha misbrukt, Yting er noko som
vart raskare eller lettare, Stabilitet noko som ikkje lenger kan krasje, stanse, lekke eller miste data, Retta noko
som gav feil svar. Ei `x.y.0`-utgjeving har ei kort tittellinje under overskrifta si: den tittelen brukar appen
som overskrift for heile serien.

## [2.3.1] - 2026-10-26 to 2026-10-30

### Security
- **Oppsettet ber om ein eingongskode**: til FinStats er sett opp, skriv han ein kode i loggen sin når han startar (`docker logs finstats`), og oppsettet ber om koden før det testar ei adresse eller set opp noko. Elles kunne den som nådde porten først, peike FinStats mot sin eigen Jellyfin og bli administrator der, eller bruke testen til å sjå seg om i nettverket.
- **Bibliotektilgangen følgjer Jellyfin**: kvar person ser berre biblioteka Jellyfin opnar for dei, og i dei berre titlane Jellyfin viser dei under aldersgrensa, merkelappane og reglane for titlar utan aldersgrense, i bibliotekssider, søk, medverkande, bilete, Nyleg lagt til, Lista mi, Kjem snart, kalenderabonnementet og varsel om nye titlar. Alle som var logga inn, kunne bla i alle bibliotek.
- Ei lenkje kunne sende nokon frå velkomsten til ein annan nettstad rett etter at dei logga inn. No vert berre FinStats' eigne sider følgde.
- Serverloggen og dei mislukka innloggingane i innsikta gav alle med *Sjå server* innloggingsadressene og aktiviteten til alle. Linjene til andre krev *Sjå alle*, og ei adresse *Sjå nettverk*, som overalt elles.
- Ein personleg mottakar kunne nå det lokale nettverket gjennom eit namn som svara annleis andre gongen det vart slått opp. No vert det kopla til adressa som vart sjekka, fleire adresseområde som aldri når internett, vert rekna som lokale, den offentlege adressa til hushaldet vert òg avvist, og eit namn som ikkje kan slåast opp, vert avvist med dei same orda som ei lokal adresse.
- Bak ein proxy som legg til ei eiga `X-Forwarded-For`-linje, eller frå eit IPv6-nettverk, kunne kvart innloggingsforsøk kome frå ei ny adresse, så grensa slo aldri inn. No får eit IPv6-/48 50 forsøk på fem minutt, ei full grense avviser ei ny adresse i staden for å gløyme ei gammal, heile serveren stoppar ved 60 feil i minuttet, og ei mislukka innlogging vert ført i revisjonsloggen éin gong per adresse og tidsrom.
- `FINSTATS_TRUST_PROXY` namngjev proxyane (adresser eller adresseområde) som ein trur på `X-Forwarded-For`, `X-Forwarded-Host` og `X-Forwarded-Proto` frå, og `1` eller `true` stolar no berre på ein motpart på loopback eller eit privat nettverk. Slått på kunne kven som helst velje si eiga adresse; slått av bak ein proxy stengde ti feil gjettingar alle ute.
- Når brukarane er lesne, vert ei innlogging for eit namn som er ukjent, deaktivert, fjerna eller ikkje sleppt inn i FinStats, svara som eit feil passord utan at Jellyfin vert spurd, og ikkje før Jellyfin ville ha svara. Slike namn kunne låse Jellyfin-kontoar, kome seg rundt regelen til Jellyfin for fjerntilgang eller gøyme seg bak adressa til FinStats.
- Den Jellyfin held til heimenettet (*Tillat eksterne tilkoplingar* av), loggar inn i FinStats berre derifrå, sidan Jellyfin vurderte brytaren ut frå adressa til FinStats sjølv, og ei brukarlesing som ikkje seier om nokon er deaktivert, held dei ute.
- Ein øktcookie vert merkt Secure når sida det vert logga inn frå, er på https, òg bak ein proxy som ikkje seier det.
- Ei innlogging som lukkast, tilgjev ikkje lenger gjettingane før ho. På https er øktcookien ein `__Host-`-cookie som ein nettstad på eit nabonamn ikkje kan byte ut, og han vert ikkje godteken under noko anna namn, og ei utlogging avsluttar kvar økt nettlesaren sende.
- Berre ein kalendernøkkel opnar kalenderabonnementet frå adressa; andre nøklar vert sende i headeren.
- Ein API-nøkkel kan ikkje lenger gje løyve, endre kven som slepp inn eller kva som er offentleg, gjenopprette ein tryggingskopi, eller leggje til, endre eller teste ei teneste eller ein varslingsmottakar: alt dette varer lenger enn nøkkelen, så det krev ein innlogga nettlesar.
- Kvart svar har `Cross-Origin-Opener-Policy: same-origin` og ein `Permissions-Policy`, API-svar har `Cross-Origin-Resource-Policy: same-origin` og er `no-store` om dei ikkje seier noko anna, og CSP-en tillèt ikkje lenger `blob:`-bilete. Ei side som opna FinStats, kunne seinare gjere fana om til ei innloggingsside av sitt eige.
- Der FinStats veit kva skjema han vert nådd med (`X-Forwarded-Proto` frå ein klarert proxy, eller den offentlege adressa), vert ei skriving frå eit anna skjema avvist som ein annan nettstad.
- `/auth/me`, sida til ein tittel og Bibliotekhelse lenkjer til Jellyfin på den offentlege adressa; adressa FinStats sjølv les Jellyfin på, vert berre vist til administratorar.
- Eit bilete vert henta frå Jellyfin berre for noko FinStats har, òg plakatane og korta på ein publisert profil, så bilete frå andre bibliotek kan ikkje bli bedne om med id.
- Nøklane til ein privat tracker vert òg fjerna frå feila i nedlastingskøen, etter forma så vel som namnet, og den som berre har *Sjå nedlastingar*, får vite kva teneste som feila og korleis, aldri den interne adressa hennar eller kvar ho vidaresender.
- Serversida viser kven som brukte ei eining sist berre med *Sjå alle*, og Ukopla media krev det i tillegg til *Administrere*. Utan det tel Oppgåver berre eigne avspelingar og straumar, eit fjerna bibliotek vert vist ut frå eigne avspelingar, og ei ventande Tautulli-tavle kan ikkje avbrytast.
- Innstillingane viser tenesta for offentleg adresse berre med vertsnamnet, og adressa til Jellyfin og datamappa berre til administratorar, og ein tittel biblioteket har mista, vert skildra berre ut frå avspelingar lesaren kan sjå.
- Ein tryggingskopi frå Tautulli med tabellar som ikkje var vanlege tabellar, kunne halde all anna skriving ventande til FinStats vart starta på nytt. Ei slik fil vert avvist, og SQLite les tryggingskopien defensivt.
- Ei opplasting over 4 GB vert avvist, og ein zippa tryggingskopi frå Tautulli vert ikkje pakka ut lenger enn storleiken han oppgjev; begge delar kunne fylle disken databasen til FinStats ligg på.
- Eit svar frå Jellyfin, Sonarr, Radarr, Seerr, oppslaget av offentleg adresse eller GeoIP-nedlastinga vert ikkje lese lenger enn det kan vere, rekna etter utpakking.
- Kva Jellyfin-brukar som helst kunne stoppe FinStats frå å registrere avspelingar for alle ved å melde tusenvis av korte avspelingar av éin tittel. Å finne ut kven som såg saman, tek no éin gjennomgang og held ingen lås medan det går føre seg.
- Avspelingar etter kvarandre, eller éin tittel spela på omgang frå eining etter eining, slår ikkje lenger framande saman til éi gruppe, og den som berre kan sjå seg sjølv, får berre vite om dei avspelinga deira gjekk side om side med, og kor lenge.
- Ei lenkje kunne framleis sende nokon frå velkomsten til ein annan nettstad etter innlogging, gjennom ein sti som `/.//`.
- Den som har *Administrere*, kan ikkje lenger endre kor mange tryggingskopiar som vert tekne vare på eller kor ofte dei vert skrivne, kor lenge ein omstart vert rekna som same avspeling (no høgst ein time) eller når brukarar, aktivitetsloggen og tryggingskopiar vert lesne, og tryggingskopiane deira skuvar aldri ut dei til ein administrator; heimeadresser og reglane for umogleg reise krev det varsla krev, og import av historikk krev òg *Sjå alle*.
- Ein lagra nøkkel, token eller passord vert berre send til serveren han vart lagra for, og ein varslingsmottakar vert oppretta eller endra berre i ei innlogga økt, aldri med ein API-nøkkel. Ein administrator kan slå av eller fjerne den personlege mottakaren til andre, men ikkje flytte han, slå av sertifikatsjekken eller lese ntfy-emnet eller Pushover-brukarnøkkelen.
- Webhookar til Discord og Slack må vere `https`, sidan adressa er nøkkelen.
- Ein flaum av mislukka innloggingar kan ikkje lenger gøyme ei ekte innlogging eller drukne varsla: namn som ikkje er kontoar, gjev eitt varsel i kvarteret som ikkje nemner nokon, varsel vert sende først, og Slack og Discord viser det ei melding seier som tekst og pingar ingen.
- Rangeringa i tilbakeblikket, Kapasitet, eit varsel og ein publisert profil seier ikkje meir enn sidene gjer: inga rangering eller del av tida til alle til den som berre kan sjå seg sjølv, travle augeblikk berre som dagar med tre straumar eller fleire utan *Sjå alle*, ingen mislukka jobb eller tilkopling til den som ikkje kan administrere FinStats, og ein offentleg profil vert endra éin gong om dagen.
- Ein tryggingskopi frå Tautulli med ein generert kolonne vert avvist, og eit bilete vert henta for ei importert avspeling først når biblioteket har tittelen.
- Ei gjenoppretting opnar aldri FinStats meir enn han er: innstillingar kjem berre tilbake når det vert bede om, innlogging, løyve og offentlege profilar vert det både denne installasjonen og tryggingskopien tillèt, og dei offentlege adressene held fram med å vere installasjonen sine. `finstats restore <file>` hentar berre tilbake historikken, slik sida gjer, og innstillingane òg med `--with-settings`.
- Ei gjenoppretting av berre historikken hentar ikkje lenger tilbake heimeadresser, dempa varsel, revisjonsoppføringar som gjev seg ut for å vere installasjonen sine eigne, eller lokaliserte titlar biblioteket manglar, og inga gjenoppretting hentar tilbake avspelingar, loggrader eller einingar datert før 1990 eller etter i morgon. Ei gjenoppretta avspeling vert rekna som importert same kva kjelde ho oppgjev, så ein omstart held aldri fram med henne.
- Datamappa er berre for brukaren til FinStats (0700, filene 0600, `umask 077` i imaget). FinStats nektar å køyre som root over ei datamappe ein annan brukar eig, og nemner `docker exec -u` som skal brukast, og oppstartsskriptet endrar eigar berre på mappa og dei eigne oppføringane til FinStats.
- Namnet på ein mottakar kan ikkje innehalde kontrollteikn, og namnet på ei eining, ein person eller ein tittel kan ikkje lenger forfalske ei linje i loggen.
- Ein parameter i ei lenkje kan ikkje lenger nå ein annan API-sti enn sida sin eigen, og elementbyggjaren i FinUI set ei adresse berre når ho er http(s) eller relativ, aldri `srcdoc`.
- Søk, publiserte profilar og arbeid starta for hand kan ikkje lenger halde FinStats oppteken: eit søk stoppar ved 200 teikn, ein profil vert lagra per språk, kort ventar i ein liten kø, og ein jobb, adresseoppslaget, nedlastinga av staddatabasen og ei testmelding startar berre så ofte, same korleis dei vert bedne om, og testmeldingar vert talde per avsendar. Å hente staddatabasen krev òg det Tryggleik-sida krev.
- Innhaldet i ein førespurnad må halde fram med å kome når dei første 15 sekunda er over, eit svar ingen les, vert gjeve opp etter 30 sekund, og éi offentleg adresse held høgst 64 tilkoplingar, medan ei adresse på denne maskinen eller i eit privat nett, òg ein proxy, er unnateken. Nokre hundre førespurnader som aldri sende innhaldet sitt, kunne hindre FinStats i å svare nokon.
- Ein e-posttenar vert ikkje lesen lenger enn noko svar kan vere (1 MB for eit heilt brev), og ei testmelding stoppar etter like lang tid som ei melding i køen kan ta. Ein tenar som aldri avslutta ei linje, kunne få FinStats til å halde meir og meir minne til han vart stoppa.
- Ei bibliotekslesing vert vurdert mot titlane lesingane har lagra, ikkje dei ein import tok med, og ein Jellystat-tittel i eit bibliotek som alt er lese, kjem inn som borte. Ein import av oppdikta titlar kunne få kvar bibliotekslesing til å stoppe FinStats, ved kvar oppstart.
- Grensa på 60 mislukka innloggingar i minuttet for heile tenaren tel berre forsøk som vert sende til Jellyfin, òg dei som framleis ventar på svar. Namn FinStats avviste sjølv, let kven som helst med nokre få adresser stoppe alle frå å logge inn, og mange gjetingar sende samstundes nådde alle Jellyfin.
- Den Jellyfin held til heimenettet, vert halden der ved kvar førespurnad, økta og nøklane deira òg, ikkje berre når dei loggar inn; heime er eit privat nett eller hushaldet si oppslegne offentlege adresse, aldri ei adresse skriven inn som heime, og bak ein proxy `FINSTATS_TRUST_PROXY` ikkje nemner, vert dei haldne ute. Ein informasjonskapsel eller nøkkel teken ut av huset opna alt frå kvar som helst, og ein forvaltar kunne opne huset for ei adresse dei sjølv valde.
- Ein API-nøkkel kan ikkje lenger skru av det som ville merke at han vert misbrukt, eller tryggleikskopiane som ville gjere om det han gjorde: kor mange tryggleikskopiar som vert haldne og kor ofte, samanslåingsvindauget, heimeadresser og reisereglar, timeplanane for brukarar, aktivitetsloggen og tryggleikskopiar, å dempe eller løyse alle varsel, ein tryggleikskopi i papirkorga eller å fjerne eit varslingsmål. Kvar av dei varte lenger enn nøkkelen når han vart trekt tilbake.
- Plakatane frå Sonarr og Radarr og ønskjelista sin neste episode følgjer òg bibliotektilgangen: ein serie Jellyfin gøymer for nokon, har verken plakat eller dato for dei. Å telje ID-ane til tenestene avslørte kva gøymde seriar som kom, og ei liste nemnde neste episode av ein gøymd serie.
- Saman-kapittelet i årsoversikta nemner berre dei som såg samstundes med deg, slik Saman-sida gjer. Ei gruppe du kom inn i gjennom nokon andre, nemnde folk du aldri såg saman med.
- Ein import frå Jellystat eller Streamystats les heile fila før han tek skrivelåsen i databasen, og skriv så berre radene han held på. Lesen under låsen kunne ei fil med nokre få ekte rader og gigabyte med kva som helst anna halde all anna skriving ventande, avspelingar og innloggingar òg.
- Ei gjenoppretting held éin tilgang per person, for ikkje fleire personar enn eit hushald mange gonger over, og sjekkar ei revisjonsoppføring for dobbel utan å lese kvar oppføring ho nett har skrive. Ein laga tryggleikskopi kunne halde minne og skrivelåsen i databasen lenger og lenger medan han vart gjenoppretta.
- Ei gjenoppretting med innstillingane slår på oppslaget av den offentlege adressa og nedlastinga av stadsdatabasen berre der denne installasjonen òg har dei på, og held aldri færre tryggleikskopiar eller skriv dei sjeldnare enn denne installasjonen gjer. Ein tryggleikskopi som heldt éin, kunne få neste køyring til å fjerne alle andre.
- Ein filstorleik ingen disk kunne romme, vert lesen som ingen storleik, og storleikar vert lagde saman slik at ingen sum kan flyte over. Éin importert storleik fekk bibliotekssidene til å svare med feil for godt.
- Når FinStats veit at han vert nådd over https (han vert beden om på den offentlege https-adressa si, eller ein tiltrudd proxy seier det), vert ei økt berre teken frå `__Host-`-informasjonskapselen, og kvar ny økt der er ei https-økt. Ei innlogging send utan `Origin` gjennom ein proxy som ikkje sa noko, laga ei vanleg økt som ein nabonettstad kunne plante i nettlesaren til nokon.
- Biletet vert bygt med kvar GitHub-handling festa til ein commit og begge grunnbileta til eit samandrag, flytte berre av Dependabot, og ei utgjeving vert bygd utan den delte byggjebufferen. Ein flytt tagg på andre sin handling kunne ha publisert kva bilete som helst som FinStats.
- Stilarket til FinUI, teikna i den besøkande sin eigen utsjånad, kan berre lastast av FinStats sine eigne sider. Ei side på ein annan port på same vert kunne lese av det om den besøkande var innlogga, og kva førehandsinnstilling dei valde.
- Kvar avvist innlogging vert svara på eitt tidspunkt rekna frå då ho vart send, anten FinStats eller Jellyfin avviste ho, og eit namn vert kjent att same kva store eller små bokstavar, òg utover A til Å. Kor raskt ei avvising kom, avslørte kva namn som var nokon, og «Åse» som skreiv `åse` vart avvist utan at Jellyfin vart spurd.
- Feilmeldingar for nedlastingar skjuler klientadresser for andre enn administratorar. Dei som kunne sjå køen, kunne før sjå interne vertsnamn, portar og stiar.
- Grensene for nøklar og varslingsmål vert handheva atomisk. Samtidige førespurnader kunne tidlegare overskride dei.

### Performance
- Ein publisert profil vert rekna ut høgst éin gong i minuttet, ikkje på nytt for kvart bilete og kort på han.
- Historikken til ein person vert gjennomsøkt etter tryggleiksvarsel éin gong om gongen, same kor mange avspelingar som startar.
- Å logge inn eller ut, trekkje tilbake ein nøkkel eller endre løyva til éin person ber berre dei direkte tilkoplingane det gjeld, om å sjå etter på nytt.
- Koplingstavla for Tautulli les historikken éin gong, ikkje éin gong per Plex-brukar.
- Eit bilete som manglar i mellomlageret, vert henta av éin førespurnad medan dei andre ventar, og eit kjelda ikkje klarte å gje, vert ikkje spurt etter igjen før om 30 sekund.
- Filene i nettgrensesnittet vert gzippa éin gong og tekne vare på, og skrifter vert ikkje komprimerte på nytt.

### Stability
- Ein tryggingskopi i det eldre formatet til Jellystat vert lesen éi rad om gongen i staden for heil, og ei linje lengre enn nokon tryggingskopi har, vert avvist.
- To opplastingar frå Tautulli samtidig skriv ikkje lenger i same fil.
- Eit sidetal langt forbi slutten av titlane til eit bibliotek eller av Bibliotekhelse gjev ei tom side, ikkje ein feil.
- Ein import eller ei gjenoppretting der opplastinga vart broten, avviser ikkje lenger alle andre til FinStats vert starta på nytt, og verken den eller ein import som vart forlaten før han byrja, etterlèt ei fil.
- Éi rad i ein tryggingskopi vert ikkje lesen lenger enn 8 MB, og éin tryggingskopi frå Tautulli vert teken imot om gongen.
- Ei avspeling tek vare på høgst 2 000 hendingar, og ein import hoppar over ei avspeling som ikkje startar og deretter sluttar mellom 1990 og i morgon.
- Eitt sterkt negativt episodenummer heng ikkje lenger kvar bibliotekslesing: Bibliotekhelse (òg samandraget), Kapasitet, ny kopling og lokalisering legg saman og samanliknar tal utan å gå rundt, og eit release-bygg sjekkar for overflyt, så noko som er oversett, stoppar éi oppgåve i staden for å gå i ring.
- Eit sesong-, episode- eller sluttnummer vert berre teke vare på der Jellyfin kunne ha lagra det (0 til 2 147 483 647), frå Jellyfin, Jellystat og Tautulli likt.
- Ein tryggingskopi frå Tautulli der id-ane går igjen, vert avvist, sidan nokre MB rader under éin id fekk prosessen til å gå tom for minne, og ein tryggingskopi vert opna som uforanderleg og etterlèt ingen `-wal` eller `-shm`.
- Ei tilkopling må sende førespurnaden sin innan 15 sekund, og høgst 512 vert betente samtidig: nokre hundre halvopne tilkoplingar tok FinStats ned. Ei melding på direktekanalen vert halden til 4 KB.
- Nokre få kilobyte frå Sonarr, Radarr eller Seerr kan ikkje lenger bli gigabyte med minne: ei liste tek ikkje vare på meir enn FinStats les av henne.
- Ein varslingsmottakar som aldri svarar, held berre seg sjølv att: mottakarar får meldingar side om side, og ein som stadig får tidsavbrot, får vere i fred i ti minutt.
- Tid att og samla fart i ein kø, tala i ei gjenoppretta avspeling og dagen ei avspeling forlèt papirkorga kan ikkje lenger flyte over.
- Ei gjenoppretting les heile fila før ho tek skrivelåsen, avviser ei som vert pakka ut til langt meir enn storleiken sin, og skriv i puljar, så innsamlaren held fram med å registrere medan.
- Tryggingskopiar vert namngjevne i UTC og aldri skrivne over ei fil som finst, og berre dei eigne kopiane FinStats tek før oppdateringar, vert rydda bort.
- Ei rad i serverloggen datert fram i tid stoppar ikkje lenger lesinga av aktivitetsloggen: han vert lesen frå den nyaste rada som ikkje er fram i tid.
- Førespurnader som kjem samtidig til ei publisert lenkje, reknar ut svaret éin gong, og korta vert teikna i ein eigen kø, så den som har lenkja, ikkje lenger kan få korta til eigaren frå tilbakeblikket til å svare 503.
- Avspelingane i eit bibliotek som manglar i lista til Jellyfin over bibliotek, ventar to dagar før dei vert flytte til eit anna bibliotek sin kopi av same tittel. Éi liste som utelét eit bibliotek, flytte dei straks, og dei vart verande flytte då biblioteket kom tilbake.
- Ein disk Jellyfin oppgjev som større enn noko tal kan romme, får ikkje lenger kvar lesing av tenardetaljane til å feile.
- Lesingar frå Jellyfin stoppar når ei side blir gjenteken utan nye ID-ar.

### Fixed
- Mislukka innloggingar på ein konto gjev eitt varsel per tidsrom som veks til eit alarmvarsel ved tjue, og eit namn tel mot ein konto berre når det er skrive heilt ("x alice" tel ikkje lenger mot alice).
- Meldingar til Discord og Slack vert escapa først og korta ned etterpå: eit einingsnamn fullt av Markdown fekk Discord til å avvise kvart varsel som nemnde det.
- Ei offentleg IPv6-adresse vert godteken for ein personleg mottakar.
- Bak ein proxy som sjølv loggar folk inn (nginx `auth_basic`), vert ikkje cookien til FinStats avvist lenger.
- Eit visingsnamn som inneheld `{{url}}`, vert vist slik det er skrive i førehandsvisinga av ei publisert side.
- Ei oppsummering for år 9999 viser standardåret i staden for ein feil.
- Ei side som miste eit løyve og fekk det attende medan ho var open, vert halden oppdatert over direktekanalen att utan å lastast på nytt.

### Removed
- **Synnmørsk** er ikkje lenger eitt av språka til FinStats: ein nettlesar som valde det, les nynorsk, skriftspråket det vart laga frå.

## [2.3.0] - 2026-10-07 to 2026-10-26

Kapasitet, direktekanal, Tautulli-stil og språk

### Added
- **Språk**: FinStats snakkar 24 språk. Britisk og amerikansk engelsk og norsk bokmål og nynorsk er skrivne og kontrollerte av utviklaren, som også kontrollerer synnmørsk, den eigne dialekta til utviklaren frå Søre Sunnmøre; dansk, tysk, spansk, finsk, fransk, hindi, islandsk, italiensk, japansk, koreansk, nederlandsk, polsk, portugisisk (Brasil), russisk, svensk, tyrkisk, ukrainsk og kinesisk (forenkla og tradisjonell) er omsetjingar ingen her har kontrollert, og viser på engelsk det dei ikkje har enno. Alle sidene i begge oppsetta, den offentlege profilen, Endringslogg og det serveren svarar med ord (ei feilmelding, korta i Tilbakeblikk, kva ein jobb i Jellyfin gjer) vert viste på språket som er valt under Innstillingar → Utsjånad → Språk, per nettlesar; til eit språk er valt, avgjer språket til nettlesaren, og ein nettlesar på eit språk FinStats ikkje snakkar, får amerikansk engelsk. Tal, datoar, einingar og lister vert skrivne slik det språket skriv dei. Eit skript kan be om eit språk med `Accept-Language`.
- **Tautulli-stil**: eit andre oppsett for dei som kjem frå Plex, lagt opp slik ein klassisk statistikkserver for Plex er. Ei linje øvst i staden for menyen, og berre statistikksidene: Heim, Bibliotek, Brukarar, Historikk, Grafar og Innstillingar, med eit tannhjul for profilen din, Innstillingar, Endringslogg, temaet og utlogging. Vert valt per nettlesar under Innstillingar → Utsjånad → Oppsett. Det lastar berre sin eigen halvpart av FinStats, så det startar raskare; Tilbakeblikk, Saman, På veg, Tryggleik og serversidene vert verande i FinStats-oppsettet. Det har dei same fargane som FinStats, lyst og mørkt, og ingenting på det rører seg. Ikkje knytt til Tautulli.
- **Heim i Tautulli-stilen**: det som vert spela, eitt kort per straum (korleis han vert spela, frå heimenettverket eller utanfrå, når han sluttar), visingsstatistikken som rangerte kort etter tal på avspelingar eller lengd dei siste N dagane, kva kvart bibliotek har og kva som har kome nyleg.
- **Grafar i Tautulli-stilen**: avspelingar etter mediatype (dag for dag, etter vekedag og time, dei ti mest brukte appane og personane), etter korleis dei vart spela (dag for dag, flest straumar samstundes kvar dag, oppløysinga til fila og den som vart sendt, appar og personar) og månad for månad, etter tal på avspelingar eller lengd, for alle eller for dei som er valde.
- **Bibliotek, Brukarar og titlar i Tautulli-stilen**: tabellar over kvart bibliotek og kvar person med kva som sist vart spela, på kva og frå kvar (adressa berre med *Sjå nettverk*), og ei side for kvart bibliotek, kvar person og kvar tittel med tala og historikken sin.
- For skript: `/api/stats/split` (avspelingar etter app, person, oppløysing, dag eller månad, delte etter mediatype eller etter korleis dei vart spela); topplister etter tal på ulike sjåarar (`sort=users`); aktivitet avgrensa til avspelingar som tel som sett (`watched=1`); innsikt berre for dei delane ein spør etter (`parts=`); flest samstundes straumar delt i omkodingar, direktestrøymingar og direkteavspelingar, òg for kvar dag; kva kvart bibliotek har, og kva som sist vart spela der; nyleg lagt til etter type (`kind=`); varmekartet etter mediatype; den siste avspelinga til ein person; om ein straum vert spela frå heimenettverket.
- **Ein direktekanal**: Oversikt, nedlastingane i På veg og statuslinja held seg oppdaterte over éin WebSocket per fane (`/api/live`), så det som vert spela og det som vert lasta ned, endrar seg på skjermen når det endrar seg, ikkje ved neste kikk. Bak ein omvend proxy som ikkje sender WebSocket-oppgraderingar vidare, eller på ei tilkopling som fall bort utan eit ord, spør sidene med nokre sekunds mellomrom som før. `/api/now-playing`, `/api/downloads` og `/api/summary` er som før, for skript.
- **Server → Kapasitet**: om denne serveren vil klare seg, ut frå det FinStats har sett. Dei travlaste augneblinkane som er registrerte, og dei frå dei siste 90 dagane: straumar samstundes, og kor mange som fekk biletet koda om med maskinvare, utan, eller utan at noko vart sagt om det (historikk importert frå Tautulli kjem frå Plex og er halden utanfor). Kvar disk biblioteka ligg på, og kor fort det Jellyfin har lagt til over seks månader, fyller han (ein disk som vart teken i bruk nyleg, vert målt frå ein månad etter den første fila, sidan han vart fylt opp den første månaden), og, der Jellyfin 10.11 eller nyare melder ledig plass, kor lenge plassen rekk i det tempoet. FinStats seier aldri kor mange straumar serveren kunne ha klart.
- **Eksporter .ics** i På veg → Kjem: lista slik du har filtrert ho, lasta ned som ei kalenderfil du importerer éin gong. Ho oppdaterer seg ikkje sjølv; ein kalendernøkkel frå Innstillingar → API-nøklar gjev framleis ein kalender som gjer det.
- **Verdt ein titt**: Kapasitet gjer merksam på det når kvar omkoding Jellyfin har meldt om på 90 dagar, gjekk utan maskinvareakselerasjon medan omkoding er vanleg, når biblioteksskanninga i Jellyfin går ein fjerdedel av tida eller meir, og når ein disk har plass til mindre enn seks månader til. Kvart punkt kjem med tala sine og kan leggjast til sides som funna i Bibliotekhelse.
- **Kva biblioteket ditt består av** kan avgrensast til eitt bibliotek eller fleire, både under Bibliotek og på sida til kvart bibliotek (`library_id=a,b`).
- **Dato- og tidsformat** under Innstillingar → Utsjånad: ein dato som 31.01.2026, 31/01/2026, 31-01-2026, 01/31/2026, 2026-01-31, 2026.01.31 eller 2026/01/31, og eit klokkeslett med 24 eller 12 timar, med eller utan 0 framfor, eller begge slik språket skriv dei. Vert lagra i denne nettlesaren.
- **Ei velkomst**: første gong nokon loggar inn, same kva nettlesar, og på slutten av oppsettet, spør FinStats om språk, dato- og tidsformat, lyst eller mørkt og oppsett, med eit bilete av kvart oppsett. Hopp over beheld britisk engelsk, 31/01/2026, 13:05, fargane til eininga og FinStats.

### Changed
- Logoen inne på ei side er det nakne merket ved sida av namnet, sett i Inter same kva skrift eit førehandsval vel; der han står åleine (favikonet), held han på den avrunda flisa si.
- Ei avspeling som vart koda om i programvare, vert registrert slik der Jellyfin seier det; før såg ho lik ut som ei frå ein server som ikkje seier noko. Sidene kallar framleis begge Programvare.
- Ei oppdatering køyrer no, éin gong og den eldste versjonen først, det ein versjon treng gjort med ein database som ein eldre FinStats skreiv, ved sida av migreringane. Arbeidet for 2.3.0 les detaljane til serveren ved første oppstart, så Kapasitet er der med ein gong.

### Performance
- **Ei side som står open, spør ikkje etter noko**: Oversikt sende 30 førespurnader i minuttet for å halde seg oppdatert, og nedlastingane i På veg 18; med direktekanalen sender ingen av dei nokon. Same kor mange faner som er opne, spør FinStats ikkje Jellyfin, Sonarr og Radarr om meir enn før, og ei skjult fane lèt framleis lesinga av nedlastingskøa gå seinare.
- Sida til ein person les berre avspelingane til den personen: summane og den siste avspelinga vart rekna ut over alle avspelingar på serveren og så avgrensa til personen, om lag fire sekund på ein million avspelingar.
- Sida til eit bibliotek les berre titlane og avspelingane i det biblioteket, der ho før talde opp alle biblioteka og heldt på eitt: ein tredjedel av tida på ein million avspelingar.
- Når klokka til Jellyfin gjekk føre klokka til FinStats, las *Etter biblioteksskanninga i Jellyfin* heile biblioteket på nytt kvart minutt til skilnaden var gått; kvar skanning vert no følgd éin gong.
- Ei GeoIP-fil som ikkje kunne opnast, vart kopiert på nytt, hundre megabyte og meir, og kvar adresse plassert på nytt, kvart kvarter; no vert ho prøvd igjen berre når ho endrar seg.
- Bibliotekhelse opnar med ein gong etter at peikaren har kvilt på lenkja: det som vart henta på førehand, vart lagt der sida aldri såg etter.

### Stability
- Ei oppdatering som feila etter at ei migrering hadde køyrt, og som Docker starta på nytt med ein gong, tok ein ny kopi av den halvvegs oppdaterte databasen ved kvar oppstart, og tre oppstartar rydda bort den einaste kopien den førre versjonen kunne opne. Kopien frå før oppdateringa vert no teken vare på.
- Ein tittel som vart sletta i Jellyfin medan biblioteket vart lese (ei fil bytt ut av Radarr), kunne få lesinga til å gå glipp av ein annan tittel som framleis var der og merkje han som fjerna, og avspelingane hans kunne så verte flytta over på ein annan kopi av han. No spør FinStats først etter ID for det ei lesing ikkje såg.
- Å gjenopprette ein tryggingskopi som var teken før ei fil vart døypt om eller oppgradert, gjenoppretta kvar avspeling som sidan var flytta over på den nye fila, ein gong til, og dobla ho for godt.
- Ein førespurnad i svaret frå Seerr som FinStats ikkje kunne lese (eit felt som fekk nytt namn i ei Seerr-oppdatering), vart merkt som fjerna av den daglege fullstendige lesinga.
- Ein jobb som trefte ein feil, vart ståande som «køyrer» for godt, og ingen utløysar starta han igjen før FinStats vart starta på nytt; no endar han som mislukka.
- Ei skadd zip-fil frå Tautulli stoppa opplastinga før fila vart fjerna, og fila, som held tilgangstokena til alle Plex-brukarane, vart liggjande til opprydjinga etter seks timar.
- Ein import eller ei gjenoppretting som vart stansa brått medan fila kom inn, la att fila i datamappa for godt, adresser og alt; no vert ho fjerna ved neste oppstart.
- Eit bilete kunne verte sendt halvskrive medan det vart lagt i cache, og nettlesaren heldt då på det slik.
- Tilbakekalla API-nøklar vart aldri sletta, same kor mange som vart laga og tilbakekalla.
- Ei lesing av avspela- og favorittmerka til nokon som kom tilbake tom eller ulesleg, fjerna alle saman til neste lesing; ho vert no avvist, slik ei tom lesing av eit bibliotek vert.
- Ein episode med eit sluttnummer ingen episode kan ha (ein dato skriven der) kunne halde Bibliotekhelse oppteken utan ende etter ei lesing av biblioteket.

### Fixed
- Ein endringsmerknad med kode inne i feit skrift (i 2.0.2 og 2.0.3) viste backtick-teikna i staden for å teikne koden.
- Eit diagram over tal kunne nummerere aksen 0, 3, 5, 8, 10, eller til og med 0, 0, 1, 1, 1: steg på 2,5 skrivne avrunda. Tal stig no i heile steg.
- Ein kalendernøkkel laga av nokon utan løyvet Administrere kom med ei lenkje på adressa i nettlesaren i staden for den offentlege adressa til FinStats, så han verka berre innanfrå; sida spurde òg etter innstillingar ho vart nekta.
- Den siste avspelinga til ein person kunne vere nokre sekund for kort til å telje som avspeling nokon annan stad, og nemnde då ein tittel som Historikk for personen ikkje viser.
- Ei gjenoppretting frå Nyleg sletta som feila, sa ingenting og let knappen snurre vidare; det same gjorde ei mislukka *Hent tilbake* i Bibliotekhelse og ei mislukka *Opne på nytt* eller *Løys alle* i Tryggleik.
- Kalenderabonnementet heiter FinStats i ein kalenderapp, ikkje finstats.
- Nedlastingskøa viste kven som hadde bede om kvar nedlasting til alle som fekk sjå ho; førespurnaden til nokon andre krev no *Sjå alle*, som i På veg.
- Å ta *Varsle* frå ein person stoppa ikkje det mottakarane til personen fekk tilsendt, og personen kunne ikkje lenger slå dei av.
- Ei avspeling som vart sett på pause og teken opp att innanfor samanslåingsvindauget, vart varsla som stoppa éin gong, medan gjenopptakinga og den verkelege slutten aldri vart varsla.
- Avspelingane av eit spor kunne verte flytta over på eit spor med same namn av ein annan artist («Intro») når albumet forsvann frå biblioteket.
- Ein varselmottakar skriven som ei IPv4-adresse inne i IPv6 (NAT64, 6to4) slapp gjennom som offentleg sjølv om han nådde det lokale nettverket.
- Ei Sonarr- eller Radarr-tilkopling som vart peika mot ein annan server, viste plakatane frå den gamle serveren i ei veke til.
- Varsla i Tryggleik sa at ingenting trong ein kikk når to eller fleire personar var valde, og ei side som vart tom då det siste varselet på ho vart løyst, hadde ingen veg tilbake.
- Kvar API-nøkkel som ikkje hadde gått ut, sa at han gjekk ut «akkurat no».
- Ein dialog vart liggjande over sida etter Tilbake, og sida ville ikkje rulle før han var lukka.
- Revisjonsloggen viste åtte typar oppføringar (ei avspeling eller ein tryggingskopi henta tilbake, papirkorga tømd, offentlege profilar, ein endra utsjånad) med ID-ane sine og ingenting under Detaljar.
- Å leggje ein tittel til i Lista mi frå kontekstmenyen feila utan eit ord når serveren nekta.
- *Sett saman* på Oversikt handsama fleire personar valde samstundes som éin person.
- Tilbakeblikket skreiv timar og dagar under ti med desimalkommaet til nettlesaren ved sida av sine eigne komma som tusenskiljeteikn.
- Kapasitet sa at ein disk ville bli full "i tempoet frå dei siste 0 månadene".
- På sida Tryggleik talde adressene utan stad avspelingane til alle gjennom all tid, same kva person og periode sida var avgrensa til.
- Nokon med *Administrere*, men ikkje *Sjå alle*, kunne leggje avspelinga til kven som helst i papirkorga.
- Eit kort frå Tilbakeblikk eller ein offentleg profil kunne kome frå snøgglageret i nettlesaren på språket som var valt før.
- Nedlastingane direktekanalen sende, var på britisk engelsk, same kva språk sida var på.
- Ei pålogging som endra kva nokon får sjå, nådde dei andre opne fanene deira først ved neste kontroll etter eit halvt minutt.
- Ei fane som gav frå seg direktesambandet til ei nyare, tok eit tilbake på neste side, så fanene til éin person heldt fram med å stengje kvarandre sine.
- Tabellar sorterte storleikar frå 1 000 til 1 023 einingar som tekst på språk som grupperer siffer med mellomrom, og prosentar på tyrkisk likeins.
- Ein tittel med ein plasshaldar i namnet, som "Big {device} Bunny", vart fylt ut av meldingane til tenaren.
- Tabellar sorterte lengder og tal av eit heile (5 av 26) som tekst på alle språk unnateke engelsk og dei nordiske.
- Ei fane som framleis var open på eitt språk, fekk svar frå tenaren på eit anna så snart ei anna fane hadde valt det.
- Eit ord ved sida av eit tal fekk feil form på polsk, russisk og ukrainsk ("5 wieczory", "2 потоков").
- Kapasitet rekna den første fyllinga av eit bibliotek som vekst, og delte det kvart bibliotek la til på seks månader, same kor lenge det var målt over; ei fil som tok plassen til ei anna (ei oppgradering frå Radarr eller Sonarr, eller ei fil Jellyfin gav ny id) talde som heilt ny, når berre skilnaden var ny.
- Folk som såg saman med SyncPlay vart ikkje ei gruppe når ein vart med nokre minutt etter at den andre hadde starta; avspelingar som sluttar samstundes på same stad, høyrer saman no, og historikken vert gruppert på nytt éin gong.
- Nokon utan *Sjå alle* kunne sjå av *Aldri sett* for eit bibliotek at ein annan hadde spela av ein tittel; no tel han berre deira eiga vising.

## [2.2.0] - 2026-10-01 to 2026-10-07

Bibliotekhelse og FinUI

### Added
- **Ein eigen meny på datamaskina**: Innstillingar → Utsjånad tilbyr åtte: sidepanelet slik det var, gruppert etter kva sidene er til, ei skjene med ikon som opnar seg til sidepanelet under peikaren, eit sidepanel som byrjar med søk, ei dokk, ei kommandolinje, sider du festar øvst, og farga fliser. Sidepanela kan stå til høgre. Dokka glir bort medan du rullar nedover og kjem att når du rullar oppover eller peikar mot botnen av vindauget; med ho flyttar statuslinja seg til toppen.
- **Ein eigen meny på telefonen**: ei fanelinje med Meir (den nye standarden), eit ark som opnar seg halvvegs, heile skjermen, sidene spreidde i ei vifte rundt tommelen, eller ei adresselinje som veks til menyen. Kvar av dei søkjer i sidene sine og sender alle andre ord vidare til søket.
- Begge vert hugsa av nettlesaren dei vart valde i, så telefonen og datamaskina held kvar på sitt; begge er FinUI-komponentar, `desktop-nav` og `mobile-nav`.
- **Ein kontekstmeny**: høgreklikk på ein tittel, ein person eller eit bibliotek kvar som helst i FinStats (ein plakat på Oversikt, ei rad i ei liste, eit namn i ein tabell, ein film i Kjem snart som enno ikkje er i biblioteket), hald fingeren på det, eller trykk på menytasten, og ein meny veks ut frå punktet du trykte på: opne, opne i ny fane, legg ein film eller ein serie til i Lista mi, opne i Jellyfin eller på TMDB og TVDB, kopier lenkja. Shift med høgreklikk gjev framleis menyen til nettlesaren. `context-menu` frå FinUI.
- **Søk utan dialog**: søket veks ut frå menyen din (eit sidepanel vert breiare, dokka stig opp, namnet i kommandolinja vert til søkjefeltet, menyen på telefonen held det) eller, om du vel det i Innstillingar → Utsjånad, tek plassen til sida medan menyen vert verande. Ctrl+Space opnar det som før; vindauget over alt saman er borte. `search` frå FinUI.
- **Alt i eit bibliotek**: kortet Nyleg lagt til for eit bibliotek tilbyr *Alt i det*: alle filmane, seriane eller albuma det har, som plakatar, etter namn, år, dato lagt til eller storleik, avgrensa medan du skriv. Fleire kjem til medan du rullar.
- **Server → Bibliotekhelse**: FinStats samanliknar kvar fil med naboane hennar og listar det som skil seg ut, kvart funn med tala bak. Det rapporterer berre; ingenting vert endra, sletta eller skanna på nytt i Jellyfin.
- **Hol**: episodar som manglar mellom to filer i ein sesong («Sesong 2: episode 4 og 5 manglar mellom 3 og 6»). Ei fil som held to episodar, tel som begge; spesialepisodar og slutten av ein sesong vert ikkje rørte.
- **Sesongar som skil seg** frå resten av serien i oppløysing, HDR eller kodek, og **avvikande episodar**: ei fil som ikkje liknar resten av sesongen, til dømes ein SD-episode i ein 1080p-sesong. Oppløysinga går etter breidd eller høgd, så ein beskoren film på 1920×800 er 1080p.
- **Kopiar og versjonar**: den same filmen eller episoden to gonger, etter dei leverandør-ID-ane som Lista mi og På veg alt går etter. To kopiar i same oppløysing viser plassen som går med to gonger; ein 4K-kopi ved sida av ein 1080p-kopi vert lista som to versjonar og aldri rekna som sløsing.
- **Tynne filer**: ein bitrate langt under det oppløysinga og kodeken vanlegvis krev («4K HEVC på 1,1 Mbps»); dei tynne episodane i ein sesong er eitt funn.
- **Dubbing som stoppar**: eit lydspråk som nokre sesongar har på kvar episode og andre på ingen.
- **Aldri identifisert**: filmar og seriar Jellyfin har kjent att utan TMDB-, TVDB- eller IMDb-ID.
- **Avvis** eit funn som er i orden med vilje, med ein merknad. Det kjem att av seg sjølv når filene endrar seg. Dei som kan administrere FinStats og sjå serveren, kan avvise; avvisingar står i revisjonsloggen og er med i ein tryggingskopi.
- Sida til eit bibliotek seier kor mykje det er å sjå på der, og leier til det.
- **FinUI**, komponentane FinStats er bygd av, er eit eige prosjekt (`github.com/finstats/finui`): kvar knapp, kvart kort og kvar kontroll i alle tilstandar og begge tema på `finui.finstats.no`, og **FinUI create** for å endre fargar, hjørneradius, tettleik og meir medan du ser på, og ta det med heim med éin kommando. **Innstillingar → System** leier dit.
- **Innstillingar → Utsjånad**: alle vel korleis FinStats ser ut for dei, med koden til eit førehandsval frå FinUI: grunnfarge, aksentfarge, diagramfargar, kontrast, hjørneradius, tettleik, kantar, kort, knappar, felt, tabellar, utheving, rørsle, ikon, menyen, sida bak alt saman, fokusringen, korleis overskrifter er sette, og skriftene for tekst, overskrifter og tal. Valet ditt gjeld berre deg; sida viser kva koden tyder og leier til FinUI create for å endre han.
- **Stilar**: elleve heile utsjånader å byrje frå med eitt val i FinUI create, og Innstillingar → Utsjånad tek imot koden. Dei er Washi (FinStats slik det vert levert), Studio, Noir, Matinee, Console, Pastille, Ledger, Gazette, Arcade, Canopy og Harbour. FinStats sjølv kjem med éin utsjånad og tilbyr inga liste over stilar.
- **Ikon som rører seg**: kvart ikon i ei lenkje, ein knapp eller ei fane gjer si eiga rørsle éin gong når du peikar på det (oppdater snurrar, pila i last ned fell ned i korga si, papirkorga lyftar lokket), og ei oppdatering du har trykt på, held fram med å snurre til svaret er kome. Står stille med redusert rørsle og med Motion: Off.
- **Rørsle, med FinMotion**: komponentane i FinUI rører seg no: knotten på ein brytar vert kasta over, streken under ei fane krabbar bort til neste fane, eit felt som vert avvist, ristar på hovudet, ein kopi vert kryssa av med eitt strøk, meldingar ligg i bunke som korta på ei hand og breier seg ut når du peikar på dei, eit ark på telefonen gjev etter som ein strikk når du dreg i det, radene i ein sortert tabell flyttar seg til plassane sine, eit tal som endrar seg, rullar som ein kilometerteljar, og diagram teiknar seg sjølve første gongen dei vert viste. Alt saman følgjer valet Motion i Innstillingar → Utsjånad og står stille med redusert rørsle. FinMotion er eit eige prosjekt (`github.com/finstats/finmotion`), lagt oppå FinUI.
- **Fleire ikon i FinUI**: 239 no, opp frå 71, kvart med si eiga rørsle (volum opp og ned, eit hovudsett og eit dempa hovudsett, ein blyant som skriv, ei bjølle som ringjer, ein telefon som surrar), og alle kan lastast ned frå galleriet til FinUI som SVG, stille eller animerte.
- **Skrifter**: trettifire skriftfamiliar med open lisens å velje mellom (Geist, Roboto, Manrope, IBM Plex Sans, Montserrat, Lora, Playfair Display, Fira Code og fleire), pakka med slik Inter er, aldri henta frå andre stader, og lasta berre når dei er valde.
- **Fleire grunnfargar og aksentfargar**: Nord, Solarized, Gruvbox, Catppuccin, Rosé Pine, Tokyo Night, Dracula, Everforest og Ocean; seksten nye aksentfargar, ti av dei med éin farge om dagen og ein annan om natta. Kvar grunnfarge har no sitt eige papir om dagen: dei grå såg like ut i det lyse temaet.
- **Sletting kan angrast i 30 dagar**: ei sletta avspeling eller ein sletta tryggingskopi ventar i papirkorga, og **Angre** i meldinga som kjem etterpå, hentar det tilbake. Aktivitet og Innstillingar → Tryggingskopiar viser kvar for seg det som nyleg vart sletta, og når det forsvinn for godt, så sant det finst noko. Ei avspeling i papirkorga er halden utanfor alle summar, diagram, tilbakeblikk og profilar, og ein import eller ei gjenoppretting hentar ho ikkje tilbake som ny; den som såg ho saman med nokon, vert rekna som åleine til ho kjem tilbake.

### Changed
- Dark Reader og liknande utvidingar lèt sidene til FinStats vere (`darkreader-lock`): FinStats teiknar begge tema sjølv, og ei side som vart måla om, kjempa mot dei.
- På telefonen glir ikkje menyen lenger inn frå venstre: fanelinja står nedst om ikkje ein annan meny er vald.
- FinStats les biblioteket ditt éin gong etter denne oppdateringa for å lære kvar filer som held fleire episodar, sluttar.
- På sida til ein tittel er **Legg til i Lista mi** like høg som **Opne i Jellyfin** ved sida av; han var ein storleik mindre.
- Filtera for tidsrom og personar vert verande øvst på skjermen medan sida rullar, så 7 d vert Alt frå botnen av ei lang side; det same gjer filtera for førespurnader og Kjem i På veg.
- Knappen for å rulle til toppen held seg unna den menyen som har botnen av vindauget: lågt i hjørnet ved sida av dokka, rett over fanelinja, tommelknappen eller adresselinja på telefonen, og elles over statuslinja. Han er `to-top` frå FinUI.
- Ei avspeling eller ein tryggingskopi som vert sletta, går til papirkorga i staden for å verte fjerna med ein gong (`PUT` med `{"deleted": true}`; `DELETE` er fjerna frå API-et). Tryggingskopiar ut over talet som vert teke vare på, vert framleis fjerna for godt.
- **Endringslogg** er eitt ark i staden for ein stabel med kort: kvar merknad er ei eiga linje med typen sin (Nytt, Endra, Retta og resten) og ikonet for typen i ei kolonne framfor, under ei overskrift for kvar serie. Lista over versjonar står framleis til høgre.

### Performance
- Når du opnar FinStats, vert berre det som er endra, lasta ned: eit stilark, eit skript eller ei skrift som nettlesaren alt har, vert stadfesta med nokre få byte i staden for å verte sendt på nytt. Før henta kvart besøk alle stilarka og modulane til FinStats i heilskap, sjølv om kvar av dei hadde ein tagg som skilde dei frå kvarandre.

### Stability
- Gjenoppretting av ein tryggingskopi les innstillingane berre frå rada for innstillingar: ei fil frå ein nyare FinStats som har meir der, fekk innstillingane sine forkasta utan eit ord.
- Ei open fane veks ikkje lenger for kvar side du har vore på: kvar side du gjekk frå, vart liggjande i minnet bak den på skjermen (om lag 2 600 element per besøk), halden fast av lista FinMotion har over det som skal rørast, og av rullelyttaren til filterlinja, til fana vart lukka.
- Ein feilskriven kommando (`finstats bakcup`), eller `restore` utan fila si, lagar eller oppgraderer ikkje lenger ein database før FinStats seier at det ikkje veit kva det skal gjere.

### Fixed
- Den vesle grafen i ei flis går ikkje lenger ut over flisa når talet ved sida av er breitt («56 d 23 t»), eller når fire fliser deler ei smalare rad: grafen gjev etter, og talet held breidda si.
- I det lyse temaet vart ein primærknapp fiolett under peikaren; no held han fargen til seglet.
- I Innstillingar står ein brytar på linje med namnet og hjelpeteksten til innstillinga; han hang øvst i rada.
- I ei rangert liste utan noko under namna, som Toppbrukarar på Oversikt, stod namnet øvst i rada ved sida av eit sentrert bilete; no er det sentrert.
- Ei rad med statistikkfliser i ei breidd mellom telefon og datamaskin (fire fliser, tre i breidda) la att den siste åleine på ei eiga rad; fire fliser står no fire, to eller ei i breidda, og tre står tre eller ei.
- Lesinga av serveren talde med ei eining som ingen hadde logga inn på, og som FinStats ikkje tek vare på: oppgåva sa tre einingar der Server-sida lista opp to.
- Eitt sekund, eitt minutt, éin time: varsla sa «1 seconds apart» og «in 1 minutes», og ei Jellyfin-oppgåve med intervall på eitt minutt køyrde «every 1 minutes».
- `finstats --help` nemner kvar variabel FinStats les; `FINSTATS_PUBLIC_IP_URL`, `FINSTATS_ALLOW_LIBRARY_SHRINK` og `FINSTATS_SKIP_PREUPDATE_BACKUP` mangla.
- Oppsettsvegvisaren sa «kunne ikkje kople til» om noko som ikkje var ei adresse, i ei setning som rann inn i den neste; no seier han at det ikkje er ei adresse.
- I Innstillingar → Tryggingskopiar forsvann eit spørsmål som vart stilt medan ein tryggingskopi vart skriven, når tryggingskopien var ferdig.
- Med menyen som ei skjene som opnar seg, synte det å lukke søket både det opne søket og den lukka skjena samstundes, og så stod skjena open til peikaren kom og gjekk. Esc gjev no fokuset til tastaturet tilbake dit det var då søket opna seg, og slepp fokuset eit klikk gav.
- På telefonen heldt sida seg dempa til søket var lukka heilt, og hoppa så til full lysstyrke i eitt bilete; no tonar dempinga ut saman med søket.
- Stega for Tautulli-importen sa at tryggingskopien vert lasta ned etter **Backup Database**. Tautulli lagrar han på si eiga maskin: stega seier no kvar (mappa `backups`, `/config` med Docker), kva fil du skal ta, og at du skal kopiere ho over.
- Søk som vart lukka før det var ferdig med å opne seg, tok animasjonen frå skjena som opnar seg: frå då av opna og stengde ho seg med eitt.
- Skjena som opnar seg, vart ståande open etter at ei side var vald frå ho, til eit klikk ein annan stad, og i nokre nettlesarar, eller med zoom, teikna ho eit rullefelt under ikona. No opnar ho seg berre for peikaren og tastaturet, og teiknar ikkje noko rullefelt når ho er lukka.
- På Tryggleik-sida tel ei adresse som databasen plasserer i eit land, men ikkje på kartet, med for det landet, i staden for blant adressene utan kjend stad.

## [2.1.2] - 2026-10-01

### Added
- **Import frå Tautulli**: ta med deg historikken frå Plex. Last opp databasetryggingskopien frå Tautulli (`.db`-fila, eller `.zip`-fila som held ho) under **Innstillingar → Import**; avspelingane får sin eigen plass i filteret *Registrert av* i Aktivitet.
- **Eit koplingsbrett for å seie kven som er kven**: Plex-brukarar på den eine sida, Jellyfin-brukarar på den andre. Dra ein leidning frå kvar Plex-brukar til den dei er no, eller klikk på den eine og så den andre, eller bruk tastaturet. Den som står utan leidning, vert ikkje importert, og to Plex-kontoar kan gå inn i éin person.
- Filmar og episodar vert kopla til biblioteket ditt etter namn (ein film etter tittel og år, ein episode etter serie og nummer), og ein som enno ikkje er på serveren, vert teken vare på og kopla på når han kjem. Ei vising som vart teken opp att, er éi avspeling; musikk vert ikkje importert.
- Tryggingskopien held tilgangstokena til Plex: FinStats les dei aldri, og fjernar fila så snart du importerer eller byrjar på nytt.
- **Ukopla media** under **Innstillingar → Data**: kvar tittel frå Tautulli, Jellystat eller Streamystats som ikkje passar med noko i biblioteket ditt, med kvar han kom frå og kvar han truleg alt finst. **Finn** koplar han frå dei beste gjetningane eller frå eit søk (ein film kan plasserast som spesialepisode i ein serie), og FinStats hugsar valet, så den same historikken importert på nytt koplar seg sjølv. Vala er med i ein tryggingskopi.

### Performance
- **Tilbakeblikket opnar tre til fem gonger raskare**: dei mest sette skodespelarane og regissørane i året vart rekna ut ved å gå gjennom året på nytt éin gong for kvar person i det. Målt på ein verkeleg historikk gjekk eit år frå 0,29 s til 0,09 s og dei siste 12 månadene frå 0,53 s til 0,12 s.

### Fixed
- **Importert historikk finn titlar som går under eit anna namn**: ei avspeling passar òg med namnet på originalspråket, så ein serie Plex eller Tautulli kjende som 오징어 게임, vert kopla til Squid Game, og *Im Westen nichts Neues* til *All Quiet on the Western Front*. Originaltitlane i biblioteket ditt vert lesne ved neste skanning.
- Namn vert samanlikna slik søket samanliknar dei (store og små bokstavar, aksentar, teiknsetjing, eit årstal skrive inn i namnet, ein innleiande «The»), og utgjevingsåret kan vere eitt år feil, så «Star Wars: Episode I - The Phantom Menace» finn «… – The Phantom Menace», og ein Kingsman frå 2014 finn den frå 2015. Ein tittel som er kjend under eit heilt anna namn, vert framleis aldri gjetta.
- Ein mislukka Streamystats-import eller ei mislukka gjenoppretting vart meldt i varsla med det interne namnet sitt.
- Knappen *Del året som kort* i Tilbakeblikket og sjåarplasseringa ved sida av hadde ikkje mellomrom mellom seg.

## [2.1.1] - 2026-10-01

### Changed
- **Lista mi vert tilboden overalt der ein tittel er**: på korta i Kjem snart på Oversikt, slik som i På veg → Kjem, og på sida til ein episode eller ein sesong, som set serien på lista.
- På ein plakat på Oversikt viser knappen for Lista mi seg medan du peikar på kortet eller når det med tastaturet, i staden for å dekkje hjørnet på kvar plakat; på ein trykkskjerm er han alltid der.

### Stability
- Å lagre ei Sonarr-, Radarr- eller Seerr-tilkopling medan nokon hadde **Innstillingar → Utgåande** open, kunne la begge vente på kvarandre for godt.
- **Innstillingar som ikkje kan lesast, stoppar FinStats ved oppstart** og nemner innstillinga det gjeld, i staden for å stille alle innstillingane tilbake til standard i det stille.
- **To endringar i innstillingane samstundes** (til dømes innstillingssida og tidsplanen for ei oppgåve) opphevar ikkje lenger kvarandre.
- Ein pause, ei spoling eller eit sporbyte som vart sett medan databasen var oppteken, vert skrive ved neste runde i staden for å gå tapt.
- Ei lesing av biblioteket feilar ikkje lenger når Lista mi til nokon har fleire oppføringar som viser seg å vere same tittel.

### Fixed
- **Ei utbytt fil er ikkje «no på serveren»**: å oppgradere fila til ein film på Lista mi til nokon seier ikkje lenger at filmen nettopp har kome.
- Éi mislukka lesing av programtillegga, dei planlagde oppgåvene eller lagringa i Jellyfin tømmer ikkje lenger den lista på Server-sida fram til neste lesing.
- **Ei Streamystats-avspeling stoppar der ho stoppa, på kvar side**: fråfallskurva for ein tittel og «fullført» for ein episode gjekk i staden etter lengda og rekna ho som anslått.

## [2.1.0] - 2026-09-29 to 2026-09-30

Lista mi

### Added
- **Ei liste som er di eiga**: set ein film eller ein serie på ho frå sida til tittelen, frå Kjem, frå Nyleg lagt til eller frå søket (Ctrl+Enter), og finn ho under **Lista mi** på din eigen profil.
- **Kvar oppføring seier kvar ho står no**: på serveren, «5 av 26 episodar», førespurd, «Kjem fredag», ikkje på serveren, forsvann frå biblioteket, eller sett. Det vert rekna ut kvar gong du ser på lista, med den same tydinga av «sett» som framdrifta di i seriar. Sette titlar vert verande, i ei eiga gruppe, til du tek dei bort.
- **Ein tittel som enno ikkje er på serveren**, kan kome på lista rett frå det Sonarr eller Radarr ventar på. Oppføringa vert kopla til tittelen av seg sjølv når han kjem, og ei fil som får nytt namn, tek oppføringa med seg.
- **«På Lista mi: no tilgjengeleg»**: eit varsel til dine eigne mottakarar når noko du venta på, kjem. Av til du kryssar av for det.
- **Ingen andre ser lista di**, ikkje ein gong ein administrator, og varselet går berre til dine mottakarar. Ingenting vert skrive til Jellyfin, Seerr, Sonarr eller Radarr.
- Listene er med i ein tryggingskopi, og ei gjenoppretting slår dei saman utan å doble noko.

### Changed
- `/api/upcoming`-oppføringar har TVDB- og TMDB-ID-ane til tittelen sin, og `/api/auth/me` seier om serveren har Lista mi.

## [2.0.4] - 2026-09-29

### Added
- **Planlegging av oppgåver, slik Jellyfin gjer det**: kvar jobb FinStats gjer på eiga hand, har utløysarar (dagleg på eit klokkeslett, kvar veke på ein bestemt dag, med eit intervall, ved oppstart eller etter biblioteksskanninga i Jellyfin), kvar med ei valfri tidsgrense. Klikk på ei oppgåve for å sjå og endre timeplanen hennar; *Legg til utløysar* opnar den same dialogen som Jellyfin har.
- **Oppgåver er ein eigen seksjon** i Innstillingar, med siste køyring for kvar jobb i ord («Køyrde for 38 minutt sidan, tok 6 minutt») og ein Køyr-knapp; før låg han øvst i System.
- **Metadataendringar**: kvar time les FinStats det Jellyfin har lagra sidan sist (namn, omtalar, sjangrar, vurderingar, medverkande, fildetaljar og bilete), så ei endring i Jellyfin kjem fram utan å vente på ei biblioteksskanning.
- Tryggingskopi og geolokaliseringsdatabasen kan òg køyrast frå Oppgåver; Esc går tilbake frå timeplanen til ei oppgåve til lista.

### Changed
- **Temaet er ein brytar med tre stopp**, Lyst · Eining · Mørkt, der knappen som bladde gjennom dei, var før: klikk på ein tredjedel av han, dra knotten eller bruk piltastane. Knotten ber ikonet for valet, og ikonet glir inn når valet vert endra.
- **Utlogging ligg under Innstillingar → Konto**, som seier kven som er innlogga her; sidepanelet held på temabrytaren.
- **Dei gamle planinnstillingane er no utløysarar**: *Følg biblioteksskanninga i Jellyfin* med reserveintervallet sitt, *Ta tryggingskopi kvar N. dag* og GeoIP-brytaren *Hald databasen oppdatert* vart til dei tilsvarande utløysarane på oppgåvene sine, med verdiane du hadde. Innsamling, Tryggingskopiar og Tryggleik lenkjer til timeplanen i staden.
- Ein oppstart køyrer ikkje lenger alle jobbane på nytt: det vert teke vare på når kvar av dei sist vart ferdig.
- Ein Jellyfin-jobb som ikkje har noko overslag enno, viser «om lag … att», med prikkar som går rundt der tida skal stå, i staden for «ETA …».

### Performance
- **Kartet under Tryggleik heng ikkje lenger etter**: å dra og zoome flyttar kartet som alt er teikna, og det vert teikna på nytt éin gong når rørsla har stilna, og pulsen for det som spelast no, får ikkje lenger heile kartet til å teikne seg 120 gonger i sekundet medan ingen rører det. Ei draging gjekk frå 34 til 60 bilete i sekundet på ei treg datamaskin.

### Fixed
- **Ein plakat eller eit portrett som er bytt ut i Jellyfin, kjem fram i FinStats**: bilete vart mellomlagra ei veke på serveren og ei veke til i nettlesaren, same kva Jellyfin hadde fått sidan. No vert dei mellomlagra under biletmerket til Jellyfin, så eit nytt bilete er ei ny fil, og nettlesaren sjekkar at kopien hans er gjeldande kvar gong han viser eitt. Når eit portrett vert bytt ut, vert berre personen lagra på nytt, så Metadataendringar les personane òg.
- **Server → Jobbar viser eit overslag med ein gong sida opnar seg**: tida som var att, vart berre målt ut frå avlesingar sida sjølv gjorde medan ho var open, så kvart besøk byrja på «ETA …» til sida hadde sett prosenten flytte seg. No tek FinStats sjølv tida på ein Jellyfin-jobb som køyrer, kvart 10. sekund medan han køyrer, og sida får overslaget servert. Ein jobb som Jellyfin rapporterer på 0 % heile tida (den første fasen av ei biblioteksskanning), held på prikkane til han flyttar seg: ingenting vert gjetta.
- **Verktøytips på kartet kjem på prikken**: i Firefox kunne verktøytipset til prikken for det som spelast no, hamne langt unna, og der prikkar ligg over kvarandre (nokon som spelar av heime), var det ein piksel som avgjorde kva verktøytips som kom fram. No namngjev eitt verktøytips alt som ligg under peikaren.
- **Statuslinja har éi form når peikaren er over**: kvar lenkje (straumane, Repo, versjonen) lyser opp som den same vesle boksen, og prikkane mellom elementa sit midt i mellomromma. Med peikaren over versjonen lyste før òg prikken og mellomrommet framfor opp.
- **Ein filsti tek heile breidda av kortet sitt**: på Fil-kortet til ein tittel, og for mappene til Jellyfin på Server-sida, vart ein sti klemd inn i éi smal celle og rann nedover ni liner ved sida av tomrom.
- Ein dialog som tonar ut, slukar ikkje lenger eit klikk på sida under.

## [2.0.3] - 2026-09-28

### Security
- Server-sida gav ID-en til kvar eining til alle med *Sjå server*; ID-ar krev *Sjå nettverk* i tillegg, som alle andre stader.
- To løyvebrytarar som vart slegne om før den første var lagra, sende den same startlista to gonger, og den andre gjorde om den første: eit løyve som var teke bort, vart verande gjeve, og ein seksjon av den offentlege profilen som var slegen av, vart verande publisert, medan begge brytarane viste det som var valt.

### Stability
- **Berre éin import eller gjenoppretting om gongen**: to opplastingar sende samstundes kom begge gjennom kontrollen, og ei gjenoppretting såg aldri etter importar; den andre venta ut databasen og feila etter fleire minutt med opplasting.
- **Ei adresse med ei øydelagd `%`-koding opnar sida for «ikkje funne»**, i staden for å late ein lasteindikator gå for alltid.
- **Eit stort sidetal i ei liste gjev ei tom side**: talet flaut over og kom tilbake som den første sida igjen.

### Fixed
- **Tabellar sorterer tal rett i ein tysk eller norsk nettlesar**: «1,204» vart lese som desimaltalet 1,204 og ikkje som 1 204, og alt over ein million som tekst.
- **Tabellvisinga til eit diagram sorterer datoar, vekedagar og posisjonar etter tid**, ikkje etter bokstavane.
- **Nedlastingshistorikken seier kvar importane kom frå**: Sonarr og Radarr namngjev indekseraren og protokollen berre når ei utgjeving vert henta, så desse fordelingane var alltid tomme. Ein mislukka sesongpakke er éin feil, ikkje éin per episode.
- **Å slette ei avspeling som vart sett saman,** finn gruppene til tittelen på nytt, i staden for å late att ei «gruppe» på éin.
- **Tid saman er for éin person aldri meir enn kor lenge personen sjølv var med**: Saman-sida godskreiv personen heile kvelden.
- **Ein kveld i lag med andre over midnatt nyttårsaftan** (eller over kanten av «siste N dagar») høyrer heilt til tidsrommet han byrja i, i staden for å verte to kveldar åleine.
- **Importerte episodeavspelingar får namnet til episoden** der Jellystat berre tok vare på namnet til serien.
- **Geolokalisering finn bydatabasen ved sida av Country- og ASN-databasane** som `geoipupdate` skriv, i staden for å slå seg sjølv av.
- **Punktet det oftast vert spola tilbake til, følgjer filtera på sida**, slik tala gjer.
- **«Året ditt er klart»** går berre til folk som har noko i tilbakeblikket sitt.
- **Å løyse eit dempa varsel på nytt held stadparet dempa**; berre å opne varselet att opphevar ei demping.
- **Ein utgått API-nøkkel tel ikkje lenger med blant dei tjue.**
- **Grensa for økter lèt den eldste økta gå** sjølv når mange innloggingar kjem i same sekund.
- **Ei heimeadresse som oppslaget fann, vert verande** når den same vert teken bort frå lista du har skrive inn.
- **Bilete i revisjonsloggen og i nøkkellista**, der det før berre stod initialar.
- **Varsellista til ein administrator seier kven ein personleg mottakar høyrer til**, i staden for «Din» på alle; **Nyleg sendt** viser alle mottakarane av den siste hendinga.
- **Esc på ein tittel eller ein person tek deg aldri ut av FinStats**, og lukkar berre dialogen som ligg øvst.
- **Ei side vert teikna sjølv når ei lagring avbryt førehandslastinga ho venta på.**
- **Spelast no viser ein sjåar som har pausa og spolar,** der sjåaren er no, ikkje først når avspelinga held fram.
- **Å fjerne ei tilkopling oppdaterer kva sider som finst**, slik det gjer å lagre ei.
- **Byggjedatoen til geolokaliseringsdatabasen held på årstalet** i amerikansk engelsk.

## [2.0.2] - 2026-09-28

### Security
- Bak ein proxy talde innloggingsgrensa den første `X-Forwarded-For`-adressa, som er den klienten sjølv sender: ei ny oppdikta adresse for kvart forsøk, og grensa slo aldri inn. No er det den siste, den proxyen skreiv.
- *Ta med IP-adresser og stader* sende adresser til ein personleg mottakar der eigaren manglar *Sjå nettverk*, slik at eigaren fekk det inga side ville vist han. Mislukka innloggingar for andre krev no *Sjå server* òg, som på Tryggleik-sida.

### Stability
- **Ein stans går aldri tapt**: når ei lesing av biblioteket vart avvist i dei første augneblinkane etter oppstart, før serveren lytta etter det, vart avvisinga notert, og FinStats køyrde vidare i staden for å stanse.
- **Ein Jellyfin som svarar i ei form FinStats ikkje kan lese, vert avvist**, ikkje trudd: alle brukarar, eller alle titlar i eit bibliotek, som kom under nøklar FinStats ikkje kjenner (`id` for `Id`), slapp forbi vernet med fullt tal og vart så merkte som fjerna.
- **Slutten på ei avspeling vert aldri mista**: når databasen var oppteken lenger enn ventetida (ein import som heldt på han), vart ei avspeling som slutta akkurat då, ståande som i gang til neste omstart. No vert slutten skriven i neste runde.
- **Innstillingar som ei gjenoppretting ikkje kan lese, vert ikkje rørte**: ein tryggingskopi der innstillingane ikkje lenger passa denne versjonen, nullstilte alle innstillingane ved gjenoppretting: samanslåingsvindauget, løyve, heimeadresser, timeplan.
- **Eit kontrollteikn i eit namn eller ein tittel øydelegg ikkje lenger korta**: profilkortet, kvart historiekort som viste den tittelen, og ZIP-fila med dei svarte med ein feil.

### Fixed
- **Varsel stoppar for nokon som er deaktivert eller sletta i Jellyfin**: dei personlege mottakarane deira fekk framleis alt dei gamle løyva tillét.
- **Eit varsel i kø vert sjekka på nytt før det vert sendt**: ein mottakar som var slått av, eller ein eigar som hadde mista eit løyve, fekk framleis det som venta på eit nytt forsøk.
- **Ein mottakar som alltid svarar `Retry-After`, vert gjeven opp** etter fem forsøk som alle andre, i staden for å verte prøvd på nytt i tretti dagar.
- **Eining-ID-ar på sida til ein person** vert berre viste med *Sjå nettverk*, som alle andre stader.
- **Eit publisert år ventar eitt døgn med alt**: dei fullførte og droppa seriane i det følgde kvar avspeling, så å trykkje play på ein droppa serie endra den offentlege sida i løpet av sekund.
- **Ein serie nokon har bede om, tel som sett** når ein av episodane er sett: kvar serie det var bede om, vart lesen som aldri sett, i kapitlet Førespurnader og på kortet Bede om.
- **Kapitlet Saman i tilbakeblikket held direkte-TV utanfor**, som resten av året: ein kanal som stod på medan folk såg saman, kunne verte den største tittelen, og delen kunne gå over 100 %.
- **Dagar-kortet for «dei siste 12 månadene» sluttar på den nyaste dagen**: rutenettet mista dei siste tre eller fire vekene.
- **Nye episodar av ein serie vert kunngjorde éin gong per kveld**: episodar på kvar si side av midnatt UTC vart kunngjorde to gonger på ein server aust eller vest for han.
- **Å kople til Seerr kunngjer ikkje gamle førespurnader som klare til å sjåast**: ein førespurnad som alt var tilgjengeleg då han vart lesen første gong, og som ikkje hadde noko anna som sa når, vart lesen som om han kom i same augneblinken.
- **Titlane i eit bibliotek som er sletta i Jellyfin, forsvinn med det**: dei vart verande i søk, på hylla Nyleg lagt til og i totalane.
- **Ein remux held seg som direktestrøyming** når ei avlesing mistar omkodingsdetaljane ein augneblink (ved ei spoling), i staden for å verte omkoding resten av avspelinga.
- **«Jellyfin er nede» tyder at fleire lesingar feila på rad**: einskilde tidsavbrot med dagars mellomrom, på ein server som låg roleg på socketen, vart lagde saman til det.
- **Ein Streamystats-import viser framdrifta si på sitt eige kort**, ikkje over det siste resultatet frå Jellystat-importen.
- **Ei adresse der vertsnamnet byrjar med «web»** (`http://web:8096`) vert lesen som eit vertsnamn, ikkje kutta av som om det var stien til nettklienten til Jellyfin.
- **Ein køfeil gøymer heile innloggingsdelen** av ei adresse sjølv når passordet inneheld ein skråstrek.

## [2.0.1] - 2026-09-28

### Changed
- **Språka til ein tittel står ikkje i vegen**: toppfeltet namngjev dei tre første lyd- og undertekstspråka, éi line for kvart slag, og eit nytt **Språk**-kort listar kvart einaste eitt: dei fullstendige saman, dei delvise med kor mange episodar som har dei. Dei to dusin undertekstspråka til ein lang serie skuva før biletet ein heil skjerm nedover.

### Fixed
- **Plakaten til ein tittel held seg ved sida av namnet**: når teksten ved sida av vart lengre enn plakaten (eit langt samandrag, mange studio), sokk plakaten ned til botnen av toppfeltet, under tomt bakgrunnsbilete.
- **Ein versjon FinStats ikkje kan lese, vert kalla det han er**: ein `app_version` som var handredigert til noko ulesleg, fekk FinStats til å nekte å starte fordi databasen skulle vere opna av «den eldre FinStats 2.0.1», noko som ikkje stemde. FinStats nektar framleis, men seier no kva verdi som ikkje kunne lesast, og korleis han skal skrivast.

## [2.0.0] - 2026-09-25 to 2026-09-28

Tilbakeblikk 2026, offentlege profilar og å sjå saman

### Added
- **Kven du såg saman med**, i tilbakeblikket: kveldar i lag med andre, tida brukt slik, tittelen som oftast samla folk, og opptil tre følgjesvenar, namngjevne i appen og lenkja til profilane deira.
- **Seriar sett til endes**, i tilbakeblikket: den siste episoden av kvar serie, sett det året, etter den same tydinga av «sett» som framdriftslinjene på profilen, og òg dei som vart byrja på og lagde til sides, under halvvegs sett og utan avspeling på to månader.
- **Det du bad om**, i tilbakeblikket: Seerr-førespurnader gjorde i løpet av året, kor mange som kom, og kor mange av dei du så såg. Berre der det finst førespurnader.
- **Året mot året før**: timar, avspelingar og dagar, og kor mykje meir eller mindre.
- **Året som ei historie**: kvart kapittel som eit kort på 1080×1920, formatet som historier på mobilen brukar (året, tala, toppseriar, filmar og musikk, sjangrar, personlegdomen, timar og dagar, kvar dag i året, rekordar, selskap, fullførte seriar, førespurnader og året før). Lagre eitt, eller alle som ein ZIP, frå «Del året som kort».
- **Eit kort namngjev aldri nokon annan**: korta vert teikna frå ein kopi av året som ikkje har felt for ein annan person, ei rangering eller ein app. I appen har eit kort ikkje noko namn i det heile.
- **Året til serveren**, for Jellyfin-administratorar: kvar tittel og kvar total og personlegdomen til huset, utan at nokon vert rangert eller namngjeven. Ein brytar på tilbakeblikksida, `scope=server` i API-et.
- **«Tilbakeblikket ditt for året er klart»**: eit varsel i desember, éin gong til kvar person som såg noko det året.
- **Offentlege profilar**: ein person kan publisere delar av profilen sin på ei lenkje alle kan opne utan konto (totalar og topptitlar, rekkjer og kva tid på døgnet det vert sett, heile året som historiekort, dei ti siste avspelingane), kvar del slått av til eigaren slår han på under Innstillingar → Offentleg profil. Einingar, adresser, appar, avspelingsmåtar, filstiar og andre personar vert aldri publiserte, same kva som er slått på, og namnet som vert vist, er eit eigaren skriv inn, aldri brukarnamnet i Jellyfin.
- **Ingenting på ei publisert side er nyare enn eitt døgn**, både nylege avspelingar og alle totalar, så ei side nokon held auge med, seier aldri kven som ser akkurat no.
- **Kort til deling**: kvar publisert profil kjem med eit bilete på 1200×630, av profilen og av året, som Discord, Signal, Mastodon og resten viser når lenkja vert limt inn, og som kan lastast ned og leggjast ut. Teikna på serveren frå det sida viser, og ikkje noko meir.
- **Ei lenkje du kan ta tilbake**: 22 tilfeldige teikn, nullstilt med eitt klikk; den gamle lenkja svarar då nøyaktig som ei som aldri har funnest.
- **Offentlege profilar er slått av til ein administrator tillèt dei**: éin brytar for heile serveren, ei liste over kven som publiserer kva, og ein måte å ta ned ein profil på. Publisering, endringar, nullstillingar og nedtakingar står i revisjonsloggen, og ein nøkkel kan ikkje gjere noko offentleg.
- **Saman**, ei eiga side: kven som ser med kven, kor mykje, og korleis det har endra seg. Tid saman, kveldar og delen av visingstida i lag med andre, kvar mot tidsrommet før; timar i lag mot timar åleine per dag eller veke; kvart par av personar med kveldane, tida saman, når dei sist såg og kva dei ser mest; titlane som oftast vert sett saman; dei siste kveldane; og, for alle som kan sjå alle, tida kvar person ser åleine mot tida i lag med andre.
- Kvart par tel: ein kveld med tre tel for kvart av dei tre para, der kortet før berre talde den nøyaktige samansetjinga av personar. Tida saman for eit par er det kortaste av dei to opphalda.
- **Sett saman**-kortet på Oversikt og på ein profil lenkjer til sida, og kortet på ein profil seier kor stor del av visingstida til personen som var i lag med andre.
- `GET /api/stats/groups` får `pairs`, `series`, `people`, `previous`, `bucket`, og `watch_s` og `share` i totalane; nøklane det hadde frå før, er uendra.
- **Kvar folk stoppar**, på kvar film- og episodeside: delen av avspelingane som framleis held på ved kvart minutt, frå alle ved starten til dei som nådde slutten, med tilbakespolingane og undertekstpåslaga i desse avspelingane merkte på same aksen. Kortet seier kor mange stopp som er målte (avspelingar FinStats registrerte) og kor mange som er rekna ut (importerte avspelingar, frå kor lenge dei varte), og treng tre avspelingar før det teiknar noko.
- **Kven som held fram**, på ei serieside: kor mange som byrja på kvar episode, i rekkjefølgje, og ei **Fullført**-kolonne per episode. «Alle gjev opp etter episode tre» er folk som aldri trykkjer play på episode fire.
- **Filer som aldri vert spela av**, under Avspeling: ein film eller episode som er starta tre gonger eller meir og aldri kom forbi tretti sekund, med kven som prøvde og i kva appar. Loggen til Jellyfin har ingen avspelingsfeil, så avspelingane er vitnet; minste avspelingslengd gjeld ikkje her, sidan det er nettopp den desse avspelingane aldri når.
- **Mest tilbakespola** og **Undertekstar slått på**, under Avspeling: titlar rangerte etter hopp bakover per avspeling, med minuttet hoppa samlar seg i, og etter delen av avspelingar der undertekstar vart slått på innan dei første ti minutta. Alle tre listene krev *Sjå alle*. Hopp kortare enn 20 sekund vart aldri registrerte, så å høyre éin replikk ein gong til er usynleg med vilje.
- Ei spoling tek vare på kvar ho kom frå, som eit tal ved sida av merkelappen (`from_s` i tidslinja til ei avspeling). Kvar spoling som alt er registrert, får talet sitt frå merkelappen, og det same gjer ein gjenoppretta tryggingskopi frå før dette.
- `GET /api/stats/files` for dei tre listene; `GET /api/items/{id}` får `insights`, og episodane til ein serie får `users` og `finished`.
- **API-nøklar**: alle som er innlogga, kan lage nøklar til seg sjølve under Innstillingar → API-nøklar, til eit skript, eit dashbord eller ein telefon. Ein nøkkel vert send som `Authorization: Bearer fs_…`, vert vist éin gong, vert lagra som hash, kan ha ein utløpsdato og vert trekt tilbake med eitt klikk; ein administrator ser nøklane til alle og kan trekkje dei tilbake. Nøkkelen høyrer til den same personen med dei same løyva som ei innlogging, lese på nytt ved kvar førespurnad, så å miste retten til å logge inn stoppar alle nøklane med ein gong. Ein nøkkel kan verken lage eller trekkje tilbake nøklar.
- **Ein kalender å abonnere på**: `GET /api/calendar.ics?key=…` er det Sonarr og Radarr har på veg, som ein iCalendar ein telefon kan abonnere på. Ein nøkkel med omfanget *kalender* opnar kalenderen og ingenting anna; kalenderen les nøkkelen frå adressa, aldri frå informasjonskapselen, fordi ein abonnert kalender ikkje kan sende nokon header, og han namngjev ingen.
- **Ein revisjonslogg** under Server → Revisjonslogg, for Jellyfin-administratorar: kvar innlogging og kvart mislukka forsøk med adressa, kvar innstilling eller kvart løyve som vart endra og til kva, kvar nøkkel som vart laga, brukt eller trekt tilbake, kvar tilkopling, mottakar, tryggingskopi og import, kvar avspeling som vart sletta og kvart varsel som vart løyst, med kven, frå kvar, gjennom kva nøkkel og om det lukkast. Vert teken vare på i eitt år og er med i tryggingskopiar. `GET /api/audit` blar gjennom han.
- **Import frå Streamystats**, ein eigen seksjon under Innstillingar ved sida av Jellystat. Ta fila frå **Settings → Backup & Import → Download Backup** der og slepp ho inn: år med historikk kjem på eit par sekund, og ingenting du alt har, vert talt på nytt. `finstats import-streamystats <file>` gjer det utan grensesnitt, `POST /api/import/streamystats` gjennom API-et; éin import køyrer om gongen, same kva slag.
- **Har du brukt begge sporingsverktøya? Importer begge filene.** Ei avspeling vert kjend att som ei FinStats alt har, først på ID-en sporingsverktøyet sjølv gav ho, og elles på at same person ser same element med ein av endane av avspelinga nær ei i historikken, så ein Jellystat-eksport og ein Streamystats-eksport av dei same kveldane kan begge importerast, i kva rekkjefølgje som helst, og ein kveld FinStats registrerte sjølv, vert ikkje importert over heller. «Nær» er Samanslåingsvindauge under Innstillingar → Innsamling.
- Importerte rader seier ikkje meir enn dei veit. Ei avspeling Streamystats registrerte sjølv, tek ikkje vare på noko om fila (ingen kodek, oppløysing, bitrate eller container), så desse felta vert ståande tomme i staden for å fyllast med ei gjetting, og korta som ville vist dei, gøymer seg. Ei avspeling Streamystats hadde henta frå Jellystat, har med seg heile økta, og den vert lesen i staden: datert etter augneblinken ho slutta, den einaste augneblinken nokon av sporingsverktøya tok vare på, og skildra som omkoding berre når økta verkeleg var det.
- Rader Streamystats skreiv fordi Jellyfin melde eit element som sett, for ei sjåing Streamystats aldri såg, vert talde og hoppa over: dei er like lange som heile filmen og ville vore visingstid ein kveld ingen såg. Resultatet seier kor mange det var.
- Korleis Streamystats-data vert tolka, felt for felt, lært frå ein verkeleg eksport og ikkje frå dokumentasjon: `docs/streamystats-import.md`.
- **Kva sporingsverktøy kvar avspeling kom frå**, som filter under Aktivitet: det FinStats registrerte sjølv, det som kom frå Jellystat, det som kom frå Streamystats. Berre dei som historikken din faktisk har, vert tilbydde, og filteret er avgrensa som alle andre lister, så nokon som berre kan sjå sine eigne avspelingar, får berre vite kvar den eigne historikken kom frå.
- **Filtera under Aktivitet er nedtrekkslister du kryssar av i**, så du kan be om filmar *og* episodar utan musikk, to avspelingsmåtar eller to personar samstundes, i staden for éin av kvar eller alle. Medietype fekk **Anna** no som typane kan kombinerast.
- Fleire personar samstundes, overalt der brukarfilteret finst (Oversikt, Avspeling, Bibliotek, Tryggleik og resten), og kartet og all statistikk følgjer med. Eit brukarfilter snevrar framleis berre inn det nokon kan sjå: utan *Sjå alle* vert ein førespurnad låst til den som spør, same kven adressa namngjev og kor mange.
- **Eit lyst tema**: washipapir, med blekktekst og ein vermilionraud aksentfarge. FinStats følgjer den lyse eller mørke innstillinga på eininga til du vel eit tema sjølv.
- **Ein temaknapp ved sida av Logg ut** går gjennom Eining, Lyst og Mørkt. Valet vert teke vare på i denne nettlesaren og teke i bruk før sida vert teikna, så ho blinkar aldri med det andre temaet.
- **Opne i Jellyfin**: kvar film-, serie- og episodeside har ein knapp som opnar sida til tittelen i Jellyfin, i ei ny fane.
- **Adressa til Jellyfin for folk** under Innstillingar → Jellyfin: dit desse knappane peikar, for når adressa FinStats koplar seg til, ikkje er ei som ein nettlesar kan nå. Tomt felt held på adressa FinStats koplar seg til; berre ein Jellyfin-administrator kan setje ho.

### Changed
- **FinStats har fått ein ny heim**: `github.com/finstats/finstats`, og Docker-biletet er `ghcr.io/finstats/finstats`. Endre namnet på biletet i `docker run` eller compose-fila di for å halde fram med å få oppdateringar; datamappa er som ho er.
- **Innstillingar viser éin seksjon på skjermen om gongen.** Sida var fjorten kort i éi kolonne og seks hundre ord med hjelpetekst. No har kvar seksjon si eiga adresse (`/settings/collection`), ei liste til venstre flyttar mellom dei, og på ein telefon bryt lista om til brikker. Lenkjer til dei gamle ankera kjem framleis fram.
- **Ei innstilling er éi rad**: kva ho er, éi line som seier regelen, og kontrollen til høgre. Tal i ein seksjon vert lagra saman med éin knapp nedst; brytarar lagrar seg sjølve som før. Statuskorta (Oppgåver, Utgåande tilkoplingar, Database) og lenkja til lisensane ligg under **System**, og dei to importverktøya har ein eigen seksjon.
- **Finn ei innstilling** frå søkjefeltet øvst (`/` set fokus på det) med kva ord som helst i namnet; det seier kva seksjon innstillinga ligg i, og Enter tek deg til rada.
- **Innstillingar er for alle som er innlogga**: seksjonane som styrer serveren, er framleis for dei som får styre han, og ein person som berre hadde løyve til varsel, kan endeleg nå sine eigne mottakarar.
- **Server-sida viser éin seksjon på skjermen om gongen**, slik Innstillingar gjer: Oversikt (system og lagring), Jobbar, Einingar, Programtillegg, Logg og Revisjonslogg, kvar på si eiga adresse. Lenkjer til `/server#jobs` kjem framleis fram.
- **Serverloggen er ein seksjon på Server-sida** (`/server/log`) i staden for ei eiga side i menyen; `/events` sender vidare dit med filtera sine.
- Eit varsel om ein jobb som feila, ein tryggingskopi som feila eller ei tilkopling som slutta å svare, lenkjer til den innstillingsseksjonen det gjeld, i staden for til toppen av sida.
- Det biblioteket ditt kan fortelje om ei importert avspeling (kva slags ting det var, kva episode, speletida, kva bibliotek ho høyrer til), vert fylt ut etter **kvar** lesing av biblioteket, ikkje berre i den augneblinken ein import skjer. Historikk vert som regel importert før FinStats nokon gong har lese biblioteket, så desse avspelingane vart før verande gjetta eller ukjende for godt.
- Å gjenopprette ein FinStats-tryggingskopi legg ikkje lenger til ei avspeling som kom frå eit sporingsverktøy under ein annan ID, og held framleis på to ekte sjåingar av same ting med nokre minutt imellom.
- Detaljane til ei avspeling namngjev det sporingsverktøyet som henta ho inn, i staden for å kalle alt som er importert, ei Jellystat-avspeling.

### Performance
- **Ein omstart går nesten med ein gong, same kor stor historikken er.** Kvar oppstart rekna ut kven som såg saman over heile historikken og avgjorde på nytt kva avspelingar som var lokale, berre for å ikkje endre noko: 24 s og 5,5 s på ti millionar avspelingar. No gjer ein oppstart berre det den førre køyringa lét att, og alt på nytt berre etter ei oppdatering eller når innstillinga bak er endra.
- **Mange som byrjar samstundes, vert registrerte på sekund.** Kvar ny avspeling, kvar lagring av framdrift og kvar eining vart skriven kvar for seg, og kvar avspeling som slutta, vart lukka med sin eigen gjennomgang av heile historikken til tittelen; ein runde skriv no kvart slag samla. 100 000 avspelingar som startar samstundes, tek 3 s, der 25 000 som slutta samstundes, før tok over fire minutt.
- **Spelast no held seg rask med tusenvis som ser.** Å finne kven som ser saman, samanlikna kvar straum med alle andre og lista kvar følgjesvein med namn: førti sekund ved 10 000 straumar, og gigabyte med minne i tillegg. No vert straumar samanlikna innanfor same tittel, og kvar viser nokre få namn og kor mange andre.
- Å rekne ut kven som såg saman, skreiv om kvar gruppert avspeling i historikken kvar gong FinStats starta, importerte eller såg ei avspeling slutte; no skriv det berre det som er endra. Å finne kva avspelingar som har slutta, vert ikkje lenger tregare med kvadratet av talet som spelar.
- **Å kjenne att ei avspeling historikken alt har, går gjennom ein indeks**, så ein import av 150 000 avspelingar tek sekund, og kopling på nytt etter ei lesing av biblioteket går frå tittel til tittel i staden for å lese kvar avspeling medan databasen er låst.
- **Innsikt i avspeling og oversikta er raskare over heile tida**: dei uavhengige delane vert rekna ut side om side, diagrammet for samtidige straumar slår opp den lokale dagen éin gong per kvarter i staden for éin gong per avspeling, og sjangrar vert talde per tittel. På ein million avspelingar: frå 4,1 s til 2,4 s, og frå 1,6 s til 1,1 s.

### Stability
- **Eit TV-bibliotek vert ikkje lenger lese som eit bibliotek som er tømt.** Lesing av eit bibliotek går i to rundar (alle element, så ein mindre runde berre for medverkande i filmar og seriar), og kontrollen som avgjer om ei lesing er til å stole på når ho skal fjerne det ho ikkje såg, såg på den andre runden. I eit bibliotek med TV eller musikk er dei to tala langt frå kvarandre, så kontrollen avviste kvar lesing og stoppa FinStats, og installasjonen starta på nytt og gjorde det same igjen. Ingenting vart nokon gong feilaktig fjerna, men inga lesing av biblioteket kunne fullførast. Dersom serie- eller musikkbiblioteket ditt ikkje har fått med seg nye titlar, er dette grunnen.
- Ein import kunne feile med «database is locked» når ei anna skriving (ei avspeling som vart oppfriska) kom mellom den første lesinga og den første skrivinga. Begge importverktøya tek skrivelåsen med ein gong dei byrjar, og journalen til databasen held seg ikkje lenger like stor som den største importen som nokon gong er gjord.
- **Ei databasefil som er skadd, avkorta, nokon annan sin eller skriveverna, vert avvist med ein gong**, med namnet på fila og beskjed om at ingenting vart endra, i staden for 30 stille sekund; databasen til eit anna program vert aldri skriven i.
- Ein tryggingskopi eller ein kopi før oppdatering som vert broten av halvvegs, lèt ikkje lenger ei fil liggje att, ei fil som kunne ta plassen til ein god kopi blant dei tre som vert tekne vare på.
- **Å skrive over staddatabasen medan FinStats køyrer, får ikkje lenger FinStats til å krasje**: FinStats les frå ein privat kopi av fila.
- Ein Jellyfin-app som rapporterte ein ny eining-ID kvar gong, kunne få minnet til FinStats til å vekse så lenge programmet køyrde; einingar som ikkje er sett på fem minutt, vert no gløymde.
- Éin person har høgst 30 innlogga økter; den eldste vert logga ut ved neste innlogging.

### Fixed
- **Den same kvelden vert ikkje lenger talt to gonger når eit element har fått nytt namn.** Å kople historikk på nytt til ein tittel som var lagd til på nytt i Jellyfin, kunne gjere ei avspeling importert frå eit sporingsverktøy til ein duplikat av ei som alt var der, fordi kontrollen som ville ha fanga det, køyrde før ID-en flytta seg. No køyrer han igjen overalt der ID-ar vert kopla på nytt. Berre importerte avspelingar vert nokon gong fjerna, aldri ei FinStats registrerte sjølv, og kor mange som forsvann, står i loggen. Den første oppstarten av 2.0 går gjennom heile historikken éin gong for dei som alt er der.
- **Ei økt følgjer no personen slik Jellyfin har han no.** Ein administrator som Jellyfin tok administratorrettane frå, heldt på full tilgang til FinStats resten av den 30 dagar lange økta si, og nokon som var deaktivert eller sletta i Jellyfin, vart verande innlogga. No mistar dei tilgangen så snart FinStats les brukarane i Jellyfin (kvart 15. minutt, eller med ein gong frå Innstillingar), slik det alltid har vore med API-nøklar.
- Innlogging skriv det Jellyfin seier om personen, inn i brukarlista til FinStats, så ein importert tryggingskopi frå eit sporingsverktøy kan ikkje lenger gjere nokon til administrator før den første lesinga av brukarane rettar det.
- Nokon som er sletta i Jellyfin, vert merkt som fjerna sjølv når to lesingar av brukarane fell i same sekund.
- **Nedlastingsfeil viser aldri passordet til ein nedlastingsklient eller API-nøkkelen til ein indekserar**, som Sonarr og Radarr kan sitere i feilmeldingane.
- Avspelingar som vart kopla på nytt til ein tittel med nytt namn under ei lesing av biblioteket, vart ikkje sjekka for å vere sett saman før neste omstart; no vert dei det, med ein gong.

## [1.6.5] - 2026-09-24

### Added
- **Ein full tryggingskopi vert teken automatisk før kvar oppgradering.** Første gongen ein nyare FinStats opnar databasen din, skriv han ein fullstendig kopi av databasen (heile historikken, innstillingane og alt) til `data/pre-update-backups/` *før* noko vert endra. Dersom ei oppgradering nokon gong øydelegg noko, er dataa dine trygge: stopp FinStats, legg kopien tilbake i staden for `finstats.db`, start versjonen du var på, og meld frå om feilen. Ingenting går tapt. Dei nyaste kopiane vert tekne vare på; set `FINSTATS_SKIP_PREUPDATE_BACKUP=1` for å slå det av dersom det er lite diskplass.

## [1.6.4] - 2026-09-24

### Added
- **Fem nye stader å sende varsel til**: Telegram, Slack, Pushover, Pushbullet og e-post, ved sida av mottakarane webhook, Discord, ntfy og Gotify som fanst frå før. Kvar ber berre om det han treng (ein chat-ID, ein brukarnøkkel, ei postkasse), og dei tre som held til hos éi teneste (Telegram, Pushover, Pushbullet), kan berre nåast der, så eit token kan ikkje postast til ein vert som berre liknar. E-post går ut over TLS frå første byte eller med `STARTTLS`, og det finst ikkje noko tredje val.
- **Lisensar**, under **Innstillingar**: GPL-lisensen til FinStats sjølv og kvar tredjepartslisens programmet byggjer på (253 Rust-crates, dei to medfølgjande skrifttypane, kartomrissa og geolokaliseringsdatabasen), kvar tekst i heilskap, slik prosjektet sjølv skreiv han. Vel ein komponent, så opnar lisensen seg i vindauget ved sida av lista; ein crate som har fleire filer, byter mellom dei på same staden. Lista vert laga ut frå dei cratene FinStats faktisk er bygd av, så ein avhengnad kan ikkje leggjast til utan at lisensen hans vert registrert.

## [1.6.3] - 2026-09-24

### Fixed
- Vernet slår no òg inn når Jellyfin listar *ingen* bibliotek eller *ingen* brukarar i det heile (ei øydelagd lesing, aldri ein vanleg dag), noko justeringa i 1.6.2 hadde sleppt gjennom. Å miste nokre bibliotek vert framleis rekna som vanleg; å miste alle på ein gong vert det ikkje.

## [1.6.2] - 2026-09-24

### Fixed
- Vernet for dataintegritet som kom i 1.6.1, var for ivrig: eit bibliotek som verkeleg mista dei fleste elementa sine (til dømes ei handfull klipp der filene forsvann), vart teke for ei øydelagd lesing, så FinStats nekta å oppdatere det og stoppa. No slår vernet berre inn ved ei *tydeleg* øydelagd lesing: eit bibliotek som er stort nok til å bety noko og kjem heilt tomt tilbake, eller eit stort bibliotek som er tømt til nesten ingenting. Vanlege endringar vert tekne i bruk som før, og tilfellet «Jellyfin returnerte ingenting» vert framleis fanga.

## [1.6.1] - 2026-09-24

### Added
- **FinStats stolar ikkje lenger på ei lesing frå Jellyfin som ville slette historikken din.** Dersom eit bibliotek som FinStats har tusenvis av element for, plutseleg kjem tomt eller nesten tomt tilbake (ei Jellyfin-oppgradering som endra API-et, ein feil forkledd som eit tomt resultat), held FinStats på dataa sine, merkjer synkroniseringa som feila (og varslar), og stoppar på ein ryddig måte med ei klar melding i staden for å merkje alt som fjerna. Start på nytt når Jellyfin er seg sjølv igjen, eller set `FINSTATS_ALLOW_LIBRARY_SHRINK=1` dersom du verkeleg tømde eit bibliotek. Det same vernet dekkjer bibliotekslista og brukarlista.

### Security
- Ein `X-Forwarded-Host`-header sendt av ein klient kunne gå god for ein framand `Origin` og få ei skriving på tvers av nettstader forbi vernet for same opphav. Den headeren vert no berre stolt på bak ein klarert proxy (`FINSTATS_TRUST_PROXY`), same regel som oppslaget av klientadressa alt følgjer. Øktinformasjonskapselen med `SameSite=Lax` var, og er framleis, det første forsvaret.
- Ein personleg varselmottakar kunne peikast mot `0.0.0.0` eller `::`, som når denne maskina. Kontrollen som held personlege mottakarar retta mot offentlege adresser, reknar no desse som lokale, saman med loopback og dei private adresseområda.

### Fixed
- **Avspeling**-sida lastar no òg for eit bibliotek som inneheld ei avspeling utan registrert avspelingsmåte (ei rad gjenoppretta frå ein tryggingskopi som er eldre enn kolonna); ei slik avspeling tel berre som direkteavspeling.
- Eit svært langt ord i eit søk eller eit filter gjev ikkje lenger serverfeil: SQLite avviser eit `LIKE`-mønster på over 50 000 teikn, så filterord vert korta ned til ei fornuftig lengd før spørjinga.
- **Innstillingar**-sida rullar ikkje lenger sidelengs på ein telefon når eit kort melder tilbake ei heil setning, til dømes resultatet av ei gjenoppretting.

## [1.6.0] - 2026-09-24

Varsel

### Added
- **Varsel**-kort under Innstillingar: legg til ein mottakar, kryss av for kva han skal få vite om, og trykk Test. Ingen mottakar tyder at det ikkje vert sendt nokon førespurnad til noko.
- Fire slag mottakarar: ein vanleg **webhook** (JSON, dokumentert i `docs/api.md`, valfri `Authorization`-header), **Discord** (ein embed gjennom ei webhook-nettadresse for ein kanal), **ntfy** (ntfy.sh eller din eigen, til eit emne) og **Gotify** (din eigen server, med eit programtoken). Titlar vert sende i meldinga og ikkje i ein header.
- Elleve slag hendingar, kvar med sitt eige kryss: umogleg reise, ein konto sett i eit nytt land, ei bølgje av mislukka innloggingar, ei oppgåve som feila, ein tryggingskopi som ikkje kunne skrivast, noko som slutta å svare (Jellyfin, Sonarr, Radarr eller Seerr) og når det svarar igjen, nye titlar i biblioteket (samla per serie og dag, slik hylla på Oversikt samlar dei), ein førespurnad som vart klar til å sjåast, og ei avspeling som byrjar eller sluttar. Dei to siste er av som standard.
- Ein førespurnad som vart klar til å sjåast, namngjev tittelen, kven som bad om han, og kor lenge dei venta.
- Personlege mottakarar: gje **Varsle**, så kan nokon leggje til sin eigen mottakar, som berre får det personen alt kan sjå: eigne førespurnader og varsel alltid, det som gjeld andre med *Sjå alle*, adresser med *Sjå nettverk*. Ein personleg mottakar må peike på ei offentleg adresse; berre ein administrator kan rette ein inn i nettverket.
- **Ta med IP-adresser og stader**, per mottakar og av som standard: ei melding om umogleg reise namngjev elles berre stader («Oslo, Noreg og London, Storbritannia»).
- **Nyleg sendt**, i same kortet: kva som vart sagt, til kva mottakar, og kva den andre sida svarte når meldinga ikkje kom fram. Ei melding som feila, vert prøvd på nytt fire gonger over om lag ein time, og ein mottakar som avgrensar kor mykje han tek imot, vert venta på.
- **Adressa til FinStats**, ei innstilling for administratorar, er det lenkja i eit varsel vert bygd frå. Tomt felt tyder at meldingane ikkje har noko lenkje.
- Dei planlagde jobbane til Jellyfin på Server-sida: kva kvar av dei gjer, i ord i staden for namnet på koden, og om han er ein av dei tunge. Ein jobb som køyrer, viser ei framdriftslinje, ein prosent og kor mykje som er att; dei andre viser når dei køyrer neste gong, når dei køyrde sist og om det gjekk bra. Jobbar Jellyfin gøymer frå sitt eige kontrollpanel, er med, og ingenting i FinStats kan starte eller stoppe ein jobb.
- Kor mykje som er att av ein jobb, vert målt ut frå farten prosenten faktisk har flytta seg med, mot den nyaste avlesinga som ligg langt nok tilbake til å seie noko, i staden for snittet over heile køyringa, så ein jobb som går raskare eller saktare, vert skildra ut frå tempoet han har no.
- Jobblistene FinStats alt les av andre grunnar (sjekken av biblioteksskanning kvart 5. minutt, serverdetaljane kvart 15.), gjev data til same målinga, så ein jobb som har køyrt i ein time, har som regel eit overslag med ein gong sida opnar seg. Jellyfin vert ikkje spurt om noko ekstra for dette.
- Ein jobb som køyrer utan at noko er målt enno, viser **ETA** og prikkar som går rundt i staden for eit tal, og ein der prosenten ikkje har flytta seg på eit halvt minutt, viser «68 % i 4 min» ved sida av. Ingenting står i staden for eit overslag som ikkje er målt.
- Ei liste over alle versjonar ved sida av endringsloggen: eitt klikk tek deg til kva utgjeving som helst, lista markerer den du les, og på ein smal skjerm vert ho ei rad med versjonsseriar over notata.

### Changed
- Endringsloggen fell ikkje lenger saman. Kvar versjonsserie står open, og versjonslista er måten du flyttar deg på, i staden for éi gruppe open om gongen og eit klikk for å nå kvar av dei andre.
- **Innstillingar → Utgåande tilkoplingar** listar kvar varselmottakar ved sida av Jellyfin og tenestetilkoplingane: kva han er, verten, om han er slått på, og når han sist tok imot ei melding.
- Ingenting ved innsamlinga, historikken eller noko tal på skjermen endrar seg. FinStats *les* framleis berre frå Jellyfin, Sonarr, Radarr og Seerr.
## [1.5.2] - 2026-09-22

### Changed
- Ei lesing av øktlista i Jellyfin og nullstillinga av klokka som bad om ho er no eitt og same kall, så dei to kan ikkje lenger kome ut av takt. Det var forma på feilen med løpske førespurnader som vart funnen medan 1.5.0 vart bygd.
- Å prove det tek no ein brøkdel av eit sekund i staden for sju minutt mot ein ekte server, så det vert sjekka kvar gong noko vert endra. Dei same førespurnadene, den same historikken, dei same tala på skjermen.

## [1.5.1] - 2026-09-22

### Fixed
- Ei omkoding vert registrert éin gong, ikkje éin gong i sekundet. Når ei avspeling byrjar med omkoding og går tilbake til direkteavspeling, lèt Jellyfin omkodingsdetaljane liggje att på økta; kvar avlesing vart samanlikna med ein verdi FinStats nettopp hadde skrive over, så kvar avlesing såg ut som ei endring, og Aktivitet-sida fylte seg med «Omkoding»-linjer med «Direkteavspeling: …» under.
- Linjer som alt er skrivne, vert rydda opp ved oppstart: der fleire etter kvarandre seier det same, vert berre den første behalden. Ei avspeling som trong omkoding, tel framleis som ei omkoding.

## [1.5.0] - 2026-09-22

Øktsporing over WebSocket

### Changed
- FinStats held éi tilkopling open til Jellyfin frå det startar, og det er slik det får vite om ei avspeling. 1.4.0 kom med dette som ei innstilling, av som standard, for at det skulle prove seg mot ekte serverar; no er det alltid på. Dei same avspelingane, pausane og hoppa, rekna ut av den same koden.
- Ingenting vert spela av: FinStats spør ikkje Jellyfin om noko i det heile. Ein kveld då ingen ser på, kostar ingen førespurnader.
- Noko vert spela av: øktlista vert lesen éin gong i sekundet. Det er det som gjev ein pause, ei spoling eller eit sporbyte skarpleiken, og det er det som avsluttar ei avspeling der klienten forsvann utan å seie farvel.
- Alt som er lasta, har stått på pause i tre avlesingar på rad: spørjinga stoppar, og tilkoplinga tek over, der ein film på pause før kosta det same spørsmålet kvart sekund, rundt 16 MB i timen. Held nokon fram, eller startar nokon andre noko, vert det fanga opp innan om lag eitt sekund, og tid på pause tel framleis aldri som visingstid.
- Kvar utgåande lesing ber om komprimerte svar og pakkar dei ut (Jellyfin, Sonarr, Radarr og Seerr komprimerer alle når ein ber om det), og det kuttar meir enn halvparten av storleiken på dei to største lesingane: øktlista under ei avspeling, og lesinga av biblioteket. Geolokaliseringsdatabasen vert framleis lasta ned slik han er lagra.
- Jellyfin-kortet i Innstillingar seier kva for ein av dei to halvdelane som køyrer, og direkteprikken følgjer om tilkoplinga faktisk fører data, ikkje kva halvdel du er i.
- Berre éin av dei to køyrer om gongen: medan FinStats spør, ber det Jellyfin slutte å sende, og ber Jellyfin byrje igjen same augneblinken den siste avspelinga sluttar. Tilkoplinga står open heile tida.
- Ein Jellyfin-server som svarar på abonnementet med ingenting, vert trudd så lenge han svarar på livsteikn: FinStats seier frå om det éin gong, les øktlista sjølv i mellomtida og held på abonnementet til den første sendinga avgjer saka. «Kan ikkje sende» og «heldt på å starte på nytt då vi spurde» ser like ut, så stilla vert aldri teken som eit svar.
- Reserveløysinga er urørt: ei tilkopling som vert lukka, vert stille eller viser seg ikkje å snakke dette, fell tilbake til å spørje på tidtakar alt ved neste runde og held fram med å kople til på nytt, med intervalla **Medan nokon ser på, sjekk kvart** og **Medan ingenting vert spela av, sjekk kvart**.

### Added
- Kva innsamlaren held på med, på `GET /api/status`: kva for ein av dei fire tilstandane han er i (`idle_socket`, `playing_poll`, `paused_socket`, `fallback`), om tilkoplinga er open og abonnerer, kor ofte han spør, kor mange øktlesingar som gjekk ut det siste minuttet, og når dette sist endra seg. Svaret kjem frå minnet (ingen førespurnad, inga databasespørjing), og det inneheld ingen namn, titlar eller tal.
- Desse felta vert kontrollerte mot kvarandre ved kvar runde, og ei usemje som varer, vert skriven til loggen.
- Éi sperre under kvar øktlesing i prosessen: høgst 2 i sekundet og 70 i minuttet; alt over det vert utsett, talt og logga høgst éin gong i minuttet. Vanleg sjåing held seg godt under.

### Fixed
- Direkteoppdateringane erstattar no verkeleg spørjinga. Med innstillinga slått på spurde 1.4.0 framleis etter øktlista kvart femte sekund og opna tilkoplinga på nytt kvart tjuande til førtiande sekund, fordi tilkoplinga vart vurdert etter kor nyleg ei liste hadde kome, og Jellyfin sender ei når noko *endrar seg* og ingenting i mellomtida. Om tilkoplinga lever, vert no avgjort av om Jellyfin svarar i det heile, sjekka med eit livsteikn om lag to gonger i minuttet.
- Ei øktliste som kom medan FinStats var mellom to rundar, vart kasta, ut frå at ei ny låg halvanna sekund bak. Ho vert no teken vare på og brukt.
- **Sist sjekka** under Innstillingar og kortet Utgåande tilkoplingar vert ikkje lenger utdaterte medan tilkoplinga er stille: det er svaret frå Jellyfin på eit livsteikn dei viser.

### Removed
- Innstillinga **La Jellyfin sende det som vert spela av** under Innstillingar → Innsamling, og `live_socket` frå innstillings-API-et.
- `socket_enabled` frå innsamlarstatusen: det finst ikkje lenger noko «slått av».
- `last_reconcile_at` frå innsamlarstatusen, erstatta av `socket_live`.

## [1.4.1] - 2026-09-22

### Changed
- Seerr vert berre lesen i heilskap når noko har endra seg: kvart femte minutt ber FinStats om éi rad, den sist endra førespurnaden, og runden sluttar der dersom FinStats alt kjenner til han. Det er om lag ein kilobyte mot rundt seksti for ei side på femti. Førespurnader som framleis er på veg, vert kontrollerte kvart kvarter, alt vert lista opp éin gong i døgnet, og ein ny førespurnad dukkar framleis opp innan fem minutt.
- Køane i Sonarr og Radarr vert lesne kvart femte sekund medan ei side viser dei, kvart minutt medan noko ligg i køen og kvart femte minutt medan han er tom, i staden for kvart minutt heile døgnet. Når sida vert opna, ei teneste vert kopla til eller det kjem ein ny førespurnad i Seerr, vert dei oppdaterte med ein gong.

## [1.4.0] - 2026-09-21

Valfrie WebSocket-økter, og alt FinStats kan nå

### Added
- **La Jellyfin sende det som vert spela av** under Innstillingar → Innsamling, av til du slår det på: éi tilkopling ber øktlista, og om lag 17 000 førespurnader i døgnet vert til nesten ingen. Avspelingane, pausane, hoppa og gruppene vert rekna ut av nøyaktig den same koden som før. Medan ho er aktiv, sender FinStats framleis éin vanleg førespurnad i minuttet *medan noko vert spela av*, og ingen når ingenting vert det. Ei tilkopling som vert lukka, er stille i femten sekund eller viser seg ikkje å snakke dette, fell tilbake til tidtakaren ved neste runde og held fram med å kople til på nytt.
- **Utgåande tilkoplingar**, eit kort i Innstillingar for Jellyfin-administratorar: kvart mål FinStats kan nå (Jellyfin-serveren din, «kva er IP-en min»-tenesta, databasen til DB-IP, og kvar Sonarr, Radarr eller Seerr), med kva det er til, om det er slått på, og når det sist svarte. Kortet vert bygt av det FinStats alt veit, og viser berre adresser: aldri ein nøkkel, aldri ein basesti.
- **Slå opp no** under Innstillingar → Heimenettverk, for dagen den offentlege adressa di endrar seg.

### Changed
- Oppslaget «kva er IP-en min» køyrer éin gong, første gongen FinStats treng ei adresse, i staden for kvart kvarter.
- Jellyfin-kortet i Innstillingar seier korleis innsamlaren får vite ting, og, når tilkoplinga er slått på men ikkje fører data, kvifor ikkje.

## [1.3.0] - 2026-09-20

Sonarr, Radarr og Seerr, på éi side

### Added
- **Tilkoplingar** under Innstillingar, for Jellyfin-administratorar: **Sonarr**, **Radarr** og **Seerr** (eller Jellyseerr/Overseerr), fleire av kvart slag, kvar av dei testa før ho vert lagra. Nedlastingsklienten din treng ikkje noko eige oppsett, fordi Sonarr og Radarr alt snakkar med han, og FinStats les det dei veit.
- **Førespurnader**: kven som bad om kva, kor lang tid det tok før det kom, og om dei nokon gong såg det. Ruter for titlar som ventar og den vanlege ventetida, eit diagram over korleis ho har endra seg månad for månad, og lister over det som kom for fleire veker sidan og aldri vart spela av, og over kven som spør om mest.
- **Kjem**: nye episodar og filmutgjevingar frå Sonarr og Radarr som ei oversikt dag for dag, merkte med om *du* ser den serien og, for dei som får sjå alle, kven andre som gjer det. Nokon vert rekna for å sjå ein serie når dei har spela av ein episode av han dei siste fire månadene. Ei «Kjem snart»-rad på Oversikt, éi per profil for seriane den personen ser, og kva som er det neste for ein tittel, på sida til tittelen.
- **Nedlastingar**: den direkte køen i kvar Sonarr og Radarr (kva det er, kor langt det har kome, kor fort det går, kva som gjekk gale ved import), med personen som bad om det ved sida. Ei sesongpakke er éi linje same kor mange episodar ho held, og det som treng merksemd, ligg øvst. Under det: kva som kom inn over ei veke, ein månad eller eit år etter indekserar, kvalitet og nedlastingsklient, og kva nedlastingar som feila. Oversikt viser den same lista, forkorta.
- **Sjå nedlastingar**, eit nytt løyve. Utan det ser folk framleis kor langt deira *eigen* førespurnad har kome og kor lang tid som står att, og ingenting anna: ingen utgjevingsnamn, ingen fart, ingen andre nedlastingar. Førespurnadene til andre krev *Sjå alle*.

### Changed
- FinStats sender `GET` til desse tenestene og ingenting anna: det finst ingen kode i FinStats som kan godkjenne ein førespurnad, starte eit søk eller leggje til, setje på pause eller fjerne ei nedlasting.
- API-nøklane deira vert lagra i den eigne databasen til FinStats, vert aldri sende tilbake til nettlesaren, aldri skrivne til ein logg og er aldri med i ein tryggingskopi. Ingen omdirigering vert følgd, så ein nøkkel kan ikkje hamne ein stad du ikkje skreiv inn, og sertifikat vert kontrollerte med mindre du slår det av for éi tilkopling.

## [1.2.2] - 2026-09-20

### Added
- Lyd- og undertekstspråk på kvar film og episode («Lyd: japansk · engelsk»).
- Kor langt ein dubbing går: ein serie eller ein sesong seier kor mange av episodane som har kvart språk («Engelsk: 13 av 26 episodar»), og episodelista har ei **Lyd**-kolonne.
- **Lydspråk** og **Undertekstspråk** på bibliotekssidene: kor mange filer som kan spelast av på kvart språk.
- FinStats les biblioteket på nytt éin gong etter denne oppdateringa for å hente språka. Det seier aldri «dubba», fordi Jellyfin ikkje oppgjev originalspråket til ein tittel, og eit spor utan språkmerke vert lista som «Ukjend».

## [1.2.1] - 2026-09-20

### Changed
- Bileta i README-en er tekne på nytt i denne versjonen, med eitt nytt av **Tryggleik**-sida. Ingenting i appen endrar seg.

## [1.2.0] - 2026-09-20

Kvar folk ser frå, på eit kart

### Added
- **Tryggleik-sida**, for dei som får sjå nettverksdetaljar og aktiviteten til alle: eit verdskart med ein prikk for kvar stad brukarane dine ser frå, med storleik etter kor mykje som skjer der. Heime vert vist i grønt, stader borte i lilla, pågåande straumar pulserer, og mislukka innloggingar utanfrå vert raude. Dra for å flytte, zoom med knappane, tastane eller Ctrl + hjul, og prikkar som overlappar, vert slegne saman til du zoomar inn. Under kartet: kvar stad med personane, avspelingane, visingstida og innloggingane, og landa etter tal på avspelingar.
- **Umogleg reise**: eit varsel når éin konto dukkar opp på to stader inga flyrute bind saman, eller på to stader langt frå kvarandre samstundes, med begge observasjonane, avstanden, tida mellom dei og farten det ville ha kravd. **Vis på kart** teiknar reisa.
- **Nytt land**: første gongen nokon spelar av eller loggar inn frå eit land FinStats ikkje har sett dei i før.
- Løys eit varsel med eit notat, eller demp eit par stader for den personen, for ein VPN eller ein telefon der operatøren held til i hovudstaden. Varsel som vert funne i gammal historikk, etter ein import eller i ein fersk database, vert arkiverte som løyste i staden for å fløyme over lista. Varsla og kva du avgjorde om dei, er med i tryggingskopiane.
- Stadene kjem frå ein bydatabase som vert lesen lokalt: ingen adresse vert nokon gong send nokon stad, og kartet vert teikna frå omriss som følgjer med FinStats i staden for frå ei karteneste. Last ned den gratis databasen til DB-IP frå Tryggleik-sida eller **Innstillingar → Tryggleik**, la FinStats fornye han kvar månad (av til du slår det på), eller legg di eiga `.mmdb`-fil (DB-IP, MaxMind GeoLite2-City) i `data/geoip/`. `FINSTATS_GEOIP_DB` peikar på ei fil ein annan stad.
- **Innstillingar → Tryggleik**: databasen som er i bruk, den månadlege oppdateringa, og kor fort (900 km/h) og kor langt frå kvarandre (500 km) to observasjonar må vere for å telje som umogleg reise. Bydatabasar bommar ofte med nokre hundre kilometer, og det er det avstanden er til for.

## [1.1.3] - 2026-09-20

### Added
- Ctrl+Space søkjer òg i dei medverkande i alt i biblioteket ditt, under **Medverkande**, med bilete, om dei spelar eller regisserer, og kor mange av titlane dine dei er med i; vel du ein, opnar sida deira. Like tilgjevande som tittelsøket (ord i kva rekkjefølgje som helst, aksentar vert ignorerte, ein feiltast er lov), og når fleire treffer like godt, kjem den som er med i flest av titlane dine, først.

## [1.1.2] - 2026-09-20

### Added
- **Nyleg lagt til** på Oversikt, over Aktivitet-diagrammet: dei 30 siste tilskota som ei rad med plakatar du kan rulle sidelengs, med piler på ei datamaskin. Kvar plakat seier når tittelen kom og kva han er (ein film, eit album eller «Sesong 4 · 3 episodar»), og ein enkelt ny episode seier kva for ein, med plakaten til sesongen når sesongen har ein.
- **Shift + musehjul** og piltastane flyttar éin plakat, Page Up og Page Down ein skjerm, Home og End går til kvar sin ende, og sveiping fungerer. Det vanlege hjulet rullar framleis sida.
- Nye episodar av same serie som kjem same dag, deler éi oppføring, og ein heil serie som vert lagd til på éin gong, er éi oppføring («3 sesongar · 60 episodar»), så éin stor import skubbar ikkje alt anna ut av lista.
- Rada bryr seg ikkje om tidsrommet eller personen på Oversikt, sidan ho handlar om biblioteket og ikkje om avspelingar, og ho gøymer seg medan biblioteket er tomt.

## [1.1.1] - 2026-09-20

### Added
- Sider opnar med ein gong: ein augneblink etter at FinStats er lasta, hentar det i det stille det du truleg opnar neste gong (Oversikt, profilen og tidslinja di, Aktivitet, Brukarar, Bibliotek og kvart bibliotek, Avspeling, serversidene, tilbakeblikket og dei mest aktive personane), så ei side syner seg straks når du opnar ho, og ho vert likevel oppdatert i bakgrunnen.
- Filmar, seriar og medverkande vert berre henta når du viser at du vil opne dei, sidan det finst tusenvis av dei: når peikaren kviler på ei lenkje, når du rører ved ho eller når du når ho med tastaturet, vert den eine sida henta.
- Det ventar til sida du opna, er ferdig lasta, tek éi side om gongen medan nettlesaren er ledig, tek pause i ei fane i bakgrunnen, gjer ingenting på ei datasparande eller svært treg tilkopling, og vert gløymt når du loggar ut.
- Ei lita **Repo**-lenkje med GitHub-merket i statuslinja, ved sida av versjonen.

### Fixed
- **Tidslinje** gløymde kva bibliotek som var slått av når sida vart lasta på nytt eller opna frå eit bokmerke, sjølv om valet stod i adressa.

## [1.1.0] - 2026-09-20

Ei tidslinje over alt du har sett

### Added
- **Tidslinje**, ved sida av **Oversikt** på kvar profil: sjåinga di som éin sti frå i dag og tilbake til den første avspelinga FinStats kjenner til, der ein kveld med episodar er falda saman til eitt stopp. Kvart stopp er ein plakat med kva som vart sett og når («Sesong 2 · Episode 3–6», ein film og om han tok meir enn éi økt, eller eit album og kor mange spor), og opnar tittelen.
- Stien følgjer breidda på vindauget: tre stopp i rada på ein brei skjerm, to på ein smalare, kvar andre rad går baklengs med ein boge som bind ho til den neste, ned til éi rett linje på ein telefon. Det første stoppet i kvar månad har namnet på månaden.
- Eldre stopp vert lasta av seg sjølv når du rullar, heilt tilbake til der historikken byrjar.
- **Bibliotek**-brytarane over stien utelèt det du ikkje vil sjå, til dømes musikk, og valet er ein del av adressa.
- Noko som framleis vert spela av, seier **Spelast no**.
- Du ser di eiga tidslinje; tidslinjene til andre krev løyvet «Sjå alle».

## [1.0.5] - 2026-09-20

### Added
- Vern mot nedgradering: databasen hugsar den nyaste FinStats-versjonen som har opna han, og ein eldre versjon nektar å starte på han i staden for å arbeide i det stille med data han ikkje fullt ut forstår. Meldinga seier kva du skal gjere. Versjonar til og med 1.0.4 kom ut før denne kontrollen fanst.

## [1.0.4] - 2026-09-20

### Fixed
- I detaljane til ei avspeling hadde kopiknappen ved ein lang **Einings-ID** falle ned på ei eiga linje.
- På Avspeling-sida vart den siste kolonnen i **Kva klientar kodar om** kutta («Omko…»); overskrifter på to ord bryt no over to linjer.

### Changed
- Skjermbileta i README-en er oppdaterte igjen; dei viste framleis versjon 0.5.0.

## [1.0.3] - 2026-09-20

### Added
- Ei episodeside viser biletet til den episoden, det Jellyfin viser i episodelista si, i staden for plakaten til serien. Ein episode Jellyfin ikkje har noko bilete av, held på plakaten.

## [1.0.2] - 2026-09-20

### Fixed
- I **Endringslogg** hadde gruppene v0.9 og v0.10 inga overskrift ved sida av versjonen. Overskrifta til ei gruppe er no alltid den første setninga om kva den første utgjevinga i gruppa handla om, så ei lang innleiing vert ikkje kutta midt i ei setning.

## [1.0.1] - 2026-09-20

### Fixed
- Det publiserte Docker-biletet ville ikkje starte på ei ny maskin: «unable to open database file: /data/finstats.db», etter å ha hengt i eit halvt minutt. Docker lagar ei manglande `data`-mappe som root, og FinStats køyrer som ein vanleg brukar. Containeren gjer no mappa til si eiga og byter så til den brukaren, så `docker run` fungerer på første forsøk. For filer som nokon annan enn brukar 1000 eig, set `PUID` og `PGID`; `--user` fungerer som før.
- Ei datamappe som verkeleg ikkje kan skrivast til, vert rapportert med ein gong, med kommandoen som rettar det, i staden for 30 sekund seinare som ein databasefeil.

## [1.0.0] - 2026-09-20

Klar for alle

### Added
- Eit ferdig Docker-bilete, no på `ghcr.io/finstats/finstats`, for 64-bits Intel/AMD- og ARM-maskiner (ein Raspberry Pi 4 eller 5 fungerer): `:latest` er den nyaste utgjevinga, ein versjonstagg står der han står, og `:edge` er utviklingsversjonen.
- Kvar utgjeving kjem på utgjevingssida til prosjektet på GitHub med denne endringsloggen.

### Changed
- Versjonsnummera tyder no det dei seier: 1.x-oppdateringar øydelegg ikkje dataa dine, tryggingskopiane eller innstillingane.
- Docker Compose-fila og all rettleiing brukar det publiserte biletet. Eit `finstats:latest` du har bygt sjølv, treng berre at namnet på biletet vert endra; `data`-mappa vert med urørt.

## [0.10.2] - 2026-09-20

### Fixed
- På ein telefon kunne Innstillingar og sida til ein serie rulle sidelengs: ein etikett meint berre for skjermlesarar slapp ut av tabellen sin, og episodelistene og rada med medverkande gjorde det same. Dei rullar no inne i seg sjølve.
- Tilbakeblikket rullar ikkje lenger sidelengs når bakgrunnsordet i eit kapittel er breiare enn sida.
- Ei lang «ser på …»-linje under **Spelast no** sluttar no med ellipse i staden for å verte kutta midt i ein bokstav.
- Plakaten framfor kvar serie på ein profil var ei ekstra, namnlaus lenkje for skjermlesarar; han er gøymd for dei, og tittelen ved sida er lenkja.

## [0.10.1] - 2026-09-20

### Fixed
- Nedlasting av ein tryggingskopi feila i nettlesaren («Kjeldefila kunne ikkje lesast» i Firefox) utan noko på sida som sa kvifor: FinStats komprimerte ein tryggingskopi, som alt er komprimert, ein gong til, og den dobbeltpakka overføringa braut av før slutten. Tryggingskopiar vert sende som dei er, med storleiken oppgjeven på førehand, så nettlesaren kan vise reell framdrift.

## [0.10.0] - 2026-09-20

Tryggingskopiar, og flytting til ein ny installasjon

### Added
- Ein tryggingskopi kvar veke, og dei fem nyaste vert tekne vare på: kvar avspeling med tidslinja for pausar og hopp, sett-merke, løyve, heimeadresser, serverloggen og innstillingane dine, i éi lita fil, nokre hundre KB for tusenvis av avspelingar. **Innstillingar → Tryggingskopiar** listar dei med ein **Last ned**-knapp kvar, lagar ein når du vil, og styrer kor ofte og kor mange. Dei ligg i `backups`-mappa i datamappa di. Ein tryggingskopi inneheld ingen API-nøkkel til Jellyfin og inga innloggingsøkt, men han er ein komplett sjåhistorikk med IP-adresser, så hald han ein privat stad.
- Gjenoppretting slår saman i staden for å skrive over: avspelingar som alt finst, vert hoppa over, så det er trygt å gjere det to gonger eller inn i ein installasjon som har køyrt ei stund. Fjern haken for «Gjenopprett òg innstillingar og løyve» for berre historikken. Tryggingskopiar fungerer på tvers av versjonar begge vegar. Frå ein terminal: `finstats backup` og `finstats restore <file>`.

### Changed
- Innsamlinga sjekkar kvart sekund medan nokon ser på og kvart 5. sekund medan ingen gjer det, der ho før sjekka kvart 5. sekund heile tida, så pausar, hopp og sporbyte vert registrerte på sekundet. Begge ligg under **Innstillingar → Innsamling**; eit gammalt intervall du har endra, må setjast på nytt.

### Fixed
- Ei IP-adresse i detaljane til ei avspeling vart broten over to linjer midt i adressa. Merket Lokal/Utanfrå flyttar seg under når det ikkje er plass.

## [0.9.1] - 2026-09-20

### Fixed
- Folk heime vart viste som utanfrå. Ei eining på ditt eige nettverk som når Jellyfin gjennom det offentlege namnet, til dømes via ein omvend proxy eller eit domene, kjem med den offentlege IP-adressa til husstanden, og berre private adresser vart rekna som lokale. FinStats lærer no den offentlege adressa og reknar dei avspelingane som lokale for heile historikken, og det hugsar tidlegare adresser.

### Added
- **Innstillingar → Heimenettverk**: adressene FinStats reknar som heime, der du kan leggje til dine eigne (ei tidlegare adresse, ein heim nummer to, ein VPN-utgang) eller slå av oppslaget. Oppslaget spør ei enkel «kva er IP-en min»-teneste kvart 15. minutt og prøver fleire (Amazon, Cloudflare og andre) fordi DNS med reklameblokkering, som Pi-hole, ofte blokkerer dei, og éi av dei treng ingen DNS i det heile. Førespurnaden inneheld ingenting om deg eller serveren din, og med brytaren av snakkar FinStats ikkje med noko anna enn Jellyfin. `FINSTATS_PUBLIC_IP_URL` vel ei teneste du sjølv har.

## [0.9.0] - 2026-09-20

Kvar tabell, i den rekkjefølgja du vil

### Added
- Alle tabellar kan sorterast: klikk på ei overskrift for å sortere etter ho, ein gong til for å snu, og ein tredje gong for den opphavlege rekkjefølgja. Innhaldet i cellene vert forstått, så «3 d 2 t» vert sortert som tid og «1,4 GB» som storleik, og tomme celler kjem sist. Det gjeld brukarlista, sjåarane, episodane, einingane, programtillegga, dei planlagde oppgåvene, tabellvisinga av kvart diagram, løyvematrisa og stolpelistene (kodekar, oppløysingar, klientar og resten), som får ei eiga, slank overskriftsrad.
- Aktivitet og serverloggen vert sorterte på tvers av alle sidene sine, ikkje berre radene på skjermen, og rekkjefølgja vert halden i adressa, så ho overlever ei ny lasting eller ei delt lenkje.
- Tabellar med ti rader eller fleire får eit lite filterfelt over seg.

## [0.8.2] - 2026-09-20

### Changed
- Årsveljaren i tilbakeblikket er den same brytaren som alle andre tidsromveljarar i FinStats (7 d · 30 d · 90 d …).

## [0.8.1] - 2026-09-19

### Added
- Sider for skodespelarar og regissørar, frå «Personane du såg mest av» i tilbakeblikket eller den nye **Medverkande**-rada på kvar film, serie eller episode: alt dei er med i på serveren din, kor mykje som er sett og av kven, og kva som framleis ventar. Sidene følgjer tidsrommet du vel, dei som berre får sjå sin eigen statistikk, ser berre si eiga sjåing der, og Esc tek deg tilbake.

## [0.8.0] - 2026-09-19

Tilbakeblikket har fått ny drakt, og meir å fortelje

### Added
- **Personane du såg mest av**: skodespelarane du brukte mest tid med, og regissørane bak det du såg, som ei rad med portrett. Medverkande vert lesne saman med biblioteket og dukkar opp etter den første lesinga av biblioteket etter denne oppdateringa.
- **Året ditt i dagar**: kvar dag i året som ei rute, lysare jo meir du såg, med den lengste rekkja di og den største dagen din over. Hald peikaren over ein dag, eller gå gjennom kalenderen med piltastane, for å sjå kva han inneheldt.
- **I tal**: avspelingar, visingstid, ulike titlar, dagar med sjåing, lengste rekkje og kor mykje som var **gjensyn**, pluss korleis året fordeler seg mellom episodar, filmar og musikk.
- Sjangerkapitlet seier kor mange sjangrar du var innom, og kor stor del den største tok.

### Changed
- Ei ny utforming: det opnar med året ditt «spela av på nytt», overskrifta ved sida av plakatane som fylte det, over ei bølgjeform av året med éin stolpe per veke og dei høgaste lyste opp, og du vel året rett over. Kvart kapittel har ei overskrift med nøkkelordet i fiolett, ei setning som ber tala, og ordet til kapitlet stort og svakt bak. Det sluttar slik filmar gjer, med rulletekstane: i hovudrollene, regi, vist på, speletid.
- Timar, vekedagar og månader deler eitt **Aktivitetsmønster**-kapittel med ein brytar mellom dei, og overskrifta følgjer det du ser på («Fredagen tok krona»).

### Fixed
- **Spelast no** på Oversikt stod tom når ingen såg på, så overskrifta såg ut til å høyre til filtera under ho. Boksen «Ingenting vert spela av akkurat no» er tilbake.

## [0.7.10] - 2026-09-19

### Added
- Bibliotekbilete: kvart bibliotekkort, og sida til biblioteket, viser biletet Jellyfin brukar for det, gjennom biletproxyen til FinStats slik som plakatane. Eit bibliotek utan bilete held på ikonet sitt.

## [0.7.9] - 2026-09-19

### Changed
- Sjangrar-kortet opnar som **liste** igjen, med radaren som det andre valet på brytaren. Eit val du alt har gjort, vert halde på.

## [0.7.8] - 2026-09-19

### Added
- Ei radarvising for Sjangrar-kortet, på Oversikt og på profilar: éi eike per sjanger, ein brytar mellom **Radar** og **Liste** i hjørnet av kortet, og valet vert hugsa. Når du held peikaren over eit punkt eller brukar piltastane, ser du visingstida og delen til den sjangeren.

## [0.7.7] - 2026-09-19

### Fixed
- Filmar som høyrer til ei samling, vart viste som «Ikkje lenger i biblioteket». Med *grupper filmar i samlingar* slått på gjev Jellyfin ut samlinga i staden for filmane inni, så FinStats såg dei aldri; på éin server gøymde det 556 av 1 238 element i Movies-biblioteket. FinStats ber no om filmane sjølve, og lesinga denne oppdateringa set i gang, hentar dei tilbake og koplar avspelingane deira til på nytt.

## [0.7.6] - 2026-09-19

### Changed
- På profilar har **Seriar**-kortet flytta ned til rett over Sjangrar, så ei lang liste med seriar ikkje lenger skubbar visingstida, aktivitetsdiagrammet og topptitlane ut av syne. Dagsrekkjene står att øvst, og ein serie du har opna, held seg open når du endrar tidsrommet.

## [0.7.5] - 2026-09-19

### Changed
- Søket treffer ord for ord, i kva rekkjefølgje som helst, med kva som helst imellom, i staden for å krevje den nøyaktige frasen, så «alya hides» finn *Alya Sometimes Hides Her Feelings in Russian*. Store og små bokstavar, aksentar og teiknsetjing spelar inga rolle, ein innleiande «The» vert ignorert, og ein feiltast vert tilgjeven («mentalsit» finn *The Mentalist*); ord på tre bokstavar eller færre må framleis skrivast rett. Betre treff kjem først, og musikk vert òg funnen etter artist.
- Søkjefelta på Aktivitet og i serverloggen treffer ord for ord på tvers av alle kolonnane: «alya opera» finn avspelingar av den serien i Opera.

## [0.7.4] - 2026-09-19

### Changed
- Endringsloggen er gruppert etter serie: kvar `0.x`-serie er éi gruppe som kan faldast saman og held utgjevingane sine, med kva serien handla om, kor mange det er og når. Berre den nyaste startar open, og ei lukka gruppe viser likevel kva for ein du køyrer.

## [0.7.3] - 2026-09-19

### Fixed
- Klokka under Spelast no nølte med nokre sekunds mellomrom: oppdateringa kvart 5. sekund teikna ho òg på nytt, så kvar gong serveren var ein brøkdel føre, flytte talet seg for tidleg og stod så stille i eit sekund. No flyttar berre tikket éin gong i sekundet ho, med nøyaktig eitt sekund (målt 999 til 1 001 ms frå kvarandre), og oppdateringa rettar framleis opp etter ein pause, eit hopp eller verkeleg avdrift, og landar på tikket.

## [0.7.2] - 2026-09-19

### Added
- Gruppesjåing, direkte: kvart Spelast no-kort seier «Med maria» medan folk ser det same saman, der ei gruppe før berre vart kjend att når avspelingane var ferdige. Direkte tel folk som saman når dei er på same tittel og anten starta innanfor gruppevindauget eller er på nesten same posisjon, noko som òg fungerer om FinStats vart starta på nytt midt i ein straum.

## [0.7.1] - 2026-09-19

### Changed
- Spelast no tel kvart sekund: klokka tikkar og framdriftslinja glir imellom, i staden for å hoppe kvar gong serveren vert spurd. Serveren vert framleis spurd kvart 5. sekund og er framleis fasiten: visinga synkroniserer på nytt ved ein pause, eit hopp eller når serveren er føre, men ikkje for ein posisjon som berre er nokre sekund gammal, sidan klientar berre melder frå om lag kvart tiande sekund. Straumar på pause står stille.

## [0.7.0] - 2026-09-19

Kven ser saman

### Added
- Gruppesjåing: når ulike personar startar same tittel samstundes, reknar FinStats det som sjåing saman. Eit **Sett saman**-kort på Oversikt viser gruppene, titlane dei deler og timane dei har hatt saman, kvar profil viser kven den personen ser mest saman med, delte avspelingar har eit lite personmerke i Aktivitet, og detaljane til ei avspeling seier kven andre som såg med. Det fungerer på heile historikken din, importerte avspelingar òg.
- Ei innstilling for kor tett startane må liggje, 60 sekund som standard: om lag ein tredjedel av ekte gruppeøkter startar 6 til 60 sekund frå kvarandre. Folk må òg halde fram med å sjå ved sida av kvarandre i eit par minutt, så to personar som tilfeldigvis opnar same episode, er ikkje ei gruppe.

### Changed
- Søket opnar med **Ctrl + Space** i staden for Ctrl/⌘ + K.

## [0.6.1] - 2026-09-19

### Added
- Administratorar kan opne tilbakeblikket til ein annan person, øvst på Tilbakeblikk-sida eller med **Opne tilbakeblikk** på profilen til personen. Det vert då skrive om dei med namn i staden for «du».

### Changed
- Alle andre ser framleis berre sitt eige tilbakeblikk, same kva løyve dei har, og det finst framleis ikkje noko tilbakeblikk for heile serveren.

## [0.6.0] - 2026-09-19

Profilen din, og kvar du er i kvar serie

### Added
- Framdrift i seriar på profilar: kvar serie du har vore innom, som ein stolpe med eitt segment per episode (sett, påbyrja eller ikkje enno), som kan opnast sesong for sesong, sortert etter kor nær du er å verte ferdig, med eigne visingar for ferdige seriar og for alt.
- **Merk som sett** på din eigen profil, per episode, sesong eller serie, for det du såg medan ingenting registrerte. Merka for «spela av» i Jellyfin vert òg brukte. Merka vert verande i FinStats og endrar aldri noko i Jellyfin; ei registrert avspeling kan ikkje avmerkast, merka du har lagt inn sjølv, kan det.
- Dagsrekkjer på profilar: den lengste rekkja di av dagar på rad med ei avspeling, den noverande rekkja di, og kor mange dagar du har sett noko.
- **Profilen min** i sidepanelet.
- Aktivitet viser kvar avspelinga stoppa (til dømes `44:15 / 45:00`) under framdriftslinja, og dato og klokkeslett under «for 3 t sidan». Avspelingar importerte frå Jellystat har ingen stoppposisjon, fordi Jellystat aldri registrerte nokon.

### Changed
- Episodar som manglar eller ikkje er sende enno, vert ikkje lenger rekna som ein del av biblioteket ditt. Jellyfin listar dei som plasshaldarar utan fil, og dei talde før med i talet på episodar.

## [0.5.1] - 2026-09-19

### Added
- Ein logo: ein avspelingsknapp skoren i tre diagramstolpar, i sidepanelet og på innloggingsskjermane, som ein skarp SVG med ein `favicon.ico` i fleire storleikar for nettlesarar og bokmerkelinjer som vil ha ein, pluss eit ikon for heimeskjermen på telefonar og nettbrett.

## [0.5.0] - 2026-09-19

Du avgjer kven som ser kva

### Added
- Løyve under Innstillingar → Tilgang, for alle på ein gong eller person for person: **sjå aktiviteten til alle** (statistikken og historikken til andre, Brukarar-sida, kvar pågåande straum), **sjå nettverksdetaljar** (IP-adresser, einings-ID-ar, lokal eller utanfrå), **sjå serveren** (Server-sida, serverloggen, mislukka innloggingar, filstiar) og **administrere FinStats** (innstillingar, oppgåver, import, sletting av avspelingar).
- Innlogging per person, for å sleppe inn nokre få utan å opne FinStats for kvar Jellyfin-brukar.
- Endringar gjeld med ein gong, òg når tilgang vert teken bort, og alt vert handheva av serveren i staden for ved å gøyme knappar.

### Changed
- Jellyfin-administratorar har framleis alt og er dei einaste som kan endre løyve; ein som får administrere FinStats, kan ikkje gje noko, verken til seg sjølv eller andre.
- Tilbakeblikket for året er framleis personleg, same kva som er gjeve.
- Ingenting endrar seg for eksisterande installasjonar før du gjev noko.

## [0.4.5] - 2026-09-19

### Changed
- FinStats er lisensiert under **GNU General Public License v3.0**. Ingen tidlegare versjon vart nokon gong publisert, så ingenting kom ut under andre vilkår. Lisensane til dei medfølgjande skrifttypane ligg ved sida av dei.

## [0.4.4] - 2026-09-19

### Added
- Esc går eitt steg tilbake: på eit opna bibliotek, ein brukar eller ein tittel tek Esc deg tilbake dit du kom frå, med lista nøyaktig slik du forlét ho. Det som er ope, vert lukka først (avspelingsdetaljar, søk, ein nedtrekksmeny, mobilmenyen), og Esc gjer ingenting medan du skriv eller brukar eit diagram med tastaturet.

## [0.4.3] - 2026-09-19

### Changed
- FinStats hugsar kva kvar side sist viste: går du tilbake til ein profil, eit bibliotek eller eit tidsrom du alt har sett på, vert det teikna med ein gong, profilbilete og alt, medan ferske tal vert lasta bak, og sida vert berre teikna på nytt om noko har endra seg. Det som vert hugsa, ligg i fana i nettlesaren og vert forkasta når du loggar ut eller endrar noko.
- Plasshaldarar for lasting dukkar berre opp når lastinga faktisk er treg. Før viste kvar side dei i minst ein tredjedel av eit sekund.

### Fixed
- Bibliotek som vart sletta i Jellyfin og aldri hadde ei avspeling, vert ikkje lenger lista. Eit sletta bibliotek med historikk vert framleis vist, merkt som fjerna.

## [0.4.2] - 2026-09-19

### Fixed
- På Brukarar-sida stod **Admin**-merket på ei eiga linje og gjorde den rada høgare enn dei andre. Merka står no ved sida av namnet.

## [0.4.1] - 2026-09-19

### Fixed
- Ordet «null» dukkar ikkje lenger opp under tittelen på Oversikt utanom desember og januar, når det ikkje er noko tilbakeblikk-banner å vise.

## [0.4.0] - 2026-09-19

Sjå kva som er endra utan å forlate appen

### Added
- **Endringslogg**-fana, som viser kvar utgjeving, med versjonen du køyrer merkt.
- Versjonen i statuslinja lenkjer dit, og fana viser ein prikk etter ei oppdatering til du har sett på ho.

### Changed
- Å kople historikk til omdøypte element på nytt går mykje fortare: frå 18 sekund til ein tredjedel av eit sekund med 666 foreldrelause titlar, så oppstarten held seg augneblinkleg når historikken veks.

## [0.3.3] - 2026-09-19

### Changed
- Biblioteket vert lese på nytt rett etter at jobben **Scan Media Library** i Jellyfin er ferdig, aldri på ein eigen tidtakar og aldri medan ei skanning køyrer. FinStats har alltid berre lese frå Jellyfin, og det startar aldri ei skanning der.
- Ei vekentleg lesing som tryggleiksnett dekkjer serverar som nyttar sanntidsovervaking, der skanningsjobben kanskje aldri køyrer.
- Det gamle intervallet er no berre ei reserveløysing, for når følgjinga er slått av eller serveren ikkje melder om nokon skanningsjobb.

### Added
- Brytaren **Følg biblioteksskanninga i Jellyfin** under Innstillingar → Innsamling, på som standard.

## [0.3.2] - 2026-09-19

### Fixed
- Historikken overlever namnebyte. Jellyfin gjev ei fil som har fått nytt namn, ein ny ID, og det gjorde at dei tidlegare avspelingane vart ståande att under det rå mappenamnet (`Title (2010) [tmdbid-…] [imdbid-…]`) utan plakat. Desse avspelingane vert no kopla til den noverande oppføringa i biblioteket: via TMDB-, IMDb- eller TVDB-ID når det gamle namnet har ein, elles via reinska tittel og år, og episodar via serie pluss sesong- og episodenummer. Eit treff må vere eintydig, så titlar som verkeleg vart sletta, vert ståande som dei er.
- Direkte-TV-kanalar importerte frå Jellystat vart rekna som filmar, fordi Jellystat ikkje registrerer nokon elementtype. Dei vert no kjende att som direkte-TV, og nye avspelingar av direkte-TV vert registrerte med typen frå Jellyfin.

### Changed
- Direkte-TV vert heilt halde utanfor tilbakeblikket: topplister, totalar, personlegdom, rekordar og plassering.

### Added
- `finstats relink` for å kople historikk på nytt når du vil. Det køyrer òg ved oppstart og etter kvar lesing av biblioteket.

## [0.3.1] - 2026-09-19

### Changed
- Tilbakeblikket er strengt personleg: alle, administratorar òg, ser berre sitt eige, og «Alle»-visinga, brukarveljaren og serverkapitlet er borte.
- Tilbakeblikket opnar på året som er «klart»: inneverande år i desember, elles året før. Andre år og «Siste 12 månader» er framleis eitt klikk unna.

## [0.3.0] - 2026-09-19

Året ditt i tilbakeblikk

### Added
- **Tilbakeblikk**-fana, ei historie å rulle gjennom om eitt år: timar med sjåing, toppseriar, filmar, spor og sjangrar, og ein sjåarpersonlegdom (nattugle, morgonfugl, helgekrigar, maratonsjåar, filmelskar, musikkelskar, vanedyr) med diagram over tid på døgnet og vekedag.
- Året månad for månad, med tittelen som prega kvar månad.
- Rekordar: største dag, største maraton, lengste rekkje dagar på rad, lengste enkeltavspeling, tittelen du såg oftast om att, første avspeling i året og den eldste tittelen du såg.
- Oppdagingar: nye seriar påbyrja, filmar og episodar fullførte, og seriane som fekk nøyaktig éin episode.
- Eit banner på éi linje på Oversikt i desember og januar når eit tilbakeblikk er klart.

## [0.2.0] - 2026-09-19

Meir enn Jellystat nokon gong registrerte

### Added
- Ei tidslinje for kvar avspeling: start, pause, framhald, hopp, byte av lyd og undertekstar, direkteavspeling som vert til omkoding, og stopp. I tillegg tal på pausar og hopp, kvar avspelinga heldt fram frå, og om sjåaren var på det lokale nettverket.
- **Server**-fana: Jellyfin-versjon, ventande oppdateringar og omstartar, diskbruk per bibliotek (Jellyfin 10.11+), programtillegg, resultat frå planlagde oppgåver og kvar registrerte eining.
- Innsikt i **avspeling**: samtidige straumar over tid, kva klientar som tvingar fram omkoding, kor langt folk kjem før dei stoppar, lokale avspelingar mot avspelingar utanfrå, og sjåvanar.
- Kva biblioteket ditt er laga av: oppløysingar, kodekar, dynamisk område, containerar, titlar per tiår, tilskot per månad, dei største titlane, og titlane ingen nokon gong har sett, kontrollert mot både historikken i FinStats og avspelingsmerka i Jellyfin.
- Oversikt: flest samtidige straumar, estimert mengd strøymde data, delen avspelingar utanfrå, sjangrar etter visingstid og mislukka innloggingar.
- Elementsider: lenkjer til IMDb og TMDB, studio, bitdjupn og biletfrekvens, og kven som har tittelen merkt som spela av i Jellyfin.
- Brukarsider: sjangrar, og filmane, episodane og favorittane Jellyfin har registrert.

### Fixed
- Tidsetiketten på eit Spelast no-kort renn ikkje lenger ut av kortet.
- Lange etikettar på Avspeling-sida vart kutta for tidleg.
- Ein serie som har forlate biblioteket, viser framleis sesong- og episodelista si.

## [0.1.0] - 2026-09-19

Første utgjeving

### Added
- Direkte innsamling frå øktene i Jellyfin: brukar, tittel, klient, eining, IP-adresse, avspelingsmetode, grunnar til omkoding og maskinvareakselerasjon, detaljar om bilete, lyd og undertekstar, og visingstid mot tid på pause.
- Innlogging med Jellyfin-kontoen din. Eit oppsett i to steg lagar ein eigen API-nøkkel for FinStats, og passord vert aldri lagra. Brukarar som ikkje er administratorar, kan om ønskjeleg logge inn for å sjå berre sin eigen statistikk.
- Sidene Oversikt, Aktivitet, Brukarar, Bibliotek, Avspeling og Serverlogg, ein kommandopalett (Ctrl/⌘ K) og ei statuslinje som oppdaterer seg direkte.
- Import frå Jellystat: store tryggingskopiar vert strøymde til disk og importerte på sekund, og det er trygt å importere same fila to gonger.
- Éi lita binærfil med innebygd nettgrensesnitt og éi SQLite-fil, pluss eit Docker-bilete.
