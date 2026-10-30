# Endringslogg

Alt som er endret i FinStats, nyeste først. Denne filen bygges inn i programmet og er det
fanen **Endringslogg** i appen viser.

Format: `## [version] - date`, eller `## [version] - first day to release day` for en versjon som ble
laget over flere dager, deretter `### Added`, `### Changed`, `### Security`, `### Performance`, `### Stability`, `### Fixed`
eller `### Removed` (Nytt, Endret, Sikkerhet, Ytelse, Stabilitet, Rettet, Fjernet) med ett punkt per endring. Sikkerhet
er noe noen kunne ha misbrukt, Ytelse er noe som er blitt raskere eller lettere, Stabilitet noe som ikke lenger kan krasje, stanse, lekke eller
miste data, og Rettet noe som ga feil svar. En `x.y.0`-versjon har en kort tittellinje under overskriften:
den tittelen bruker appen som overskrift for hele serien.

## [2.3.1] - 2026-10-26 to 2026-10-28

### Security
- **Oppsettet ber om en engangskode**: til FinStats er satt opp, skriver den en kode i loggen sin når den starter (`docker logs finstats`), og oppsettet ber om koden før det tester en adresse eller setter opp noe. Ellers kunne den som nådde porten først, peke FinStats mot sin egen Jellyfin og bli administrator der, eller bruke testen til å se seg om i nettverket.
- **Bibliotektilgangen følger Jellyfin**: hver person ser bare bibliotekene Jellyfin åpner for dem, og i dem bare titlene Jellyfin viser dem under aldersgrensen, taggene og reglene for titler uten aldersgrense, i bibliotekssider, søk, medvirkende, bilder, Nylig lagt til, Min liste, Kommer snart, kalenderabonnementet og varsler om nye titler. Alle som var logget inn, kunne bla i alle biblioteker.
- En lenke kunne sende noen fra velkomsten til et annet nettsted rett etter at de logget inn. Nå følges bare FinStats' egne sider.
- Serverloggen og de mislykkede innloggingene i innsikten ga alles innloggingsadresser og aktivitet til alle med *Se server*. Andres linjer krever *Se alle*, og en adresse *Se nettverk*, som overalt ellers.
- En personlig mottaker kunne nå det lokale nettverket gjennom et navn som svarte annerledes andre gang det ble slått opp. Nå kobles det til adressen som ble sjekket, flere adresseområder som aldri når internett, regnes som lokale, husstandens egen offentlige adresse avvises også, og et navn som ikke kan slås opp, avvises med de samme ordene som en lokal adresse.
- Bak en proxy som legger til en egen `X-Forwarded-For`-linje, eller fra et IPv6-nettverk, kunne hvert innloggingsforsøk komme fra en ny adresse, så grensen slo aldri inn. Nå får et IPv6-/48 50 forsøk på fem minutter, en full grense avviser en ny adresse i stedet for å glemme en gammel, hele serveren stopper ved 60 feil i minuttet, og en mislykket innlogging føres i revisjonsloggen én gang per adresse og tidsrom.
- `FINSTATS_TRUST_PROXY` navngir proxyene (adresser eller adresseområder) hvis `X-Forwarded-For`, `X-Forwarded-Host` og `X-Forwarded-Proto` blir trodd, og `1` eller `true` stoler nå bare på en motpart på loopback eller et privat nettverk. Slått på kunne hvem som helst velge sin egen adresse; slått av bak en proxy stengte ti feil gjetninger alle ute.
- Når brukerne er lest, besvares en innlogging for et navn som er ukjent, deaktivert, fjernet eller ikke sluppet inn i FinStats, som et feil passord uten at Jellyfin blir spurt, og ikke før Jellyfin ville ha svart. Slike navn kunne låse Jellyfin-kontoer, komme seg rundt Jellyfins regel for fjerntilgang eller gjemme seg bak FinStats' adresse.
- Den Jellyfin holder til hjemmenettet (*Tillat eksterne tilkoblinger* av), logger inn i FinStats bare derfra, siden Jellyfin bedømte bryteren ut fra FinStats' egen adresse, og en brukerlesing som ikke sier om noen er deaktivert, holder dem ute.
- En øktcookie merkes Secure når siden det logges inn fra, er på https, også bak en proxy som ikke sier det.
- En innlogging som lykkes, tilgir ikke lenger gjetningene før den. På https er øktcookien en `__Host-`-cookie som et nettsted på et nabonavn ikke kan erstatte, og den godtas ikke under noe annet navn, og en utlogging avslutter hver økt nettleseren sendte.
- Bare en kalendernøkkel åpner kalenderabonnementet fra adressen; andre nøkler sendes i headeren.
- En API-nøkkel kan ikke lenger gi tillatelser, endre hvem som slipper inn eller hva som er offentlig, gjenopprette en sikkerhetskopi, eller legge til, endre eller teste en tjeneste eller en varslingsmottaker: alt dette varer lenger enn nøkkelen, så det krever en innlogget nettleser.
- Hvert svar har `Cross-Origin-Opener-Policy: same-origin` og en `Permissions-Policy`, API-svar har `Cross-Origin-Resource-Policy: same-origin` og er `no-store` med mindre de sier noe annet, og CSP-en tillater ikke lenger `blob:`-bilder. En side som åpnet FinStats, kunne senere gjøre fanen om til en innloggingsside av sitt eget.
- Der FinStats vet hvilket skjema den nås med (en klarert proxys `X-Forwarded-Proto`, eller den offentlige adressen), avvises en skriving fra et annet skjema som et annet nettsted.
- `/auth/me`, en tittels side og Bibliotekhelse lenker til Jellyfin på den offentlige adressen; adressen FinStats selv leser Jellyfin på, vises bare til administratorer.
- Et bilde hentes fra Jellyfin bare for noe FinStats har, også plakatene og kortene på en publisert profil, så bilder fra andre biblioteker kan ikke bes om med id.
- Nøklene til en privat tracker fjernes også fra feilene i nedlastingskøen, etter formen så vel som navnet, og den som bare har *Se nedlastinger*, får vite hvilken tjeneste som feilet og hvordan, aldri dens interne adresse eller hvor den videresender.
- Serversiden viser hvem som brukte en enhet sist bare med *Se alle*, og Ukoblede medier krever det i tillegg til *Administrere*. Uten det teller Oppgaver bare egne avspillinger og strømmer, et fjernet bibliotek vises ut fra egne avspillinger, og en ventende Tautulli-tavle kan ikke avbrytes.
- Innstillingene viser tjenesten for offentlig adresse bare med vertsnavnet, og Jellyfins adresse og datamappen bare til administratorer, og en tittel biblioteket har mistet, beskrives bare ut fra avspillinger leseren kan se.
- En sikkerhetskopi fra Tautulli med tabeller som ikke var vanlige tabeller, kunne holde all annen skriving ventende til FinStats ble startet på nytt. En slik fil avvises, og SQLite leser sikkerhetskopien defensivt.
- En opplasting over 4 GB avvises, og en zippet sikkerhetskopi fra Tautulli pakkes ikke ut lenger enn størrelsen den oppgir; begge deler kunne fylle disken FinStats' egen database ligger på.
- Et svar fra Jellyfin, Sonarr, Radarr, Seerr, oppslaget av offentlig adresse eller GeoIP-nedlastingen leses ikke lenger enn det kan være, regnet etter utpakking.
- Enhver Jellyfin-bruker kunne stoppe FinStats fra å registrere avspillinger for alle ved å melde tusenvis av korte avspillinger av én tittel. Å finne ut hvem som så sammen, tar nå én gjennomgang og holder ingen lås imens.
- Avspillinger etter hverandre, eller én tittel spilt på rundgang fra enhet etter enhet, slår ikke lenger fremmede sammen til én gruppe, og den som bare kan se seg selv, får bare vite om dem avspillingen deres gikk side om side med, og for hvor lenge.
- En lenke kunne fortsatt sende noen fra velkomsten til et annet nettsted etter innlogging, gjennom en sti som `/.//`.
- Den som har *Administrere*, kan ikke lenger endre hvor mange sikkerhetskopier som beholdes eller hvor ofte de skrives, hvor lenge en omstart regnes som samme avspilling (nå høyst en time) eller når brukere, aktivitetsloggen og sikkerhetskopier leses, og sikkerhetskopiene deres skyver aldri ut en administrators; hjemmeadresser og reglene for umulig reise krever det varslene krever, og import av historikk krever også *Se alle*.
- En lagret nøkkel, token eller passord sendes bare til serveren den ble lagret for, og en varslingsmottaker opprettes eller endres bare i en innlogget økt, aldri med en API-nøkkel. En administrator kan slå av eller fjerne andres personlige mottaker, men ikke flytte den, slå av sertifikatsjekken eller lese ntfy-emnet eller Pushover-brukernøkkelen.
- Webhooker til Discord og Slack må være `https`, siden adressen er nøkkelen.
- En flom av mislykkede innlogginger kan ikke lenger skjule en ekte innlogging eller drukne varslene: navn som ikke er kontoer, gir ett varsel i kvarteret som ikke nevner noen, varsler sendes først, og Slack og Discord viser det en melding sier som tekst og pinger ingen.
- Rangeringen i tilbakeblikket, Kapasitet, et varsel og en publisert profil sier ikke mer enn sidene gjør: ingen rangering eller andel av alles tid til den som bare kan se seg selv, travle øyeblikk bare som dager med tre strømmer eller flere uten *Se alle*, ingen mislykket jobb eller tilkobling til den som ikke kan administrere FinStats, og en offentlig profil endres én gang om dagen.
- En sikkerhetskopi fra Tautulli med en generert kolonne avvises, og et bilde hentes for en importert avspilling først når biblioteket har tittelen.
- En gjenoppretting åpner aldri FinStats mer enn den er: innstillinger kommer bare tilbake når det blir bedt om, innlogging, tillatelser og offentlige profiler blir det både denne installasjonen og sikkerhetskopien tillater, og de offentlige adressene forblir denne installasjonens. `finstats restore <file>` henter bare tilbake historikken, slik siden gjør, og innstillingene også med `--with-settings`.
- En gjenoppretting av bare historikken henter ikke lenger tilbake hjemmeadresser, dempede varsler, revisjonsoppføringer som gir seg ut for å være denne installasjonens egne, eller lokaliserte titler biblioteket mangler, og ingen gjenoppretting henter tilbake avspillinger, loggrader eller enheter datert før 1990 eller etter i morgen. En gjenopprettet avspilling regnes som importert uansett hvilken kilde den oppgir, så en omstart fortsetter den aldri.
- Datamappen er bare for FinStats' egen bruker (0700, filene 0600, `umask 077` i imaget). FinStats nekter å kjøre som root over en datamappe en annen bruker eier, og nevner `docker exec -u` som skal brukes, og oppstartsskriptet endrer eier bare på mappen og FinStats' egne oppføringer.
- Navnet på en mottaker kan ikke inneholde kontrolltegn, og navnet på en enhet, en person eller en tittel kan ikke lenger forfalske en linje i loggen.
- En parameter i en lenke kan ikke lenger nå en annen API-sti enn sidens egen, og FinUIs elementbygger setter en adresse bare når den er http(s) eller relativ, aldri `srcdoc`.
- Søk, publiserte profiler og arbeid startet for hånd kan ikke lenger holde FinStats opptatt: et søk stopper ved 200 tegn, en profil lagres per språk, kort venter i en liten kø, og en jobb, adresseoppslaget, nedlastingen av stedsdatabasen og en testmelding starter bare så ofte, uansett hvordan de blir bedt om, med testmeldinger talt per avsender. Å hente stedsdatabasen krever også det Sikkerhet-siden krever.
- Innholdet i en forespørsel må fortsette å komme når de første 15 sekundene er over, et svar ingen leser, blir gitt opp etter 30 sekunder, og én adresse holder høyst 64 tilkoblinger, en betrodd proxy unntatt. Noen hundre forespørsler som aldri sendte innholdet sitt, kunne hindre FinStats i å svare noen.
- En e-postserver leses ikke lenger enn noe svar kan være (1 MB for et helt brev), og en testmelding stopper etter like lang tid som en melding i køen kan ta. En server som aldri avsluttet en linje, kunne få FinStats til å holde mer og mer minne til den ble stoppet.
- En bibliotekslesing vurderes mot titlene lesingene har lagret, ikke dem en import tok med, og en Jellystat-tittel i et bibliotek som allerede er lest, kommer inn som borte. En import av oppdiktede titler kunne få hver bibliotekslesing til å stoppe FinStats, ved hver oppstart.
- Grensen på 60 mislykkede innlogginger i minuttet for hele serveren teller bare forsøk som sendes til Jellyfin, også de som fortsatt venter på svar. Navn FinStats avviste selv, lot hvem som helst med noen få adresser stoppe alle fra å logge inn, og mange gjetninger sendt samtidig nådde alle Jellyfin.
- Den Jellyfin holder til hjemmenettet, holdes der ved hver forespørsel, økten og nøklene deres også, ikke bare når de logger inn; hjemme er et privat nett eller husstandens oppslåtte offentlige adresse, aldri en adresse skrevet inn som hjemme, og bak en proxy `FINSTATS_TRUST_PROXY` ikke nevner, holdes de ute. En informasjonskapsel eller nøkkel tatt ut av huset åpnet alt fra hvor som helst, og en forvalter kunne åpne huset for en adresse de selv valgte.
- En API-nøkkel kan ikke lenger skru av det som ville merke at den misbrukes, eller sikkerhetskopiene som ville gjøre om det den gjorde: hvor mange sikkerhetskopier som beholdes og hvor ofte, sammenslåingsvinduet, hjemmeadresser og reiseregler, timeplanene for brukere, aktivitetsloggen og sikkerhetskopier, å dempe eller løse alle varsler, en sikkerhetskopi i papirkurven eller å fjerne et varslingsmål. Hver av dem varte lenger enn nøkkelen når den ble trukket tilbake.
- Plakatene fra Sonarr og Radarr og ønskelistens neste episode følger også bibliotektilgangen: en serie Jellyfin skjuler for noen, har verken plakat eller dato for dem. Å telle tjenestenes ID-er avslørte hvilke skjulte serier som kom, og en liste nevnte neste episode av en skjult serie.
- Sammen-kapittelet i årsoversikten nevner bare dem som så samtidig med deg, slik Sammen-siden gjør. En gruppe du kom inn i gjennom noen andre, nevnte folk du aldri så sammen med.
- En import fra Jellystat eller Streamystats leser hele filen før den tar skrivelåsen i databasen, og skriver så bare radene den beholder. Lest under låsen kunne en fil med noen få ekte rader og gigabyte med hva som helst annet holde all annen skriving ventende, avspillinger og innlogginger også.
- En gjenoppretting holder én tilgang per person, for ikke flere personer enn en husstand mange ganger over, og sjekker en revisjonsoppføring for dobbel uten å lese hver oppføring den nettopp har skrevet. En laget sikkerhetskopi kunne holde minne og databasens skrivelås lenger og lenger mens den ble gjenopprettet.
- En gjenoppretting med innstillingene slår på oppslaget av den offentlige adressen og nedlastingen av stedsdatabasen bare der denne installasjonen også har dem på, og beholder aldri færre sikkerhetskopier eller skriver dem sjeldnere enn denne installasjonen gjør. En sikkerhetskopi som beholdt én, kunne få neste kjøring til å fjerne alle andre.
- En filstørrelse ingen disk kunne romme, leses som ingen størrelse, og størrelser legges sammen slik at ingen sum kan flyte over. Én importert størrelse fikk bibliotekssidene til å svare med feil for godt.
- Når FinStats vet at den nås over https (den offentlige adressen er https, eller en betrodd proxy sier det), tas en økt bare fra `__Host-`-informasjonskapselen, og med en offentlig https-adresse er hver nye økt en https-økt. En innlogging sendt uten `Origin` gjennom en proxy som ikke sa noe, laget en vanlig økt som et nabonettsted kunne plante i noens nettleser.
- Bildet bygges med hver GitHub-handling festet til en commit og begge grunnbildene til et sammendrag, flyttet bare av Dependabot, og en utgivelse bygges uten den delte byggebufferen. En flyttet tagg på andres handling kunne ha publisert hvilket som helst bilde som FinStats.
- Stilarket til FinUI, tegnet i besøkerens eget utseende, kan bare lastes av FinStats' egne sider. En side på en annen port på samme vert kunne lese av det om besøkeren var innlogget, og hvilken forhåndsinnstilling de valgte.

### Performance
- En publisert profil regnes ut høyst én gang i minuttet, ikke på nytt for hvert bilde og kort på den.
- En persons historikk gjennomsøkes etter sikkerhetsvarsler én gang om gangen, uansett hvor mange avspillinger som starter.
- Å logge inn eller ut, trekke tilbake en nøkkel eller endre én persons tillatelser ber bare de direkte tilkoblingene det gjelder om å se etter på nytt.
- Koblingstavlen for Tautulli leser historikken én gang, ikke én gang per Plex-bruker.
- Et bilde som mangler i hurtigbufferen, hentes av én forespørsel mens de andre venter, og et kilden ikke klarte å gi, spørres ikke etter igjen før om 30 sekunder.
- Filene i nettgrensesnittet gzippes én gang og beholdes, og skrifter komprimeres ikke på nytt.

### Stability
- En sikkerhetskopi i Jellystats eldre format leses én rad om gangen i stedet for hel, og en linje lengre enn noen sikkerhetskopi har, avvises.
- To opplastinger fra Tautulli samtidig skriver ikke lenger i samme fil.
- Et sidetall langt forbi slutten av et biblioteks titler eller av Bibliotekhelse gir en tom side, ikke en feil.
- En import eller gjenoppretting der opplastingen ble brutt, avviser ikke lenger alle andre til FinStats startes på nytt, og verken den eller en import som ble forlatt før den begynte, etterlater en fil.
- Én rad i en sikkerhetskopi leses ikke lenger enn 8 MB, og én sikkerhetskopi fra Tautulli tas imot om gangen.
- En avspilling beholder høyst 2 000 hendelser, og en import hopper over en avspilling som ikke starter og deretter slutter mellom 1990 og i morgen.
- Ett sterkt negativt episodenummer henger ikke lenger hver bibliotekslesing: Bibliotekhelse (også sammendraget), Kapasitet, ny kobling og lokalisering legger sammen og sammenligner tall uten å gå rundt, og et release-bygg sjekker for overflyt, så noe som er oversett, stopper én oppgave i stedet for å gå i ring.
- Et sesong-, episode- eller sluttnummer beholdes bare der Jellyfin kunne ha lagret det (0 til 2 147 483 647), fra Jellyfin, Jellystat og Tautulli likt.
- En sikkerhetskopi fra Tautulli der id-ene gjentar seg, avvises, siden noen MB rader under én id fikk prosessen til å gå tom for minne, og en sikkerhetskopi åpnes som uforanderlig og etterlater ingen `-wal` eller `-shm`.
- En tilkobling må sende forespørselen sin innen 15 sekunder, og høyst 512 betjenes samtidig: noen hundre halvåpne tilkoblinger tok FinStats ned. En melding på direktekanalen holdes til 4 KB.
- Noen få kilobyte fra Sonarr, Radarr eller Seerr kan ikke lenger bli gigabyte med minne: en liste beholder ikke mer enn FinStats leser av den.
- En varslingsmottaker som aldri svarer, holder bare seg selv igjen: mottakere får meldinger side om side, og en som stadig får tidsavbrudd, får være i fred i ti minutter.
- Tid igjen og samlet hastighet i en kø, tallene i en gjenopprettet avspilling og dagen en avspilling forlater papirkurven kan ikke lenger flyte over.
- En gjenoppretting leser hele filen før den tar skrivelåsen, avviser en som pakkes ut til langt mer enn størrelsen sin, og skriver i puljer, så innsamleren fortsetter å registrere imens.
- Sikkerhetskopier navngis i UTC og skrives aldri over en fil som finnes, og bare FinStats' egne kopier fra før oppdateringer ryddes bort.
- En rad i serverloggen datert i fremtiden stopper ikke lenger lesingen av aktivitetsloggen: den leses fra den nyeste raden som ikke er i fremtiden.
- Forespørsler som kommer samtidig til en publisert lenke, regner ut svaret én gang, og kortene tegnes i en egen kø, så den som har lenken, ikke lenger kan få eierens kort fra tilbakeblikket til å svare 503.
- Avspillingene i et bibliotek som mangler i Jellyfins liste over biblioteker, venter to dager før de flyttes til et annet biblioteks kopi av samme tittel. Én liste som utelot et bibliotek, flyttet dem straks, og de ble værende flyttet da biblioteket kom tilbake.
- En disk Jellyfin oppgir som større enn noe tall kan romme, får ikke lenger hver lesing av serverdetaljene til å feile.

### Fixed
- Mislykkede innlogginger på en konto gir ett varsel per tidsrom som blir til et alarmvarsel ved tjue, og et navn teller mot en konto bare når det er skrevet helt ("x alice" teller ikke lenger mot alice).
- Meldinger til Discord og Slack escapes først og kortes ned etterpå: et enhetsnavn fullt av Markdown fikk Discord til å avvise hvert varsel som nevnte det.
- En offentlig IPv6-adresse godtas for en personlig mottaker.
- Bak en proxy som selv logger folk inn (nginx `auth_basic`), avvises ikke lenger FinStats' cookie.
- Et visningsnavn som inneholder `{{url}}`, vises slik det er skrevet i forhåndsvisningen av en publisert side.
- En oppsummering for år 9999 viser standardåret i stedet for en feil.
- En side som mistet en tillatelse og fikk den tilbake mens den var åpen, holdes oppdatert over direktekanalen igjen uten å lastes på nytt.

### Removed
- **Synnmørsk** er ikke lenger et av FinStats' språk: en nettleser som valgte det, leser nynorsk, skriftspråket det ble laget fra.

## [2.3.0] - 2026-10-07 to 2026-10-26

Kapasitet, direktekanal, Tautulli-stil og språk

### Added
- **Språk**: FinStats snakker 24 språk. Britisk og amerikansk engelsk og norsk bokmål og nynorsk er skrevet og kontrollert av utvikleren, som også kontrollerer synnmørsk, utviklerens egen dialekt fra Søre Sunnmøre; dansk, tysk, spansk, finsk, fransk, hindi, islandsk, italiensk, japansk, koreansk, nederlandsk, polsk, portugisisk (Brasil), russisk, svensk, tyrkisk, ukrainsk og kinesisk (forenklet og tradisjonell) er oversettelser ingen her har kontrollert, og viser på engelsk det de ikke har ennå. Alle sidene i begge oppsettene, den offentlige profilen, Endringslogg og det serveren svarer med ord (en feilmelding, kortene i Tilbakeblikk, hva en jobb i Jellyfin gjør) vises på språket som er valgt under Innstillinger → Utseende → Språk, per nettleser; til et språk er valgt, avgjør nettleserens eget språk, og en nettleser på et språk FinStats ikke snakker, får amerikansk engelsk. Tall, datoer, enheter og lister skrives slik det språket skriver dem. Et skript kan be om et språk med `Accept-Language`.
- **Tautulli-stil**: et andre oppsett for dem som kommer fra Plex, bygd opp slik en klassisk statistikkserver for Plex er. En linje øverst i stedet for menyen, og bare statistikksidene: Hjem, Biblioteker, Brukere, Historikk, Grafer og Innstillinger, med et tannhjul for profilen din, Innstillinger, Endringslogg, temaet og utlogging. Velges per nettleser under Innstillinger → Utseende → Oppsett. Den laster bare sin egen halvdel av FinStats og starter derfor raskere; Tilbakeblikk, Sammen, På vei, Sikkerhet og serversidene finnes i FinStats-oppsettet. Den bruker FinStats' egne farger, lyst og mørkt, og ingenting i den beveger seg. Ikke tilknyttet Tautulli.
- **Hjem i Tautulli-stilen**: det som spilles nå, med et kort per strøm (hvordan den spilles av, hjemmefra eller utenfra, når den er ferdig), visningsstatistikken som rangerte kort etter antall avspillinger eller varighet de siste N dagene, hva hvert bibliotek inneholder, og hva som har kommet nylig.
- **Grafer i Tautulli-stilen**: avspillinger etter medietype (dag for dag, etter ukedag og time, de ti mest brukte appene og personene), etter hvordan de ble spilt av (dag for dag, flest strømmer samtidig hver dag, filens og den sendte oppløsningen, apper og personer) og måned for måned, etter antall avspillinger eller varighet, for alle eller for de valgte personene.
- **Biblioteker, Brukere og titler i Tautulli-stilen**: tabeller over alle biblioteker og personer med hva som sist ble spilt av, på hva og hvorfra (adressen bare med *Se nettverk*), og en side for hvert bibliotek, hver person og hver tittel med tallene sine og sin egen historikk.
- For skript: `/api/stats/split` (avspillinger etter app, person, oppløsning, dag eller måned, delt etter medietype eller etter hvordan de ble spilt av); topplister etter antall ulike seere (`sort=users`); aktivitet avgrenset til avspillinger som regnes som sett (`watched=1`); innsikt bare for delene det spørres om (`parts=`); toppbelastningen delt i omkodinger, direktestrømminger og direkteavspillinger, også per dag; hva hvert bibliotek inneholder og hva som sist ble spilt av der; nylig lagt til etter type (`kind=`); varmekartet etter medietype; en persons siste avspilling; om en strøm spilles av hjemmefra.
- **En direktekanal**: Oversikt, nedlastingene i På vei og statuslinjen holder seg oppdatert over én WebSocket per fane (`/api/live`), så det som spilles og lastes ned, endrer seg på skjermen idet det skjer, ikke ved neste titt. Bak en omvendt proxy som ikke slipper WebSocket-oppgraderinger gjennom, eller på en tilkobling som falt bort uten et ord, spør sidene med noen sekunders mellomrom som før. `/api/now-playing`, `/api/downloads` og `/api/summary` er som før, for skript.
- **Server → Kapasitet**: klarer denne serveren seg, ut fra det FinStats har sett? De travleste øyeblikkene som er registrert, og de siste 90 dagenes: strømmer samtidig, og hvor mange som fikk bildet omkodet med maskinvare, uten, eller uten at noe ble oppgitt (en historikk importert fra Tautulli er Plex' og holdes utenfor). Hver disk bibliotekene ligger på, og hvor fort det Jellyfin har lagt til de siste seks månedene, fyller den (en disk som ble fylt opp nylig, måles fra en måned etter den første filen, siden den første måneden er selve påfyllingen), og, der Jellyfin 10.11 eller nyere oppgir ledig plass, hvor lenge plassen holder i det tempoet. FinStats sier aldri hvor mange strømmer serveren kunne ha tålt.
- **Eksporter .ics** under På vei → Kommer: listen slik du har filtrert den, lastet ned som en kalenderfil du importerer én gang. Den oppdaterer seg ikke selv; en kalendernøkkel fra Innstillinger → API-nøkler gir fortsatt en kalender som gjør det.
- **Verdt en titt**: Kapasitet sier fra når hver omkoding Jellyfin har rapportert om de siste 90 dagene, gikk uten maskinvareakselerasjon mens omkoding er vanlig, når biblioteksskanningen i Jellyfin går en fjerdedel av tiden eller mer, og når en disk har plass til mindre enn seks måneder til. Hvert punkt kommer med tallene sine og kan legges til side, slik som funnene i Bibliotekhelse.
- **Hva biblioteket ditt består av** kan avgrenses til ett bibliotek eller flere, både under Biblioteker og på siden til hvert bibliotek (`library_id=a,b`).
- **Dato- og tidsformat** under Innstillinger → Utseende: en dato som 31.01.2026, 31/01/2026, 31-01-2026, 01/31/2026, 2026-01-31, 2026.01.31 eller 2026/01/31, og et klokkeslett med 24 eller 12 timer, med eller uten 0 foran, eller begge slik språket skriver dem. Lagres i denne nettleseren.
- **En velkomst**: første gang noen logger inn, uansett nettleser, og på slutten av oppsettet, spør FinStats om språk, dato- og tidsformat, lyst eller mørkt og oppsett, med et bilde av hvert oppsett. Hopp over beholder britisk engelsk, 31/01/2026, 13:05, enhetens farger og FinStats.

### Changed
- Logoen inne på en side er det rene merket ved siden av navnet, satt i Inter uansett hvilken skrift et forhåndsvalg bruker; der den står alene (favikonet), beholder den den avrundede flisen.
- En avspilling som er omkodet i programvare, registreres som det der Jellyfin oppgir det; før så den ut akkurat som en fra en server som ikke oppgir det. Sidene kaller fortsatt begge Programvare.
- En oppdatering kjører nå, én gang og eldste versjon først, det en versjon trenger å gjøre med en database som en eldre FinStats har skrevet, ved siden av migreringene. I 2.3.0 leses serverens detaljer ved første oppstart, så Kapasitet er klar med en gang.

### Performance
- **En side som står åpen, spør ikke om noe**: Oversikt gjorde 30 forespørsler i minuttet for å holde seg oppdatert og nedlastingene i På vei 18; med direktekanalen gjør begge ingen. Uansett hvor mange faner som er åpne, spør FinStats ikke Jellyfin, Sonarr og Radarr om noe mer enn før, og en skjult fane lar fortsatt nedlastingskøen leses sjeldnere.
- En persons side leser bare den personens avspillinger: totalene og den siste avspillingen ble regnet ut over alle avspillinger på serveren og så avgrenset til personen, rundt fire sekunder på en million avspillinger.
- Et biblioteks side leser bare det bibliotekets titler og avspillinger, der den før talte opp alle bibliotekenes og beholdt ett: en tredjedel av tiden på en million avspillinger.
- Når Jellyfins klokke gikk foran FinStats', leste *Etter biblioteksskanning i Jellyfin* hele biblioteket på nytt hvert minutt til forskjellen var passert; nå følges hver skanning én gang.
- En GeoIP-fil som ikke kunne åpnes, ble kopiert på nytt, hundre megabyte og mer, og hver adresse ble plassert på nytt, hvert kvarter; nå prøves den igjen først når den endres.
- Bibliotekhelse åpnes med en gang etter at pekeren har hvilt på lenken: det som var hentet på forhånd, ble lagt et sted siden aldri så etter.

### Stability
- En oppdatering som feilet etter at en migrering hadde kjørt, og som Docker straks startet på nytt, tok en ny kopi av den halvveis oppdaterte databasen ved hver oppstart, og tre oppstarter ryddet bort den ene kopien den forrige versjonen kunne åpne. Kopien fra før oppdateringen blir nå beholdt.
- En tittel som ble slettet i Jellyfin mens biblioteket ble lest (en fil som Radarr byttet ut), kunne få lesingen til å overse en annen tittel som fortsatt var der, og merke den som fjernet, og avspillingene kunne så bli flyttet over til en annen kopi av den. Det en lesing ikke så, blir nå først etterspurt via ID.
- Gjenoppretting av en sikkerhetskopi tatt før en fil ble omdøpt eller oppgradert, gjenopprettet én gang til hver avspilling som siden var flyttet over til den nye filen, og doblet den for godt.
- En forespørsel i svaret fra Seerr som FinStats ikke kunne lese (et felt som fikk nytt navn i en Seerr-oppdatering), ble merket som fjernet av den daglige fullstendige lesingen.
- En oppgave som traff en feil, ble stående som «kjører» for godt, og ingen utløser startet den igjen før FinStats ble startet på nytt; nå avsluttes den som mislykket.
- En skadet Tautulli-zip stoppet opplastingen før filen ble fjernet, så den ble liggende til opprydningen etter seks timer, med tilgangstokenene til alle Plex-brukerne.
- En import eller gjenoppretting som ble avbrutt mens filen kom inn, etterlot filen i datamappen for godt, med adresser og alt; nå fjernes den ved neste oppstart.
- Et bilde kunne bli servert halvskrevet mens det ble lagt i cachen, og nettleseren beholdt det da slik.
- Tilbakekalte API-nøkler ble tatt vare på for godt, uansett hvor mange som ble laget og tilbakekalt.
- En lesing av noens avspilt- og favorittmerker som kom tilbake tom eller uleselig, fjernet alle sammen til neste lesing; den blir nå avvist, slik en tom lesing av et bibliotek blir.
- En episode med et sluttnummer ingen episode kan ha (en dato skrevet der) kunne holde Bibliotekhelse opptatt uten ende etter en lesing av biblioteket.

### Fixed
- Et punkt i endringsloggen med kode inne i fet skrift (i 2.0.2 og 2.0.3) viste backtick-tegnene i stedet for å tegne koden.
- Et diagram over antall kunne nummerere aksen 0, 3, 5, 8, 10, eller til og med 0, 0, 1, 1, 1: steg på 2,5 skrevet avrundet. Antall stiger nå i hele steg.
- En kalendernøkkel laget av noen uten tillatelsen Administrere fikk en lenke til nettleserens egen adresse i stedet for den offentlige adressen til FinStats, så den virket bare innenfra; siden ba også om innstillinger den ble nektet.
- En persons siste avspilling kunne være noen sekunder for kort til å telle som en avspilling andre steder, og dermed nevne en tittel personens historikk ikke viser.
- En gjenoppretting fra Nylig slettet som mislyktes, sa ingenting og lot knappen fortsette å snurre; det samme gjorde en mislykket *Hent tilbake* i Bibliotekhelse og en mislykket *Gjenåpne* eller *Merk alle som løst* i Sikkerhet.
- Kalenderabonnementet heter FinStats i en kalenderapp, ikke finstats.
- Nedlastingskøen viste hvem som hadde bedt om hver nedlasting, til alle som fikk se den; andres forespørsler krever nå *Se alle*, som i På vei.
- Når noen mistet tillatelsen *Varsle*, fikk mottakerne deres fortsatt varsler, og de kunne ikke lenger slå dem av.
- En avspilling som ble satt på pause og gjenopptatt innenfor sammenslåingsvinduet, ble varslet som stoppet én gang, mens gjenopptakelsen og den faktiske slutten aldri ble varslet.
- Avspillingene av et spor kunne bli flyttet over til en annen artists spor med samme navn («Intro») når albumet forsvant fra biblioteket.
- En varselmottaker skrevet som en IPv4-adresse inne i IPv6 (NAT64, 6to4) ble godtatt som offentlig selv om den nådde det lokale nettverket.
- En Sonarr- eller Radarr-tilkobling som ble pekt mot en annen server, fortsatte å vise plakatene fra den gamle serveren i en uke.
- Varslene i Sikkerhet sa at ingenting trengte en titt når to eller flere personer var valgt, og en side som ble tom da det siste varselet ble løst, hadde ingen vei tilbake.
- Alle API-nøkler som ikke hadde utløpt, sa at de utløp «akkurat nå».
- En dialog ble liggende over siden etter Tilbake, og siden kunne ikke rulles før den var lukket.
- Revisjonsloggen viste åtte typer oppføringer (en avspilling eller sikkerhetskopi hentet tilbake, papirkurven tømt, offentlige profiler, et endret utseende) med ID-en sin og ingenting under Detaljer.
- Å legge en tittel til i Min liste fra hurtigmenyen feilet uten et ord når serveren sa nei.
- *Sett sammen* på Oversikt behandlet flere personer valgt samtidig som én person.
- Tilbakeblikket skrev timer og dager under ti med nettleserens desimalkomma ved siden av sine egne tusenskillekommaer.
- Kapasitet sa at en disk ville bli full "i tempoet fra de siste 0 månedene".
- På siden Sikkerhet telte adressene uten sted alles avspillinger gjennom all tid, uansett hvilken person og periode siden var avgrenset til.
- Noen med *Administrere*, men ikke *Se alle*, kunne legge hvem som helst sin avspilling i papirkurven.
- Et kort fra Tilbakeblikk eller en offentlig profil kunne komme fra nettleserens hurtigbuffer på språket som var valgt før.
- Nedlastingene direktekanalen sendte, var på britisk engelsk, uansett hvilket språk siden var på.
- En pålogging som endret hva noen får se, nådde de andre åpne fanene deres først ved neste kontroll etter et halvt minutt.
- En fane som ga fra seg direktetilkoblingen til en nyere, tok en tilbake på neste side, så fanene til én person fortsatte å stenge hverandres.
- Tabeller sorterte størrelser fra 1 000 til 1 023 enheter som tekst på språk som grupperer sifre med mellomrom, og prosenter på tyrkisk likeså.
- En tittel med en plassholder i navnet, som "Big {device} Bunny", ble fylt ut av serverens meldinger.
- Tabeller sorterte varigheter og antall av en helhet (5 av 26) som tekst på alle språk unntatt engelsk og de nordiske.
- En fane som fortsatt var åpen på ett språk, fikk svar fra serveren på et annet så snart en annen fane hadde valgt det.
- Et ord ved siden av et tall fikk feil form på polsk, russisk og ukrainsk ("5 wieczory", "2 потоков").
- Kapasitet regnet den første fyllingen av et bibliotek som vekst, og delte det hvert bibliotek la til på seks måneder, uansett hvor lenge det var målt over; en fil som tok plassen til en annen (en oppgradering fra Radarr eller Sonarr, eller en fil Jellyfin ga ny id) talte som helt ny, når bare forskjellen var ny.
- Folk som så sammen med SyncPlay ble ikke en gruppe når én ble med noen minutter etter at den andre hadde startet; avspillinger som slutter samtidig på samme sted, hører sammen nå, og historikken grupperes på nytt én gang.
- Noen uten *Se alle* kunne se av *Aldri sett* for et bibliotek at en annen hadde spilt av en tittel; nå teller den bare deres egen visning.

## [2.2.0] - 2026-10-01 to 2026-10-07

Bibliotekhelse og FinUI

### Added
- **Din egen meny på datamaskinen**: Innstillinger → Utseende tilbyr åtte: sidepanelet slik det var, ett gruppert etter hva sidene er til, en skinne med ikoner som åpner seg til sidepanelet under pekeren, et sidepanel som begynner med søk, en dokk, en kommandolinje, sider du fester øverst, og fargede fliser. Sidepanelene kan stå til høyre. Dokken glir bort mens du ruller nedover, og kommer tilbake når du ruller oppover eller peker mot bunnen av vinduet; med dokken flytter statuslinjen seg til toppen.
- **Din egen meny på telefonen**: en fanelinje med Mer (den nye standarden), et ark som åpner seg halvveis, hele skjermen, sidene spredt som en vifte rundt tommelen, eller en adresselinje som vokser til menyen. Hver av dem søker i sidene sine og sender andre ord videre til søket.
- Begge huskes av nettleseren de ble valgt i, så telefonen og datamaskinen har hvert sitt valg; begge er FinUI-komponenter, `desktop-nav` og `mobile-nav`.
- **En hurtigmeny**: høyreklikk på en tittel, en person eller et bibliotek hvor som helst i FinStats (en plakat på Oversikt, en rad i en liste, et navn i en tabell, en film i Kommer snart som ikke er i biblioteket ennå), hold fingeren på den eller trykk på menytasten, så vokser en meny ut fra punktet du trykket på: åpne, åpne i ny fane, legg en film eller serie til i Min liste, åpne i Jellyfin eller på TMDB og TVDB, kopier lenken. Shift med høyreklikk gir fortsatt nettleserens egen meny. FinUIs `context-menu`.
- **Søk uten dialog**: søket vokser ut av menyen din (et sidepanel utvider seg til det, dokken reiser seg til det, navnet i kommandolinjen blir til det, menyen på telefonen rommer det), eller det tar sidens plass mens menyen blir stående, om du velger det i Innstillinger → Utseende. Ctrl+Space åpner det som før; vinduet over alt annet er borte. FinUIs `search`.
- **Alt i et bibliotek**: kortet Nylig lagt til i et bibliotek tilbyr *Alt i det*: alle filmer, serier eller album det har, som plakater, etter navn, år, dato lagt til eller størrelse, avgrenset mens du skriver. Flere kommer til mens du ruller.
- **Server → Bibliotekhelse**: FinStats sammenligner hver fil med naboene og lister det som skiller seg ut, hvert funn med tallene bak. Den rapporterer bare; ingenting blir endret, slettet eller skannet på nytt i Jellyfin.
- **Hull**: episoder som mangler mellom to filer i en sesong («Sesong 2: episode 4 og 5 mangler mellom 3 og 6»). En fil som inneholder to episoder, teller som begge; spesialepisoder og slutten av en sesong holdes utenfor.
- **Sesonger som skiller seg ut** fra resten av serien i oppløsning, HDR eller kodek, og **avvikende episoder**: én fil som er ulik resten av sesongen, for eksempel en SD-episode i en 1080p-sesong. Oppløsning går etter bredde eller høyde, så en beskåret film i 1920×800 er 1080p.
- **Kopier og versjoner**: samme film eller episode to ganger, etter leverandør-ID-ene Min liste og På vei allerede går etter. To kopier i samme oppløsning viser plassen som brukes to ganger; en 4K-kopi ved siden av en 1080p-kopi listes som to versjoner og regnes aldri som sløsing.
- **Tynne filer**: en bitrate langt under det oppløsningen og kodeken vanligvis krever («4K HEVC på 1,1 Mbps»); de tynne episodene i en sesong er ett funn.
- **Dubbing som stopper**: et lydspråk som noen sesonger har på hver episode og andre på ingen.
- **Aldri identifisert**: filmer og serier Jellyfin har matchet uten TMDB-, TVDB- eller IMDb-ID.
- **Avvis** et funn som er slik med vilje, med et notat. Det kommer tilbake av seg selv når filene endres. De som kan administrere FinStats og se serveren, kan avvise; avvisninger står i revisjonsloggen og er med i en sikkerhetskopi.
- Siden til et bibliotek sier hvor mange ting det er å se på der, og leder til dem.
- **FinUI**, komponentene FinStats er bygd av, er et eget prosjekt (`github.com/finstats/finui`): hver knapp, hvert kort og hver kontroll i alle tilstander og begge temaer på `finui.finstats.no`, og **FinUI create** for å endre farger, hjørneradius, tetthet og mer mens du ser på, og ta det med hjem med én kommando. **Innstillinger → System** leder dit.
- **Innstillinger → Utseende**: alle velger selv hvordan FinStats ser ut for dem, med koden til et FinUI-forhåndsvalg: grunnfarge, aksentfarge, diagramfarger, kontrast, hjørneradius, tetthet, kantlinjer, kort, knapper, felt, tabeller, utheving, bevegelse, ikoner, menyen, bakgrunnen bak det hele, fokusringen, hvordan overskrifter settes, og skriftene for tekst, overskrifter og tall. Valget ditt gjelder bare deg; siden viser hva koden betyr, og leder til FinUI create for å endre den.
- **Stiler**: elleve komplette utseender å starte fra med ett valg i FinUI create, og koden derfra tas inn i Innstillinger → Utseende. De er Washi (FinStats slik den leveres), Studio, Noir, Matinee, Console, Pastille, Ledger, Gazette, Arcade, Canopy og Harbour. FinStats selv leveres med ett utseende og tilbyr ingen liste over stiler.
- **Ikoner som beveger seg**: hvert ikon i en lenke, en knapp eller en fane gjør sin egen bevegelse én gang når du peker på det (oppdater snurrer, nedlastingspilen faller ned i skuffen, papirkurven løfter på lokket), og en oppdateringsknapp du har trykket på, fortsetter å snurre til svaret er kommet. De står stille med redusert bevegelse, og med Bevegelse: Av.
- **Bevegelse, med FinMotion**: komponentene i FinUI beveger seg nå: knotten på en bryter kastes over, streken under en fane siger over til neste fane, et felt som avvises, rister på hodet, en kopiering hakes av i ett strøk, meldinger legger seg som kort på hånden og vifter seg ut når du peker på dem, et ark på telefonen gir etter som en strikk når du drar i det, radene i en sortert tabell flytter seg på plass, et tall som endres, ruller som en kilometerteller, og diagrammer tegner seg selv første gang de vises. Alt følger valget for Bevegelse i Innstillinger → Utseende og står stille med redusert bevegelse. FinMotion er et eget prosjekt (`github.com/finstats/finmotion`), lagt oppå FinUI.
- **Flere ikoner i FinUI**: 239 nå, opp fra 71, hvert med sin egen bevegelse (volum opp og ned, et hodesett og et dempet hodesett, en blyant som skriver, en bjelle som ringer, en telefon som vibrerer), og hvert av dem kan lastes ned fra galleriet til FinUI som SVG, stillestående eller animert.
- **Skrifter**: trettifire fritt lisensierte skriftfamilier å velge mellom (Geist, Roboto, Manrope, IBM Plex Sans, Montserrat, Lora, Playfair Display, Fira Code med flere), levert med programmet som Inter, aldri hentet fra andre steder, og lastet bare når de er valgt.
- **Flere grunnfarger og aksenter**: Nord, Solarized, Gruvbox, Catppuccin, Rosé Pine, Tokyo Night, Dracula, Everforest og Ocean; seksten nye aksenter, ti av dem med én farge om dagen og en annen om natten. Hver grunnfarge har nå sitt eget papir om dagen: gråtonene så like ut i det lyse temaet.
- **Sletting kan angres i 30 dager**: en slettet avspilling eller sikkerhetskopi venter i papirkurven, og **Angre** i meldingen som følger, henter den tilbake. Aktivitet og Innstillinger → Sikkerhetskopier viser hver sin liste over det som nylig er slettet, med når det forsvinner for godt, så lenge det finnes noe der. En avspilling i papirkurven er utenfor alle totaler, diagrammer, tilbakeblikk og profiler, og en import eller gjenoppretting henter den ikke inn igjen som ny; den som så den sammen med noen, regnes som alene til den kommer tilbake.

### Changed
- Dark Reader og lignende utvidelser lar FinStats' sider være i fred (`darkreader-lock`): FinStats tegner begge temaene selv, og en side som ble malt om, kjempet mot dem.
- På telefonen glir ikke menyen lenger inn fra venstre: fanelinjen står nederst med mindre en annen meny er valgt.
- FinStats leser biblioteket ditt én gang etter denne oppdateringen for å lære hvor filer som inneholder flere episoder, slutter.
- På siden til en tittel er **Legg til i Min liste** like høy som **Åpne i Jellyfin** ved siden av; den var en størrelse mindre.
- Filtrene for tidsrom og personer blir værende øverst på skjermen mens siden ruller, så du kan bytte fra 7 d til Alt nederst på en lang side; det samme gjør filtrene for forespørsler og kommende titler i På vei.
- Knappen som går til toppen, holder seg unna den menyen som har bunnen av vinduet: lavt i hjørnet ved siden av dokken, rett over fanelinjen, tommelknappen eller adresselinjen på telefonen, og ellers over statuslinjen. Det er FinUIs `to-top`.
- Å slette en avspilling eller sikkerhetskopi flytter den til papirkurven i stedet for å fjerne den med en gang (`PUT` med `{"deleted": true}`; `DELETE` er borte fra API-et). Sikkerhetskopier utover antallet som beholdes, fjernes fortsatt for godt.
- **Endringslogg** er ett ark i stedet for en stabel med kort: hvert punkt er en egen linje med typen sin (Nytt, Endret, Rettet og resten) og typens ikon i en kolonne foran, under en overskrift for hver serie. Listen over versjoner står fortsatt til høyre.

### Performance
- Når FinStats åpnes, hentes bare det som er endret: et stilark, et skript eller en skrift nettleseren allerede har, bekreftes med noen få byte i stedet for å sendes på nytt. Før hentet hvert besøk alle stilark og moduler i FinStats i sin helhet, selv om hver av dem hadde en tagg som skilte versjonene fra hverandre.

### Stability
- Gjenoppretting av en sikkerhetskopi leser innstillingene bare fra innstillingsraden: en fil fra en nyere FinStats som lagrer mer der, mistet innstillingene sine uten et ord.
- En åpen fane vokser ikke lenger for hver side som besøkes: hver side som ble forlatt, ble liggende i minnet bak den på skjermen (rundt 2 600 elementer per besøk), holdt fast av FinMotions liste over det den beveger og av rullelytteren til filterlinjen, til fanen ble lukket.
- En feilskrevet kommando (`finstats bakcup`), eller `restore` uten fil, oppretter eller oppgraderer ikke lenger en database før den sier at den ikke vet hva den skal gjøre.

### Fixed
- Det lille diagrammet i en flis renner ikke lenger utenfor flisen når tallet ved siden av er bredt («56 d 23 t»), eller når fire fliser deler en smalere rad: diagrammet viker, og tallet beholder bredden sin.
- I det lyse temaet ble en primærknapp fiolett under pekeren; nå beholder den seglets farge.
- I Innstillinger står en bryter på linje med navnet og hjelpeteksten til innstillingen; den hang øverst i raden.
- I en rangert liste uten noe under et navn, som Toppbrukere på Oversikt, sto navnet øverst i raden ved siden av et sentrert bilde; nå er det sentrert.
- En rad med statistikkfliser i en bredde mellom telefon og datamaskin (fire fliser, tre i bredden) lot den siste stå alene på en egen rad; fire fliser legges nå fire, to eller én i bredden, og tre legges tre eller én.
- Lesingen av serveren telte med en enhet ingen hadde logget inn på, og som den ikke tar vare på: oppgaven sa tre enheter der Server-siden listet to.
- Ett sekund, ett minutt, én time: varsler sa «med 1 sekunder imellom» og «om 1 minutter», og en Jellyfin-jobb med ett minutts intervall kjørte «hvert 1 minutter».
- `finstats --help` nevner alle variablene FinStats leser; `FINSTATS_PUBLIC_IP_URL`, `FINSTATS_ALLOW_LIBRARY_SHRINK` og `FINSTATS_SKIP_PREUPDATE_BACKUP` manglet.
- Oppsettsveiviseren sa om en adresse som ikke var en adresse, at den «ikke kunne koble til», i en setning som rant over i den neste; nå sier den at adressen ikke er en adresse.
- I Innstillinger → Sikkerhetskopier forsvant et spørsmål som ble stilt mens en sikkerhetskopi ble skrevet, da sikkerhetskopien var ferdig.
- Når menyen var en skinne som åpner seg, og søket ble lukket, vistes det åpne søket og den lukkede skinnen samtidig, og deretter ble skinnen stående åpen til pekeren kom og gikk. Esc gir nå tastaturfokuset tilbake dit det var da søket ble åpnet, og slipper fokuset et klikk ga.
- Når søket ble lukket på telefonen, forble siden dempet til søket var borte, og hoppet så til full lysstyrke i ett bilde; nå toner dempingen ut sammen med søket.
- Stegene for Tautulli-importen sa at sikkerhetskopien lastes ned etter **Backup Database**. Tautulli lagrer den på sin egen maskin: stegene sier nå hvor (mappen `backups`, `/config` med Docker), hvilken fil du skal ta, og at du må kopiere den over.
- Et søk som ble lukket før det var ferdig med å åpne seg, etterlot skinnen som åpner seg, uten animasjon: fra da av åpnet og lukket den seg momentant.
- Skinnen som åpner seg, ble stående åpen etter at en side var valgt fra den, til du klikket et annet sted, og i noen nettlesere, eller med zoom, tegnet den et rullefelt under ikonene. Nå åpner den seg bare for pekeren og tastaturet, og tegner ikke noe rullefelt når den er lukket.
- På siden Sikkerhet teller en adresse som databasen plasserer i et land, men ikke på kartet, nå med for det landet i stedet for blant adressene uten kjent sted.

## [2.1.2] - 2026-10-01

### Added
- **Import fra Tautulli**: ta med deg Plex-historikken din. Last opp databasesikkerhetskopien fra Tautulli (`.db`-filen, eller `.zip`-filen som inneholder den) under **Innstillinger → Import**; avspillingene får en egen plass i filteret *Registrert av* i Aktivitet.
- **Et koblingsbrett for å si hvem som er hvem**: Plex-brukere på den ene siden, Jellyfin-brukere på den andre. Dra en ledning fra hver Plex-bruker til den de er nå, eller klikk på den ene og så den andre, eller bruk tastaturet. Den som står uten ledning, blir ikke importert, og to Plex-kontoer kan gå inn i én person.
- Filmer og episoder matches mot biblioteket ditt etter navn (en film etter tittel og år, en episode etter serie og nummer), og en som ikke er på serveren din ennå, blir tatt vare på og koblet til når den kommer. En gjenopptatt visning er én avspilling; musikk importeres ikke.
- Sikkerhetskopien inneholder tilgangstokenene til Plex: FinStats leser dem aldri, og fjerner filen så snart du har importert eller begynner på nytt.
- **Ukoblede medier** under **Innstillinger → Data**: alle titler fra Tautulli, Jellystat eller Streamystats som ikke passer med noe i biblioteket ditt, med hvor de kom fra og hvor de mest sannsynlig allerede finnes. **Finn** kobler dem til ut fra de beste gjetningene eller et søk (en film kan kobles til som en spesialepisode av en serie), og FinStats husker valget, så den samme historikken kobler seg til av seg selv når den importeres igjen. Valgene er med i en sikkerhetskopi.

### Performance
- **Tilbakeblikket åpnes tre til fem ganger raskere**: årets mest sette skuespillere og regissører ble regnet ut ved å gå gjennom året på nytt én gang for hver person i det. Målt på en ekte historikk gikk et år fra 0,29 s til 0,09 s og de siste 12 månedene fra 0,53 s til 0,12 s.

### Fixed
- **Importert historikk finner titler som går under et annet navn**: en avspilling matches også mot tittelens originalnavn, så en serie Plex eller Tautulli kjente som 오징어 게임, kobles til Squid Game, og *Im Westen nichts Neues* til *All Quiet on the Western Front*. Originaltitlene i biblioteket ditt leses ved neste skanning.
- Navn sammenlignes slik søket sammenligner dem (store og små bokstaver, aksenter, tegnsetting, et årstall skrevet inn i navnet, en innledende «The»), og et utgivelsesår kan bomme med ett, så «Star Wars: Episode I - The Phantom Menace» finner «… – The Phantom Menace», og en Kingsman fra 2014 finner den fra 2015. En tittel som er kjent under et helt annet navn, blir fortsatt aldri gjettet.
- En mislykket import fra Streamystats eller gjenoppretting ble meldt i varslene med sitt interne navn.
- Knappen *Del året som kort* i tilbakeblikket og seerrangeringen ved siden av hadde ikke noe mellomrom mellom seg.

## [2.1.1] - 2026-10-01

### Changed
- **Min liste tilbys overalt der en tittel er**: på kortene i Kommer snart på Oversikt, slik som i På vei → Kommer, og på siden til en episode eller sesong, som legger serien i listen.
- På en plakat på Oversikt vises Min liste-knappen mens du peker på kortet eller når det med tastaturet, i stedet for å dekke hjørnet på hver plakat; på en berøringsskjerm er den alltid der.

### Stability
- Å lagre en Sonarr-, Radarr- eller Seerr-tilkobling mens noen hadde **Innstillinger → Utgående tilkoblinger** åpen, kunne få begge til å vente på hverandre for godt.
- **Innstillinger som ikke kan leses, stopper FinStats ved oppstart** og nevner innstillingen som har feil, i stedet for i det stille å tilbakestille alle innstillinger til standard.
- **To endringer i innstillingene samtidig** (for eksempel innstillingssiden og tidsplanen for en oppgave) opphever ikke lenger hverandre.
- En pause, en spoling eller et sporbytte som ble sett mens databasen var opptatt, skrives ved neste runde i stedet for å gå tapt.
- En lesing av biblioteket feiler ikke lenger på en Min liste med flere oppføringer som viser seg å være samme tittel.

### Fixed
- **En utskiftet fil er ikke «nå på serveren»**: når filen til en film på noens Min liste oppgraderes, får de ikke lenger beskjed om at den nettopp har kommet.
- Én mislykket lesing av programtilleggene, de planlagte jobbene eller lagringen i Jellyfin tømmer ikke lenger den listen på Server-siden frem til neste lesing.
- **En avspilling fra Streamystats stopper der den stoppet, på alle sider**: frafallskurven til en tittel og «fullført» for en episode gikk i stedet etter lengden og regnet den som anslått.

## [2.1.0] - 2026-09-29 to 2026-09-30

Min liste

### Added
- **Din egen liste**: legg en film eller serie til i Min liste fra siden dens, fra Kommer, fra Nylig lagt til eller fra søket (Ctrl+Enter), og finn den under **Min liste** på din egen profil.
- **Hver oppføring sier hvor den står nå**: på serveren, «5 av 26 episoder», forespurt, «Kommer fredag», ikke på serveren, forsvunnet fra biblioteket, eller sett. Det regnes ut hver gang du ser på listen, med den samme tolkningen av «sett» som fremdriften i seriene dine. Sette titler blir værende, i en egen gruppe, til du tar dem bort.
- **En tittel som ikke er på serveren din ennå**, kan legges til rett fra det Sonarr eller Radarr venter på. Oppføringen kobles til tittelen av seg selv når den kommer, og en omdøpt fil tar oppføringen med seg.
- **«På Min liste: nå tilgjengelig»**: et varsel til dine egne mottakere når noe du ventet på, kommer. Av til du krysser av for det.
- **Ingen andre ser listen din**, heller ikke en administrator, og varselet går bare til dine mottakere. Ingenting skrives til Jellyfin, Seerr, Sonarr eller Radarr.
- Min liste er med i en sikkerhetskopi, og en gjenoppretting slår listene sammen uten å doble noe.

### Changed
- Oppføringene i `/api/upcoming` har TVDB- og TMDB-ID-ene til tittelen, og `/api/auth/me` sier om serveren har Min liste.
## [2.0.4] - 2026-09-29

### Added
- **Planlagte oppgaver, slik Jellyfin gjør det**: hver jobb FinStats gjør av seg selv, har utløsere (daglig på et klokkeslett, ukentlig på en dag, med et intervall, ved oppstart eller etter Jellyfins biblioteksskanning), hver med en valgfri tidsgrense. Klikk på en oppgave for å se og endre tidsplanen; *Legg til utløser* åpner den samme dialogen som i Jellyfin.
- **Oppgaver er en egen seksjon** i Innstillinger, med siste kjøring for hver jobb i ord («Kjørte sist for 38 minutter siden og tok 6 minutter») og en Kjør-knapp; før lå den øverst i System.
- **Metadataendringer**: hver time leser FinStats det Jellyfin har lagret siden sist (navn, beskrivelser, sjangre, vurderinger, medvirkende, fildetaljer og bilder), så en endring i Jellyfin vises uten å vente på en biblioteksskanning.
- Sikkerhetskopi og geolokasjonsdatabasen kan også kjøres fra Oppgaver; Esc går fra tidsplanen for en oppgave tilbake til listen.

### Changed
- **Temaet er en bryter med tre stopp**, Lyst · Enhet · Mørkt, der knappen som vekslet mellom dem satt før: klikk på en tredjedel av den, dra knotten eller bruk piltastene. Knotten viser ikonet for valget, og et nytt ikon glir inn når valget endres.
- **Logg ut ligger under Innstillinger → Konto**, som viser hvem som er logget inn her; sidepanelet beholder temabryteren.
- **De gamle tidsplaninnstillingene er nå utløsere**: *Følg Jellyfins biblioteksskanning* med reserveintervallet sitt, *Ta sikkerhetskopi hver N. dag* og GeoIP-bryteren *Hold databasen oppdatert* ble tilsvarende utløsere på oppgavene sine, med verdiene du hadde. Innsamling, Sikkerhetskopier og Sikkerhet lenker til tidsplanen i stedet.
- En oppstart kjører ikke lenger alle jobbene på nytt: FinStats husker når hver av dem sist ble ferdig.
- En Jellyfin-jobb som ennå ikke har et anslag, viser «omtrent … igjen», med prikker som går i ring der tiden skal stå, i stedet for «ETA …».

### Performance
- **Sikkerhetskartet henger ikke lenger etter**: når du drar og zoomer, flyttes kartet som allerede er tegnet, og det tegnes på nytt én gang når bevegelsen er ferdig. Pulsen for det som spilles nå, får heller ikke lenger hele kartet til å tegnes på nytt 120 ganger i sekundet mens ingen rører det. Et drag gikk fra 34 til 60 bilder i sekundet på en treg datamaskin.

### Fixed
- **En plakat eller et portrett som byttes i Jellyfin, vises i FinStats**: bilder ble mellomlagret en uke på serveren og en uke til i nettleseren, uansett hva Jellyfin hadde fått siden. Nå mellomlagres de under Jellyfins bildetagg, så et nytt bilde er en ny fil, og nettleseren sjekker at kopien er gjeldende hver gang den viser et bilde. Et nytt portrett lagrer bare personen på nytt, så Metadataendringer leser også personene.
- **Server → Jobber viser et anslag så snart siden åpnes**: gjenværende tid ble bare målt ut fra avlesninger siden selv gjorde mens den var åpen, så hvert besøk startet på «ETA …» til siden hadde sett prosenten flytte seg. Nå tar FinStats tiden på en kjørende Jellyfin-jobb selv, hvert 10. sekund mens den kjører, og siden får anslaget servert. En jobb Jellyfin melder på 0 % hele veien (første fase av en biblioteksskanning), beholder prikkene til den beveger seg: ingenting gjettes.
- **Verktøytips på kartet vises ved prikken**: i Firefox kunne verktøytipset for prikken som spilles nå, havne langt unna, og der prikker overlapper (noen som spiller av hjemme), avhang det av én piksel hvilket verktøytips som ble vist. Nå nevner ett verktøytips alt under pekeren.
- **Statuslinjen lyser opp på én måte**: hver lenke (strømmene, Repo, versjonen) lyser opp som den samme lille boksen, og prikkene mellom elementene står midt i mellomrommene. Når pekeren lå over versjonen, lyste prikken og mellomrommet foran også opp.
- **En filbane tar hele bredden av kortet sitt**: på Fil-kortet til en tittel, og for Jellyfins mapper på Server-siden, ble en bane presset inn i én smal celle og rant nedover ni linjer ved siden av tom plass.
- En dialog som tones ut, sluker ikke lenger et klikk på siden under.

## [2.0.3] - 2026-09-28

### Security
- Server-siden ga ID-en til hver enhet til alle med *Se server*; ID-er krever *Se nettverk* i tillegg, som overalt ellers.
- To tillatelsesbrytere som ble slått før den første var lagret, sendte den samme utgangslisten to ganger, og den andre opphevet den første: en tillatelse som ble tatt bort, var fortsatt gitt, og en del av en offentlig profil som ble slått av, var fortsatt publisert, mens begge bryterne viste det som var valgt.

### Stability
- **Bare én import eller gjenoppretting om gangen**: to opplastinger sendt samtidig kom begge gjennom kontrollen, og en gjenoppretting så aldri etter importer; den andre ventet ut databasen og feilet etter flere minutters opplasting.
- **En adresse med en ødelagt `%`-koding åpner siden for ukjent adresse**, i stedet for å bli stående med en spinner for godt.
- **Et høyt sidetall i en liste gir en tom side**: tallet løp over og kom tilbake som første side igjen.

### Fixed
- **Tabeller sorterer tall riktig i en tysk eller norsk nettleser**: «1,204» ble lest som 1.204, og alt over en million som tekst.
- **Tabellvisningen til et diagram sorterer datoer, ukedager og posisjoner etter tid**, ikke etter bokstavene.
- **Nedlastingshistorikken viser hvor importer kom fra**: Sonarr og Radarr oppgir indekserer og protokoll bare ved hentingen, så de fordelingene var alltid tomme. En mislykket sesongpakke er én feil, ikke én per episode.
- **Når en avspilling som ble sett sammen, slettes**, finner FinStats gruppene for tittelen på nytt i stedet for å etterlate en «gruppe» på én.
- **Tid sammen for én person er aldri mer enn tiden de selv var med**: Sammen-siden regnet hele kvelden som deres.
- **En kveld i selskap over midnatt nyttårsaften** (eller kanten av «siste N dager») hører i sin helhet til perioden den startet i, i stedet for å bli to kvelder alene.
- **Importerte episodeavspillinger får navnet til episoden** der Jellystat bare tok vare på seriens.
- **Geolokasjon finner bydatabasen ved siden av land- og ASN-databasene** som `geoipupdate` skriver, i stedet for å slå seg selv av.
- **Stedet som spoles mest tilbake, følger filtrene på siden**, slik tellingene gjør.
- **«Året ditt er klart»** går bare til personer som har noe i tilbakeblikket sitt.
- **Å løse et dempet varsel på nytt holder paret dempet**; bare gjenåpning opphever en demping.
- **En utløpt API-nøkkel teller ikke lenger med blant de tjue.**
- **Øktgrensen slipper den eldste økten** selv når mange innlogginger kommer i samme sekund.
- **En hjemmeadresse som oppslaget fant, blir værende** når den samme fjernes fra listen du har skrevet inn.
- **Bilder i revisjonsloggen og nøkkellisten**, der det før bare var initialer.
- **Varsellisten til en administrator viser hvem en personlig mottaker tilhører**, i stedet for «Din» på alles; **Nylig sendt** viser alle mottakerne for den siste hendelsen.
- **Esc på en tittel eller en person forlater aldri FinStats**, og lukker bare dialogen som ligger øverst.
- **En side tegnes selv når en lagring avbryter forhåndshentingen den ventet på.**
- **Spilles nå viser en seer som spoler mens avspillingen står på pause**, der vedkommende er nå, ikke først når avspillingen fortsetter.
- **Når en tilkobling fjernes, oppdateres hvilke sider som finnes**, slik det skjer når en lagres.
- **Byggedatoen til geolokasjonsdatabasen beholder årstallet** på amerikansk engelsk.

## [2.0.2] - 2026-09-28

### Security
- Bak en proxy telte innloggingsgrensen den første `X-Forwarded-For`-adressen, som er den klienten selv sender: en ny oppdiktet adresse for hvert forsøk, og grensen slo aldri inn. Nå er det den siste, den proxyen skrev.
- *Ta med IP-adresser og steder* sendte adresser til en personlig mottaker der eieren mangler *Se nettverk*, adresser ingen side ville vist eieren. Andres mislykkede innlogginger krever nå også *Se server*, som på Sikkerhet-siden.

### Stability
- **En stans går aldri tapt**: når en lesing av biblioteket ble avvist i de første øyeblikkene etter oppstart, før serveren lyttet etter den, ble avvisningen notert og FinStats kjørte videre i stedet for å stoppe.
- **En Jellyfin som svarer i en form FinStats ikke kan lese, blir avvist**, ikke trodd: hver bruker, eller hver tittel i et bibliotek, som kom under nøkler FinStats ikke kjenner (`id` for `Id`), slapp gjennom vakten med full telling og ble så merket som fjernet.
- **Slutten på en avspilling går aldri tapt**: når databasen var opptatt lenger enn ventetiden (en import som holdt den), ble en avspilling som sluttet i det øyeblikket, stående som pågående til neste omstart. Nå skrives den ved neste runde.
- **Innstillinger en gjenoppretting ikke kan lese, blir ikke rørt**: en sikkerhetskopi med innstillinger som ikke lenger passet denne versjonen, nullstilte alle innstillinger ved gjenoppretting: sammenslåingsvindu, tillatelser, hjemmeadresser, tidsplan.
- **Et kontrolltegn i et navn eller en tittel ødelegger ikke lenger kortene**: profilkortet, hvert historiekort som viste den tittelen, og ZIP-filen med dem svarte med en feil.

### Fixed
- **Varsler stopper for den som er deaktivert eller slettet i Jellyfin**: de personlige mottakerne deres fikk fortsatt alt de gamle tillatelsene ga tilgang til.
- **Et varsel i kø sjekkes på nytt før det sendes**: en mottaker som var slått av, eller en eier som hadde mistet en tillatelse, fikk likevel det som ventet på et nytt forsøk.
- **En mottaker som alltid svarer `Retry-After`, blir gitt opp** etter fem forsøk som alle andre, i stedet for å bli prøvd på nytt i tretti dager.
- **Enhets-ID-er på en persons side** vises bare med *Se nettverk*, som overalt ellers.
- **Et publisert år venter et døgn med alt**: de fullførte og droppede seriene fulgte hver avspilling, så å trykke play på en droppet serie endret den offentlige siden i løpet av sekunder.
- **En forespurt serie teller som sett** når en av episodene er det: alle forespurte serier ble lest som aldri sett, i Forespørsler-kapitlet og på Forespurt-kortet.
- **Sammen-kapitlet i tilbakeblikket utelater direkte-TV**, som resten av året: en kanal som sto på mens folk så sammen, kunne bli årets topptittel, og andelen kunne bli over 100 %.
- **Dager-kortet for «de siste 12 månedene» slutter på den nyeste dagen**: rutenettet mistet de siste tre–fire ukene.
- **Nye episoder av en serie kunngjøres én gang per kveld**: episoder på hver sin side av midnatt UTC ble kunngjort to ganger på en server øst eller vest for den.
- **Tilkobling av Seerr kunngjør ikke gamle forespørsler som klare til å ses**: en forespørsel som allerede var tilgjengelig ved første lesing, uten noe annet spor av når, ble lest som om den kom i det øyeblikket.
- **Titlene i et bibliotek som slettes i Jellyfin, forsvinner sammen med det**: de ble liggende i søk, på Nylig lagt til-hyllen og i totalene.
- **En remux forblir en direktestrøm** når en avlesning et øyeblikk mangler omkodingsdetaljene (ved en spoling), i stedet for å bli en omkoding resten av avspillingen.
- **«Jellyfin er nede» betyr at lesinger har feilet flere ganger på rad**: spredte tidsavbrudd med dagers mellomrom, på en server som sto stille på socketen, ble lagt sammen til det.
- **En Streamystats-import viser fremdriften på sitt eget kort**, ikke oppå det siste resultatet fra Jellystat-importen.
- **En adresse der vertsnavnet begynner med «web»** (`http://web:8096`) leses som et vertsnavn og kuttes ikke som om det var stien til Jellyfins nettklient.
- **En køfeil skjuler hele innloggingen** i en adresse også når passordet inneholder en skråstrek.

## [2.0.1] - 2026-09-28

### Changed
- **Språkene til en tittel ligger ikke lenger i veien**: toppfeltet nevner de tre første lyd- og undertekstspråkene, én linje hver, og et nytt **Språk**-kort viser alle: de komplette samlet, de delvise med hvor mange episoder som har dem. To dusin undertekstspråk på en lang serie dyttet før bakgrunnsbildet en hel skjermhøyde ned.

### Fixed
- **Plakaten til en tittel blir stående ved siden av navnet**: når teksten ved siden av ble lengre enn plakaten (et langt sammendrag, mange studioer), sank plakaten til bunnen av toppfeltet, under tomt bakgrunnsbilde.
- **En versjon FinStats ikke kan lese, kalles nettopp det**: en `app_version` som var endret for hånd til noe uleselig, fikk FinStats til å nekte å starte som «den eldre FinStats 2.0.1», noe den ikke var. Den nekter fortsatt, og sier nå hvilken verdi den ikke kunne lese og hvordan den skal skrives.

## [2.0.0] - 2026-09-25 to 2026-09-28

Tilbakeblikk 2026, offentlige profiler og Sammen

### Added
- **Hvem du så sammen med**, i tilbakeblikket: kvelder i selskap, tiden du brukte der, tittelen som samlet folk mest, og opptil tre følgesvenner, navngitt i appen og lenket til profilene deres.
- **Serier sett til ende**, i tilbakeblikket: den siste episoden av hver serie du så ferdig det året, etter samme forståelse av «sett» som fremdriftslinjene på profilen, og også dem du begynte på og la til side, under halvveis sett og uten noe avspilt på to måneder.
- **Det du ba om**, i tilbakeblikket: Seerr-forespørsler fra året, hvor mange som kom og hvor mange du så etterpå. Bare der det finnes forespørsler.
- **Året mot året før**: timer, avspillinger og dager, og hvor mye mer eller mindre.
- **Året som en historie**: hvert kapittel som et kort på 1080×1920, formatet historier på mobilen bruker (året, tallene, toppseriene, -filmene og -musikken, sjangre, personligheten, timer og dager, hver dag i året, rekorder, selskap, fullførte serier, forespørsler og året før). Lagre ett, eller alle som en ZIP, fra «Del året som kort».
- **Et kort nevner aldri noen andre**: kortene tegnes fra en kopi av året uten felt for en annen person, en rangering eller en app. I appen har et kort ikke noe navn i det hele tatt.
- **Serverens år**, for Jellyfin-administratorer: alle titler og totaler og husets personlighet, uten at noen rangeres eller nevnes. En bryter på tilbakeblikksiden, `scope=server` i API-et.
- **«Tilbakeblikket på året ditt er klart»**: et varsel i desember, én gang til hver person som så noe det året.
- **Offentlige profiler**: en person kan publisere deler av profilen sin på en lenke alle kan åpne uten konto (totaler og topptitler, rekker og når de ser, hele året som historiekort, de ti siste avspillingene), og hver del er av til eieren slår den på under Innstillinger → Offentlig profil. Enheter, adresser, apper, avspillingsmåter, filbaner og andre personer publiseres aldri, uansett hva som er slått på, og navnet som vises, er et eieren skriver inn, aldri Jellyfin-brukernavnet.
- **Ingenting på en publisert side er nyere enn et døgn**, verken nylige avspillinger eller noen total, så en side noen følger med på, røper aldri hvem som ser noe akkurat nå.
- **Kort som kan deles**: hver publiserte profil har et bilde på 1200×630, av profilen og av året, som Discord, Signal, Mastodon og resten viser når lenken limes inn, og som kan lastes ned og legges ut. Tegnes på serveren ut fra det siden viser, og ikke noe mer.
- **En lenke du kan trekke tilbake**: 22 tilfeldige tegn, nullstilt med ett klikk; den gamle lenken svarer da nøyaktig som en som aldri har eksistert.
- **Offentlige profiler er av til en administrator tillater dem**: én bryter for hele serveren, en liste over hvem som publiserer hva, og en måte å ta ned en profil på. Publisering, endringer, nullstillinger og nedtakinger står i revisjonsloggen, og en nøkkel kan ikke gjøre noe offentlig.
- **Sammen**, en egen side: hvem som ser med hvem, hvor mye, og hvordan det har endret seg. Tid sammen, kvelder og andelen av visningstiden i selskap, hver mot perioden før; timer i selskap mot timer alene per dag eller uke; hvert par av personer med kveldene sine, tiden sammen, når de sist så noe og hva de ser mest; titlene som oftest ses sammen; de siste kveldene; og, for den som kan se alle, hver persons tid alene mot tiden i selskap.
- Hvert par teller: en kveld med tre teller for hvert av de tre parene, der kortet før bare telte nøyaktig den samme gruppen personer. Tiden et par har sammen, er det korteste av de to oppholdene.
- Kortet **Sett sammen** på oversikten og på en profil lenker til siden, og kortet på en profil viser hvor stor andel av personens visningstid som var i selskap.
- `GET /api/stats/groups` får `pairs`, `series`, `people`, `previous`, `bucket`, og `watch_s` og `share` i totalene; nøklene som fantes fra før, er uendret.
- **Hvor folk stopper**, på hver film- og episodeside: andelen avspillinger som fortsatt ser ved hvert minutt, fra alle ved start til dem som nådde slutten, med tilbakespolingene og undertekstene som ble slått på i disse avspillingene, markert på samme akse. Kortet viser hvor mange stopp som er målt (avspillinger FinStats registrerte) og hvor mange som er anslått (importerte avspillinger, ut fra hvor lenge de varte), og trenger tre avspillinger før det tegner noe.
- **Hvem fortsetter å se**, på en serieside: hvor mange personer som startet hver episode, i rekkefølge, og en **Fullført**-kolonne per episode. «Alle gir seg etter episode tre» er folk som aldri trykker play på episode fire.
- **Filer som aldri spilles av**, under Avspilling: en film eller episode som er startet tre ganger eller mer og aldri har kommet forbi tretti sekunder, med hvem som prøvde og i hvilke apper. Jellyfins logg inneholder ingen avspillingsfeil, så avspillingene er vitnet; minste avspillingslengde gjelder ikke her, siden det er nettopp den disse avspillingene aldri når.
- **Mest tilbakespolt** og **Undertekster slått på**, under Avspilling: titler rangert etter tilbakespolinger per avspilling, med minuttet de samler seg rundt, og etter andelen avspillinger der undertekster ble slått på innen de første ti minuttene. Alle tre listene krever *Se alle*. Spolinger kortere enn 20 sekunder ble aldri registrert, så en kort repetisjon av én replikk er usynlig med vilje.
- En spoling husker hvor den kom fra, som et tall ved siden av etiketten (`from_s` i tidslinjen til en avspilling). Hver spoling som allerede er registrert, får tallet sitt fra etiketten, og det samme gjør en gjenopprettet sikkerhetskopi fra før dette.
- `GET /api/stats/files` for de tre listene; `GET /api/items/{id}` får `insights`, og episodene til en serie får `users` og `finished`.
- **API-nøkler**: alle som er logget inn, kan lage nøkler til seg selv under Innstillinger → API-nøkler, til et skript, et dashbord eller en telefon. En nøkkel sendes som `Authorization: Bearer fs_…`, vises én gang, lagres hashet, kan ha en utløpsdato og trekkes tilbake med ett klikk; en administrator ser alles og kan trekke dem tilbake. Nøkkelen gjelder samme person med samme tillatelser som en innlogging, lest på nytt ved hvert kall, så den som mister retten til å logge inn, mister alle nøklene sine med en gang. En nøkkel kan verken lage eller trekke tilbake nøkler.
- **En kalenderfeed**: `GET /api/calendar.ics?key=…` er det Sonarr og Radarr har på vei, som en iCalendar en telefon kan abonnere på. En nøkkel med omfanget *kalender* åpner feeden og ikke noe annet; feeden leser nøkkelen fra adressen fordi en abonnert kalender ikke kan sende noen header, aldri fra informasjonskapselen, og den nevner ingen.
- **En revisjonslogg** under Server → Revisjonslogg, for Jellyfin-administratorer: hver innlogging og hvert mislykket forsøk med adressen, hver innstilling eller tillatelse som ble endret og til hva, hver nøkkel som ble laget, brukt eller trukket tilbake, hver tilkobling, mottaker, sikkerhetskopi og import, hver avspilling som ble slettet og hvert varsel som ble løst, med hvem, hvorfra, via hvilken nøkkel og om det lyktes. Beholdes i et år og er med i sikkerhetskopiene. `GET /api/audit` blar gjennom den.
- **Import fra Streamystats**, en egen seksjon under Innstillinger ved siden av Jellystat. Ta filen fra **Settings → Backup & Import → Download Backup** og slipp den inn: år med historikk kommer på et par sekunder, og ingenting du allerede har, telles på nytt. `finstats import-streamystats <file>` gjør det uten grensesnitt, `POST /api/import/streamystats` gjennom API-et; én import kjører om gangen, uansett type.
- **Brukte du begge sporingsverktøyene? Importer begge filene.** En avspilling gjenkjennes som en FinStats allerede har, på sporingsverktøyets egen ID for den, og ellers på at samme person ser samme element og at starten eller slutten ligger nær en avspilling i historikken. Dermed kan en Jellystat-eksport og en Streamystats-eksport av de samme kveldene importeres begge, i hvilken som helst rekkefølge, og ingen av dem importeres over en kveld FinStats registrerte selv. «Nær» er Sammenslåingsvindu under Innstillinger → Innsamling.
- Importerte rader innrømmer det de ikke vet. En avspilling Streamystats registrerte selv, har ingenting om filen (ingen kodek, oppløsning, bithastighet eller container), så de feltene står tomme i stedet for å fylles med en gjetning, og kortene som ville vist dem, skjuler seg. En avspilling den hadde hentet fra Jellystat, har med hele økten, og den leses i stedet: datert etter øyeblikket den sluttet, det eneste øyeblikket noen av sporingsverktøyene tok vare på, og beskrevet som omkoding bare når økten faktisk var det.
- Rader Streamystats skrev fordi Jellyfin meldte et element som sett, for en visning den aldri så, telles og hoppes over: de er like lange som hele filmen og ville blitt visningstid en kveld ingen så på. Resultatet sier hvor mange det var.
- Hvordan Streamystats-data tolkes, felt for felt, lært fra en ekte eksport og ikke fra dokumentasjon: `docs/streamystats-import.md`.
- **Hvilket sporingsverktøy hver avspilling kom fra**, som et filter på Aktivitet: det FinStats registrerte selv, det som kom fra Jellystat, det som kom fra Streamystats. Bare de historikken din faktisk har, tilbys, og filteret er avgrenset som alle andre lister, så den som bare kan se sine egne avspillinger, bare får vite hvor sin egen historikk kom fra.
- **Filtrene på Aktivitet er nedtrekkslister med avkrysning**, så du kan be om filmer *og* episoder uten musikk, to avspillingsmåter eller to personer samtidig, i stedet for én av hver eller alt. Medietype har fått **Annet** nå som den kan kombineres.
- Flere personer samtidig, overalt der brukerfilteret finnes (oversikten, avspilling, biblioteker, sikkerhet og resten), og kartet og all statistikk følger med. Et brukerfilter snevrer fortsatt bare inn det noen har lov til å se: uten *Se alle* låses et kall til den som spør, uansett hvem og hvor mange adressen nevner.
- **Et lyst tema**: washi-papir, med blekktekst og sinoberrød aksentfarge. FinStats følger enhetens lyse eller mørke innstilling til du velger selv.
- **En temaknapp ved siden av Logg ut** veksler mellom Enhet, Lyst og Mørkt. Valget lagres i denne nettleseren og tas i bruk før siden tegnes, så det andre temaet blinker aldri forbi.
- **Åpne i Jellyfin**: hver film-, serie- og episodeside har en knapp som åpner tittelens side i Jellyfin, i en ny fane.
- **Jellyfins adresse for brukerne** under Innstillinger → Jellyfin: dit disse knappene peker, for når adressen FinStats kobler til, ikke er en nettleser kan nå. Tom beholder adressen FinStats kobler til; bare en Jellyfin-administrator kan endre den.

### Changed
- **FinStats har flyttet**: `github.com/finstats/finstats`, og imaget er `ghcr.io/finstats/finstats`. Bytt imagenavnet i `docker run` eller compose-filen for å fortsette å få oppdateringer; datamappen forblir som den er.
- **Innstillinger viser én seksjon om gangen.** Siden var fjorten kort i én kolonne og seks hundre ord med hjelpetekst. Nå har hver seksjon sin egen adresse (`/settings/collection`), en liste til venstre bytter mellom dem, og på en telefon brytes listen opp i brikker. Lenker til de gamle ankrene virker fortsatt.
- **En innstilling er én rad**: hva den er, én linje som sier regelen, og kontrollen til høyre. Tall i en seksjon lagres samlet med én knapp nederst; brytere lagrer seg fortsatt selv. Statuskortene (Oppgaver, Utgående tilkoblinger, Database) og lenken til lisensene ligger under **System**, og de to importverktøyene har en egen seksjon.
- **Finn en innstilling** fra feltet i toppen (`/` setter fokus på det) med et hvilket som helst ord i navnet; det viser hvilken seksjon innstillingen ligger i, og Enter tar deg til raden.
- **Innstillinger er for alle som er logget inn**: seksjonene som styrer serveren, er forbeholdt dem som har lov til å styre den, og en person som bare hadde tilgang til varsler, kommer endelig til sine egne mottakere.
- **Server-siden viser én seksjon om gangen**, slik Innstillinger gjør: Oversikt (system og lagring), Jobber, Enheter, Programtillegg, Logg og Revisjonslogg, hver på sin egen adresse. Lenker til `/server#jobs` virker fortsatt.
- **Serverloggen er en seksjon på Server-siden** (`/server/log`) i stedet for en egen side i menyen; `/events` sender videre dit med filtrene.
- Et varsel om en mislykket jobb, en mislykket sikkerhetskopi eller en tilkobling som sluttet å svare, lenker til innstillingsseksjonen det gjelder, i stedet for til toppen av siden.
- Det biblioteket ditt kan fortelle om en importert avspilling (hva slags innhold det var, hvilken episode, spilletiden, hvilket bibliotek den hører til), fylles inn etter **hver** lesing av biblioteket, ikke bare idet en import skjer. Historikk importeres som regel før FinStats i det hele tatt har lest biblioteket, så de avspillingene forble gjettet eller ukjente for godt.
- Gjenoppretting av en FinStats-sikkerhetskopi legger ikke lenger til en avspilling som kom fra et sporingsverktøy under en annen ID, og beholder fortsatt to ekte visninger av det samme med noen minutters mellomrom.
- Detaljene for en avspilling nevner sporingsverktøyet den kom fra, i stedet for å kalle alt som er importert, en Jellystat-avspilling.

### Performance
- **En omstart går nesten umiddelbart, uansett hvor lang historikken er.** Hver oppstart regnet ut hvem som så sammen over hele historikken og avgjorde på nytt hvilke avspillinger som var lokale, begge deler uten å endre noe: 24 s og 5,5 s på ti millioner avspillinger. Nå gjør en oppstart bare det den forrige kjøringen etterlot seg, og alt sammen på nytt bare etter en oppdatering eller når innstillingen bak er endret.
- **Mange som starter samtidig, registreres på sekunder.** Hver nye avspilling, hver lagring av fremdrift og hver enhet ble skrevet for seg, og hver avsluttet avspilling ble lukket med sin egen gjennomgang av hele historikken til tittelen; nå skriver en runde hver type samlet. 100 000 avspillinger som starter samtidig, tar 3 s, der 25 000 som sluttet samtidig, før tok over fire minutter.
- **Spilles nå forblir rask med tusenvis av seere.** For å finne hvem som ser sammen, ble hver strøm sammenlignet med hver annen, og hver følgesvenn listet opp ved navn: førti sekunder ved 10 000 strømmer, og gigabyte med minne i tillegg. Nå sammenlignes strømmer innenfor én tittel, og hver viser noen få navn og hvor mange andre.
- Utregningen av hvem som så sammen, skrev om hver gruppert avspilling i historikken hver gang FinStats startet, importerte eller så en avspilling slutte; nå skriver den bare det som er endret. Å finne hvilke avspillinger som var avsluttet, blir ikke lenger tregere med kvadratet av antallet som spilles.
- **Gjenkjenning av en avspilling historikken allerede har, går via en indeks**, så en import av 150 000 avspillinger tar sekunder, og ny kobling etter en lesing av biblioteket går fra tittel til tittel i stedet for å lese hver avspilling mens databasen holdes.
- **Innsikt i avspilling og oversikten er raskere over all tid**: de uavhengige delene regnes ut side om side, samtidighetsdiagrammet slår opp den lokale dagen én gang per kvarter i stedet for én gang per avspilling, og sjangre telles per tittel. På en million avspillinger: fra 4,1 s til 2,4 s, og fra 1,6 s til 1,1 s.

### Stability
- **Et TV-bibliotek leses ikke lenger som et bibliotek som er tømt.** Å lese et bibliotek tar to runder (hvert element, så en mindre runde for medvirkende i filmer og serier), og kontrollen som avgjør om en lesing er pålitelig nok til å fjerne det den ikke så, så på den andre. I et serie- eller musikkbibliotek er de to tallene langt fra hverandre, så kontrollen avviste hver lesing og stoppet FinStats, og installasjonen startet på nytt og gjorde det samme igjen. Ingenting ble noen gang feilaktig fjernet, men ingen lesing av biblioteket ble fullført. Hvis serie- eller musikkbiblioteket ditt ikke har fått med seg nye titler, er dette grunnen.
- En import kunne feile med «database is locked» når en annen skriving (en avspilling som ble oppdatert) kom mellom den første lesingen og den første skrivingen. Begge importverktøyene tar skrivelåsen når de begynner, og databasejournalen blir ikke lenger værende like stor som den største importen som noen gang er gjort.
- **En databasefil som er skadet, avkortet, tilhører noen andre eller er skrivebeskyttet, avvises med en gang**, med navnet på filen og beskjed om at ingenting ble endret, i stedet for 30 sekunder i stillhet; det skrives aldri til et annet programs database.
- En sikkerhetskopi eller et øyeblikksbilde før oppdatering som avbrytes halvveis, etterlater ikke lenger en fil som kunne ta plassen til et godt øyeblikksbilde blant de tre som beholdes.
- **Å overskrive stedsdatabasen mens FinStats kjører, får den ikke lenger til å krasje**: FinStats leser fra en privat kopi av filen.
- En Jellyfin-app som meldte en ny enhets-ID hver gang, kunne få minnebruken til FinStats til å vokse så lenge den kjørte; enheter som ikke er sett på fem minutter, glemmes nå.
- Én person har høyst 30 innloggede økter; den eldste logges ut ved neste innlogging.

### Fixed
- **Den samme kvelden telles ikke lenger to ganger når et element har fått nytt navn.** Når historikk ble koblet på nytt til en tittel som var lagt til igjen i Jellyfin, kunne en avspilling importert fra et sporingsverktøy bli et duplikat av en som allerede fantes, fordi kontrollen som ville fanget det, kjørte før ID-en ble flyttet. Nå kjører den igjen overalt der ID-er kobles på nytt. Bare importerte avspillinger fjernes noen gang, aldri en FinStats registrerte selv, og hvor mange som forsvant, står i loggen. Første oppstart av 2.0 går gjennom hele historikken én gang for å finne dem som allerede finnes.
- **En økt følger nå personen slik Jellyfin ser vedkommende nå.** En administrator Jellyfin fratok rettighetene, beholdt full tilgang til FinStats resten av den 30 dager lange økten, og den som var deaktivert eller slettet i Jellyfin, forble innlogget. Nå mister de tilgangen så snart FinStats leser brukerne fra Jellyfin (hvert 15. minutt, eller med en gang fra Innstillinger), slik API-nøkler alltid har gjort.
- Innlogging skriver det Jellyfin sier om personen inn i brukerlisten til FinStats, så en importert sikkerhetskopi fra et sporingsverktøy ikke lenger kan gjøre noen til administrator før den første lesingen av brukerne retter det.
- Den som er slettet i Jellyfin, merkes som fjernet også når to lesinger av brukerne faller i samme sekund.
- **Nedlastingsfeil viser aldri passordet til en nedlastingsklient eller API-nøkkelen til en indekserer**, som Sonarr og Radarr kan sitere i dem.
- Avspillinger som ble koblet på nytt til en omdøpt tittel under en lesing av biblioteket, ble ikke sjekket for felles visning før neste omstart; nå blir de det med en gang.

## [1.6.5] - 2026-09-24

### Added
- **En full sikkerhetskopi tas automatisk før hver oppgradering.** Første gang en nyere FinStats åpner databasen din, skriver den en fullstendig kopi av den (hele historikken, innstillingene og alt) til `data/pre-update-backups/` *før* den endrer noe. Hvis en oppgradering noen gang ødelegger noe, er dataene dine trygge: stopp FinStats, legg kopien tilbake i stedet for `finstats.db`, start versjonen du hadde, og meld fra om feilen. Ingenting går tapt. De nyeste kopiene beholdes; sett `FINSTATS_SKIP_PREUPDATE_BACKUP=1` for å slå det av hvis det er trangt om diskplassen.

## [1.6.4] - 2026-09-24

### Added
- **Fem nye steder å sende varsler til**: Telegram, Slack, Pushover, Pushbullet og e-post, ved siden av webhook-, Discord-, ntfy- og Gotify-mottakerne som fantes fra før. Hver ber bare om det den trenger (en chat-ID, en brukernøkkel, en postkasse), og de tre som hører til én tjeneste (Telegram, Pushover, Pushbullet), kan bare nås der, så et token aldri kan sendes til en vert som bare ligner. E-post går over TLS fra første byte eller `STARTTLS`, og det finnes ikke noe tredje alternativ.
- **Lisenser**, under **Innstillinger**: FinStats' egen GPL og hver tredjepartslisens programmet er bygget på (253 Rust-crates, de to medfølgende skriftene, kartomrissene og geolokasjonsdatabasen), hver tekst i sin helhet, slik prosjektet selv skrev den. Velg en komponent, så åpnes lisensen i vinduet ved siden av listen; en crate med flere filer bytter mellom dem på stedet. Listen lages fra de cratene FinStats faktisk bygges av, så en avhengighet kan ikke legges til uten at lisensen blir registrert.

## [1.6.3] - 2026-09-24

### Fixed
- Sikkerhetsvakten slår nå også inn når Jellyfin viser *ingen* biblioteker eller *ingen* brukere i det hele tatt (en ødelagt lesing, aldri en vanlig dag), noe justeringen i 1.6.2 hadde sluppet gjennom. Å miste noen biblioteker regnes fortsatt som vanlig; å miste alle på én gang gjør ikke det.

## [1.6.2] - 2026-09-24

### Fixed
- Vakten for dataintegritet som kom i 1.6.1, var for ivrig: et bibliotek som virkelig mistet de fleste elementene sine (for eksempel en håndfull klipp der filene forsvant), ble tatt for en ødelagt lesing, så FinStats nektet å oppdatere det og stoppet. Nå slår den bare inn ved en *åpenbart* ødelagt lesing: et bibliotek stort nok til å bety noe som kommer helt tomt tilbake, eller et stort bibliotek som er tømt til nesten ingenting. Vanlige endringer tas i bruk som før, og tilfellet «Jellyfin returnerte ingenting» fanges fortsatt.

## [1.6.1] - 2026-09-24

### Added
- **FinStats stoler ikke lenger på en lesing fra Jellyfin som ville slettet historikken din.** Hvis et bibliotek FinStats har tusenvis av elementer for, plutselig kommer tilbake tomt eller nesten tomt (en Jellyfin-oppgradering som endret API-et, en feil forkledd som et tomt resultat), beholder FinStats dataene den har, merker synkroniseringen som mislykket (og varsler), og stopper ryddig med en tydelig melding i stedet for å merke alt som fjernet. Start på nytt når Jellyfin er seg selv igjen, eller sett `FINSTATS_ALLOW_LIBRARY_SHRINK=1` hvis du faktisk tømte et bibliotek. Den samme vakten gjelder bibliotekslisten og brukerlisten.

### Security
- En `X-Forwarded-Host`-header sendt av en klient kunne gå god for en fremmed `Origin` og slippe en skriving på tvers av nettsteder forbi same-origin-vakten. Den headeren stoles nå bare på bak en klarert proxy (`FINSTATS_TRUST_PROXY`), samme regel som oppslaget av klientadressen allerede følger. Øktinformasjonskapselen med `SameSite=Lax` var, og forblir, første forsvarslinje.
- En personlig varselmottaker kunne pekes mot `0.0.0.0` eller `::`, som når denne maskinen. Kontrollen som holder personlige mottakere rettet mot offentlige adresser, regner nå disse som lokale, sammen med loopback og de private adresseområdene.

### Fixed
- Siden **Avspilling** feiler ikke lenger for et bibliotek med en avspilling uten registrert avspillingsmåte (en rad gjenopprettet fra en sikkerhetskopi eldre enn kolonnen); en slik avspilling teller ganske enkelt som direkteavspilling.
- Et svært langt ord i et søk eller et filter gir ikke lenger serverfeil: SQLite avviser et `LIKE`-mønster på over 50 000 tegn, så filterord kortes ned til en fornuftig lengde før spørringen.
- Siden **Innstillinger** ruller ikke lenger sidelengs på en telefon når et kort viser en hel setning, som resultatet av en gjenoppretting.

## [1.6.0] - 2026-09-24

Varsler

### Added
- Kortet **Varsler** under Innstillinger: legg til en mottaker, kryss av for hva den skal få høre om, og trykk Test. Uten mottakere blir ingenting kontaktet.
- Fire typer mottakere: en vanlig **webhook** (JSON, dokumentert i `docs/api.md`, valgfri `Authorization`-header), **Discord** (en embed via en kanals webhook-URL), **ntfy** (ntfy.sh eller din egen, til et emne) og **Gotify** (din egen server, med et applikasjonstoken). Titler sendes i selve meldingen og ikke i en header.
- Elleve typer hendelser, hver med sin egen avkrysning: umulig reise, en konto sett i et nytt land, en bølge av mislykkede innlogginger, en oppgave som feilet, en sikkerhetskopi som ikke kunne skrives, noe som sluttet å svare (Jellyfin, Sonarr, Radarr eller Seerr) og når det svarer igjen, nye titler i biblioteket (samlet per serie og dag, slik hyllen på oversikten samler dem), en forespørsel som ble klar til å ses, og en avspilling som begynner eller slutter. De to siste er av som standard.
- En forespørsel som ble klar til å ses, nevner tittelen, hvem som ba om den og hvor lenge de ventet.
- Personlige mottakere: gi **Varsle**, så kan personen legge til sin egen, som bare får det vedkommende allerede kan se: egne forespørsler og varsler alltid, andres med *Se alle*, adresser med *Se nettverk*. En personlig mottaker må peke mot en offentlig adresse; bare en administrator kan rette en inn i det lokale nettverket.
- **Ta med IP-adresser og steder**, per mottaker og av som standard: ellers nevner en melding om umulig reise bare steder («Oslo, Norge og London, Storbritannia»).
- **Nylig sendt**, i samme kort: hva som ble sagt, til hvilken mottaker, og hva den andre siden svarte når meldingen ikke kom frem. En mislykket melding prøves på nytt fire ganger i løpet av omtrent en time, og en mottaker som begrenser antall kall, får vente.
- **Adressen til FinStats**, en innstilling for administratorer, er det lenken i et varsel bygges fra. Tom betyr at meldingene ikke har noen lenke.
- Jellyfins egne planlagte oppgaver på Server-siden: hva hver av dem gjør, i ord i stedet for navnet i koden, og om den er en av de tunge. En oppgave som kjører, viser en fremdriftslinje, en prosent og hvor mye som gjenstår; resten viser når de kjører neste gang, når de sist kjørte og om det gikk bra. Oppgaver Jellyfin skjuler fra sitt eget kontrollpanel, er tatt med, og ingenting i FinStats kan starte eller stoppe en oppgave.
- Hvor mye som gjenstår av en oppgave, måles ut fra farten prosenten faktisk har beveget seg med, mot den nyeste avlesningen som ligger langt nok tilbake til å si noe, i stedet for snittet av hele kjøringen, så en oppgave som går fortere eller saktere, beskrives med tempoet den har nå.
- Oppgavelistene FinStats allerede leser av andre grunner (sjekken av biblioteksskanning hvert 5. minutt, serverdetaljene hvert 15.), mates inn i den samme målingen, så en oppgave som har kjørt i en time, har som regel et anslag i det siden åpnes. FinStats spør ikke Jellyfin om noe ekstra for dette.
- En kjørende oppgave uten noe målt ennå viser **ETA** og en ellipse som går i ring i stedet for et tall, og en oppgave der prosenten ikke har flyttet seg på et halvt minutt, viser «68 % i 4 min» ved siden av. Ingenting står i stedet for et anslag som ikke er målt.
- En liste over alle versjoner ved siden av endringsloggen: ett klikk går til hvilken som helst versjon, den markerer den du leser, og på en smal skjerm blir den en rad med versjonsserier over notatene.

### Changed
- Endringsloggen folder seg ikke lenger sammen. Hver versjonsserie står åpen, og versjonslisten er måten å komme seg rundt på, i stedet for én gruppe åpen om gangen og et klikk for å nå en annen.
- **Innstillinger → Utgående tilkoblinger** viser hver varselmottaker sammen med Jellyfin og tjenestetilkoblingene: hva den er, verten, om den er slått på, og når den sist tok imot en melding.
- Ingenting ved innsamlingen, historikken eller noe tall på skjermen endres. FinStats *leser* fortsatt bare fra Jellyfin, Sonarr, Radarr og Seerr.

## [1.5.2] - 2026-09-22

### Changed
- Å lese Jellyfins øktliste og å nullstille klokken som ba om lesingen er nå ett og samme kall, så de to kan ikke lenger komme ut av takt. Det var opphavet til feilen med løpske forespørsler som ble funnet mens 1.5.0 ble laget.
- Å bevise det tar nå en brøkdel av et sekund i stedet for sju minutter mot en ekte server, så det sjekkes hver gang noe endres. Samme forespørsler, samme historikk, samme tall på skjermen.

## [1.5.1] - 2026-09-22

### Fixed
- En omkoding registreres én gang, ikke én gang i sekundet. Når en avspilling starter med omkoding og går over til direkteavspilling, lar Jellyfin omkodingsdetaljene bli liggende på økten. Hver avlesning ble sammenlignet med en verdi FinStats nettopp hadde overskrevet, så alle avlesninger så ut som en endring, og Aktivitet-siden fyltes med «Omkoding»-linjer med «Direkteavspilling: …» under.
- Linjer som allerede er skrevet, ryddes opp ved oppstart: der flere etter hverandre sier det samme, beholdes den første. En avspilling som trengte omkoding, teller fortsatt som omkoding.

## [1.5.0] - 2026-09-22

Øktsporing over WebSocket

### Changed
- FinStats holder én tilkobling åpen til Jellyfin fra det øyeblikket det starter, og det er slik det får vite om en avspilling. I 1.4.0 kom dette som en innstilling, av som standard, for å bevise seg mot ekte servere; nå er det alltid på. Samme avspillinger, pauser og hopp, regnet ut av samme kode.
- Ingenting spilles: FinStats spør ikke Jellyfin om noe som helst. En kveld der ingen ser på, koster ingen forespørsler.
- Noe spilles: øktlisten leses én gang i sekundet. Det er det som gjør pauser, spoling og sporbytter presise, og det er det som avslutter en avspilling der klienten forsvant uten å si fra.
- Alt som er lastet, har stått på pause tre avlesninger på rad: spørringen stopper, og tilkoblingen tar over. Før kostet en film på pause det samme spørsmålet hvert sekund, rundt 16 MB i timen. Fortsetter noen, eller starter noen andre noe, fanges det opp innen omtrent et sekund, og tid på pause regnes fortsatt aldri som visningstid.
- Alle utgående lesinger ber om komprimerte svar og pakker dem ut (Jellyfin, Sonarr, Radarr og Seerr komprimerer alle når de blir bedt om det), og det tar mer enn halvparten av de to største lesingene: øktlisten under en avspilling og lesingen av biblioteket. Geolokasjonsdatabasen lastes fortsatt ned slik den er lagret.
- Jellyfin-kortet i Innstillinger viser hvilken av de to halvdelene som kjører, og direkteprikken følger om tilkoblingen bærer, ikke hvilken halvdel du er i.
- Bare én av de to kjører om gangen: mens FinStats spør, ber det Jellyfin slutte å sende, og ber den begynne igjen idet den siste avspillingen slutter. Tilkoblingen holdes åpen hele tiden.
- En Jellyfin som svarer på abonnementet med ingenting, blir trodd så lenge den svarer på keep-alive: FinStats sier fra én gang, leser øktlisten selv i mellomtiden og forblir abonnert til den første pushen avgjør saken. «Kan ikke pushe» og «holdt på å starte på nytt da vi spurte» ser like ut, så stillhet er aldri en dom.
- Reserveløsningen er uendret: en tilkobling som lukkes, blir stille eller viser seg ikke å støtte dette, faller tilbake til å spørre på tidtaker allerede ved neste runde og fortsetter å koble til igjen, med intervallene **Mens noen ser på, sjekk hvert** og **Mens ingenting spilles, sjekk hvert**.

### Added
- Hva innsamleren gjør, på `GET /api/status`: hvilken av de fire tilstandene den er i (`idle_socket`, `playing_poll`, `paused_socket`, `fallback`), om tilkoblingen er åpen og abonnert, hvor ofte den spør, hvor mange øktlesinger som gikk ut det siste minuttet, og når dette sist endret seg. Besvares fra minnet (ingen forespørsel, ingen spørring) og inneholder ingen navn, titler eller antall.
- Feltene kontrolleres mot hverandre ved hver runde, og et avvik som varer, skrives til loggen.
- Én begrenser under alle øktlesinger i prosessen: høyst 2 i sekundet og 70 i minuttet. Alt utover det blir utsatt, telt og logget høyst én gang i minuttet. Vanlig bruk holder seg godt under grensen.

### Fixed
- Direkteoppdateringer erstatter nå virkelig spørringene. Når 1.4.0 var slått på, ba den likevel om øktlisten hvert femte sekund og åpnet tilkoblingen på nytt hvert tjuende til førtiende sekund, fordi den bedømte tilkoblingen etter hvor nylig en liste hadde kommet, og Jellyfin sender en når noe *endres* og ingenting innimellom. Nå er det avgjørende om Jellyfin svarer i det hele tatt, sjekket med en keep-alive omtrent to ganger i minuttet.
- En øktliste som kom mens FinStats var mellom to runder, ble kastet, ut fra antakelsen om at en ny lå halvannet sekund bak. Den beholdes og brukes.
- **Sist sjekket** i Innstillinger og kortet Utgående tilkoblinger blir ikke lenger utdatert mens tilkoblingen er stille: det de viser, er Jellyfins svar på en keep-alive.

### Removed
- Innstillingen **La Jellyfin sende det som spilles** under Innstillinger → Innsamling, og `live_socket` fra innstillings-API-et.
- `socket_enabled` fra innsamlerens status: det finnes ikke lenger noe «slått av».
- `last_reconcile_at` fra innsamlerens status, erstattet av `socket_live`.

## [1.4.1] - 2026-09-22

### Changed
- Seerr leses bare i sin helhet når noe er endret: hvert femte minutt ber FinStats om én rad, den sist endrede forespørselen, og runden slutter der hvis den allerede kjenner den. Det er omtrent én kilobyte mot rundt seksti for en side med femti. Forespørsler som fortsatt er underveis, sjekkes hvert kvarter, alt listes én gang i døgnet, og en ny forespørsel dukker fortsatt opp innen fem minutter.
- Køene i Sonarr og Radarr leses hvert femte sekund mens en side viser dem, hvert minutt mens noe ligger i køen og hvert femte minutt mens den er tom, i stedet for hvert minutt hele døgnet. Å åpne siden, koble til en tjeneste eller en ny forespørsel i Seerr oppdaterer dem med en gang.

## [1.4.0] - 2026-09-21

Valgfrie WebSocket-økter, og alt FinStats kan nå

### Added
- **La Jellyfin sende det som spilles** under Innstillinger → Innsamling, av til du slår det på: én tilkobling bærer øktlisten, og rundt 17 000 forespørsler i døgnet blir nesten ingen. Avspillinger, pauser, hopp og grupper regnes ut av nøyaktig samme kode som før. Mens det er aktivt, gjør FinStats fortsatt én vanlig forespørsel i minuttet *mens noe spilles*, og ingen når ingenting gjør det. En tilkobling som lukkes, er stille i femten sekunder eller viser seg ikke å støtte det, faller tilbake til tidtakeren ved neste runde og fortsetter å koble til igjen.
- **Utgående tilkoblinger**, et kort i Innstillinger for Jellyfin-administratorer: alle mottakere FinStats kan nå (din Jellyfin, «hva er IP-adressen min»-tjenesten, databasen fra DB-IP og hver Sonarr, Radarr eller Seerr), med hva den brukes til, om den er slått på og når den sist svarte. Bygget av det FinStats allerede vet, og det viser bare adresser: aldri en nøkkel, aldri en grunnsti.
- **Slå opp nå** under Innstillinger → Hjemmenettverk, for den dagen den offentlige adressen din endres.

### Changed
- Oppslaget «hva er IP-adressen min» kjøres én gang, første gang FinStats trenger en adresse, i stedet for hvert kvarter.
- Jellyfin-kortet i Innstillinger viser hvordan innsamleren får beskjed, og hvis tilkoblingen er slått på uten å bære, hvorfor.

## [1.3.0] - 2026-09-20

Sonarr, Radarr og Seerr på én side

### Added
- **Tilkoblinger** under Innstillinger, for Jellyfin-administratorer: **Sonarr**, **Radarr** og **Seerr** (eller Jellyseerr/Overseerr), flere av hver, og hver testes før den lagres. Nedlastingsklienten din trenger ikke noe eget oppsett, for Sonarr og Radarr snakker allerede med den, og FinStats leser det de vet.
- **Forespørsler**: hvem som ba om hva, hvor lang tid det tok før det kom, og om de noen gang så det. Fliser for titler som venter og typisk ventetid, et diagram over hvordan det har endret seg måned for måned, og lister over det som kom for flere uker siden og aldri ble spilt av, og hvem som ber om mest.
- **Kommer**: nye episoder og filmutgivelser fra Sonarr og Radarr som en dagsplan, merket med om *du* ser på serien og, for dem som får se alle, hvem andre som gjør det. En serie regnes som sett når noen har spilt av en episode av den de siste fire månedene. En «Kommer snart»-rad på oversikten, én per profil for seriene den personen ser på, og hva som kommer for en tittel på dens egen side.
- **Nedlastinger**: køen i sanntid i hver Sonarr og Radarr (hva det er, hvor langt det har kommet, hvor fort det går, hva som gikk galt ved import), med personen som ba om det ved siden av. En sesongpakke er én linje uansett hvor mange episoder den har, og det som trenger oppmerksomhet, ligger øverst. Under den: hva som kom inn siste uke, måned eller år per indekserer, kvalitet og nedlastingsklient, og hvilke nedlastinger som feilet. Oversikten viser den samme listen, forkortet.
- **Se nedlastinger**, en ny tillatelse. Uten den ser folk fortsatt hvor langt deres *egen* forespørsel har kommet og hvor lang tid som gjenstår, og ingenting annet: ingen utgivelsesnavn, ingen hastigheter, ingen andre nedlastinger. Andres forespørsler krever *Se alle*.

### Changed
- FinStats sender `GET` til disse tjenestene og ingenting annet: det finnes ingen kode i det som kan godkjenne en forespørsel, starte et søk eller legge til, pause eller fjerne en nedlasting.
- API-nøklene deres lagres i FinStats' egen database, sendes aldri tilbake til nettleseren, skrives aldri til en logg og er aldri med i en sikkerhetskopi. Ingen omdirigering følges, så en nøkkel kan ikke havne et sted du ikke har skrevet inn, og sertifikater verifiseres med mindre du slår det av for én tilkobling.

## [1.2.2] - 2026-09-20

### Added
- Lyd- og undertekstspråk på hver film og episode («Lyd: japansk · engelsk»).
- Hvor langt en dubbing går: en serie eller sesong viser hvor mange av episodene som har hvert språk («Engelsk: 13 av 26 episoder»), og episodelisten har en **Lyd**-kolonne.
- **Lydspråk** og **Undertekstspråk** på biblioteksidene: hvor mange filer som kan spilles av på hvert språk.
- FinStats leser biblioteket på nytt én gang etter denne oppdateringen for å få med språkene. Det sier aldri «dubbet», fordi Jellyfin ikke oppgir en tittels originalspråk, og et spor uten språkmerke vises som «Ukjent».

## [1.2.1] - 2026-09-20

### Changed
- Bildene i README er tatt på nytt for denne versjonen, med et nytt et av **Sikkerhet**-siden. Ingenting i appen endres.

## [1.2.0] - 2026-09-20

Hvor folk ser fra, på et kart

### Added
- **Sikkerhet-siden**, for dem som får se nettverksdetaljer og alles aktivitet: et verdenskart med en prikk for hvert sted brukerne dine ser fra, større jo mer som skjer der. Hjemme er grønt, steder ute er lilla, direktestrømmer pulserer og mislykkede innlogginger utenfra er røde. Dra for å flytte, zoom med knappene, tastene eller Ctrl + hjul, og prikker som overlapper, slås sammen til du zoomer inn. Under kartet: hvert sted med personer, avspillinger, visningstid og innlogginger, og landene etter antall avspillinger.
- **Umulig reise**: et varsel når én konto sees på to steder ingen flyrute forbinder, eller på to steder langt fra hverandre samtidig, med begge observasjonene, avstanden, tiden mellom dem og farten det ville ha krevd. **Vis på kart** tegner reisen.
- **Nytt land**: første gang noen spiller av eller logger inn fra et land de ikke har vært sett i før.
- Løs et varsel med et notat, eller demp et par steder for den personen, for en VPN eller en mobil der operatøren holder til i hovedstaden. Varsler som finnes i gammel historikk, etter en import eller i en ny database, legges bort som løst i stedet for å oversvømme listen. Varslene og det du har bestemt om dem, er med i sikkerhetskopiene.
- Stedene kommer fra en bydatabase som leses lokalt: ingen adresse sendes noensinne noe sted, og kartet tegnes fra omriss som følger med FinStats, ikke fra en karttjeneste. Last ned DB-IPs gratis database fra Sikkerhet-siden eller **Innstillinger → Sikkerhet**, la FinStats oppdatere den hver måned (av til du slår det på), eller legg din egen `.mmdb` (DB-IP, MaxMind GeoLite2-City) i `data/geoip/`. `FINSTATS_GEOIP_DB` peker på en fil et annet sted.
- **Innstillinger → Sikkerhet**: databasen som brukes, den månedlige oppdateringen, og hvor fort (900 km/h) og hvor langt fra hverandre (500 km) to observasjoner må være for å regnes som umulig reise. Bydatabaser bommer ofte med noen hundre kilometer, og det er det avstanden er til for.

## [1.1.3] - 2026-09-20

### Added
- Ctrl+Space søker nå også i de medvirkende i hele biblioteket ditt, under **Medvirkende**, med bilde, om de spiller eller regisserer, og hvor mange av titlene dine de er med i; velger du én, åpnes siden deres. Like tilgivende som tittelsøket (ordene i hvilken som helst rekkefølge, aksenter ignoreres, en feiltasting tillates), og når flere treffer like godt, kommer den som er med i flest av titlene dine, først.

## [1.1.2] - 2026-09-20

### Added
- **Nylig lagt til** på oversikten, over Aktivitet-diagrammet: de 30 nyeste som en rad med plakater du kan rulle sidelengs, med piler på en datamaskin. Hver viser når den kom og hva den er (en film, et album eller «Sesong 4 · 3 episoder»), og én enkelt ny episode viser hvilken, med sesongens egen plakat når den har en.
- **Shift + musehjul** og piltastene flytter én plakat, Page Up og Page Down en skjermfull, Home og End går til hver sin ende, og sveiping virker. Det vanlige hjulet ruller fortsatt siden.
- Nye episoder av samme serie som kommer samme dag, deler én oppføring, og en hel serie som legges til på én gang, er én oppføring («3 sesonger · 60 episoder»), så én stor import ikke skyver alt annet ut av listen.
- Raden ser bort fra oversiktens tidsrom og person, siden den handler om biblioteket og ikke om avspillinger, og skjuler seg selv mens biblioteket er tomt.

## [1.1.1] - 2026-09-20

### Added
- Sider åpnes med en gang: et øyeblikk etter at FinStats har lastet, henter det i det stille det du sannsynligvis åpner neste gang (oversikten, profilen og tidslinjen din, Aktivitet, Brukere, Biblioteker og hvert bibliotek, Avspilling, serversidene, tilbakeblikket og de mest aktive personene), så en side vises umiddelbart når du åpner den, og oppdateres fortsatt i bakgrunnen.
- Filmer, serier og medvirkende hentes bare når du viser interesse, siden det finnes tusenvis av dem: å la pekeren hvile på en lenke, trykke på den eller nå den med tastaturet henter den ene siden.
- Det venter til siden du åpnet er ferdig lastet, tar én side om gangen mens nettleseren er ledig, tar pause i en bakgrunnsfane, gjør ingenting på en datasparende eller svært treg tilkobling, og glemmes når du logger ut.
- En liten **Repo**-lenke med GitHub-merket i statuslinjen, ved siden av versjonen.

### Fixed
- **Tidslinje** glemte hvilke biblioteker som var slått av når siden ble lastet på nytt eller åpnet fra et bokmerke, selv om valget sto i adressen.

## [1.1.0] - 2026-09-20

En tidslinje over alt du har sett

### Added
- **Tidslinje**, ved siden av **Oversikt** på hver profil: det du har sett som én sti fra i dag og tilbake til den første avspillingen FinStats kjenner til, med en kveld med episoder samlet i ett stopp. Hvert stopp er en plakat med hva som ble sett og når («Sesong 2 · Episode 3–6», en film og om den tok mer enn én økt, eller et album og hvor mange spor), og åpner tittelen.
- Stien følger vinduets bredde: tre stopp per rad på en bred skjerm, to på en smalere, annenhver rad går baklengs med en sving som binder den til den neste, helt ned til én rett linje på en mobil. Det første stoppet i hver måned viser måneden.
- Eldre stopp lastes av seg selv mens du ruller, helt tilbake til der historikken begynner.
- **Biblioteker**-bryterne over stien utelater det du ikke vil se, for eksempel musikk, og valget er en del av adressen.
- Noe som fortsatt spilles, viser **Spilles nå**.
- Du ser din egen tidslinje; andres krever tillatelsen «Se alle».

## [1.0.5] - 2026-09-20

### Added
- Beskyttelse mot nedgradering: databasen husker den nyeste FinStats-versjonen som har åpnet den, og en eldre versjon nekter å starte på den i stedet for å jobbe i stillhet med data den ikke forstår fullt ut. Meldingen sier hva du skal gjøre. Versjoner til og med 1.0.4 kom ut før denne sjekken fantes.

## [1.0.4] - 2026-09-20

### Fixed
- I detaljene for en avspilling hadde kopieringsknappen ved siden av en lang **Enhets-ID** falt ned på en egen linje.
- På Avspilling-siden ble den siste kolonnen i **Hvilke klienter koder om** kuttet («Omk…»); overskrifter på to ord brytes nå.

### Changed
- Skjermbildene i README er oppdatert igjen; de viste fortsatt versjon 0.5.0.

## [1.0.3] - 2026-09-20

### Added
- En episodeside viser episodens eget bilde, det Jellyfin viser i episodelisten sin, i stedet for seriens plakat. En episode Jellyfin ikke har bilde av, beholder plakaten.

## [1.0.2] - 2026-09-20

### Fixed
- I **Endringslogg** manglet gruppene v0.9 og v0.10 overskrift ved siden av versjonen. En gruppes overskrift er nå alltid den første setningen om hva den første versjonen handlet om, så en lang innledning ikke kuttes midt i en setning.

## [1.0.1] - 2026-09-20

### Fixed
- Det publiserte imaget ville ikke starte på en ny maskin: «unable to open database file: /data/finstats.db», etter å ha hengt i et halvt minutt. Docker oppretter en manglende `data`-mappe som root, og FinStats kjører som en vanlig bruker. Containeren gjør nå mappen til sin egen og bytter så til den brukeren, så `docker run` virker på første forsøk. For filer som eies av en annen enn bruker 1000, sett `PUID` og `PGID`; `--user` virker som før.
- En datamappe som virkelig ikke kan skrives til, meldes med en gang, med kommandoen som løser det, i stedet for 30 sekunder senere som en databasefeil.

## [1.0.0] - 2026-09-20

Klar for alle

### Added
- Et ferdig image, nå på `ghcr.io/finstats/finstats`, for 64-biters Intel/AMD- og ARM-maskiner (en Raspberry Pi 4 eller 5 fungerer): `:latest` er den nyeste versjonen, en versjonstagg blir stående der den er, og `:edge` er utviklingsversjonen.
- Hver versjon vises på prosjektets side for GitHub-utgivelser med denne endringsloggen.

### Changed
- Versjonsnumrene betyr nå det de sier: 1.x-oppdateringer ødelegger ikke dataene, sikkerhetskopiene eller innstillingene dine.
- Docker Compose-filen og alle instruksjoner bruker det publiserte imaget. Et selvbygget `finstats:latest` trenger bare at imagenavnet endres; `data`-mappen følger med uendret.

## [0.10.2] - 2026-09-20

### Fixed
- På en mobil kunne Innstillinger og siden til en serie rulle sidelengs: en etikett som bare var ment for skjermlesere, slapp ut av tabellen sin, og episodelistene og raden med medvirkende gjorde det samme. Nå ruller de inne i seg selv.
- Tilbakeblikket ruller ikke lenger sidelengs når et kapittels bakgrunnsord er bredere enn siden.
- En lang «ser på …»-linje under **Spilles nå** slutter med en ellipse i stedet for å kuttes midt i en bokstav.
- Plakaten foran hver serie på en profil var en ekstra lenke uten navn for skjermlesere; den er nå skjult for dem, og tittelen ved siden av er lenken.

## [0.10.1] - 2026-09-20

### Fixed
- Nedlasting av en sikkerhetskopi feilet i nettleseren («the source file could not be read» i Firefox) uten at noe på siden sa hvorfor: FinStats komprimerte en sikkerhetskopi, som allerede er komprimert, en gang til, og den dobbeltpakkede overføringen brøt av før slutten. Sikkerhetskopier sendes nå som de er, med størrelsen oppgitt på forhånd, så nettleseren kan vise reell fremdrift.

## [0.10.0] - 2026-09-20

Sikkerhetskopier, og flytting til en ny installasjon

### Added
- En sikkerhetskopi hver uke, de fem nyeste beholdes: hver avspilling med tidslinjen over pauser og hopp, merker for sett, tillatelser, hjemmeadresser, serverloggen og innstillingene dine, i én liten fil, noen hundre kB for tusenvis av avspillinger. **Innstillinger → Sikkerhetskopier** viser dem med en **Last ned**-knapp hver, lager en ved behov og bestemmer hvor ofte og hvor mange. De ligger i mappen `backups` i datamappen din. En sikkerhetskopi inneholder ingen API-nøkkel til Jellyfin og ingen innloggingsøkt, men den er en fullstendig visningshistorikk med IP-adresser, så oppbevar den et privat sted.
- Gjenoppretting slår sammen i stedet for å overskrive: avspillinger som allerede finnes, hoppes over, så det er trygt å gjøre to ganger, eller inn i en installasjon som har kjørt en stund. Fjern haken for «Gjenopprett også innstillinger og tillatelser» for å få bare historikken. Sikkerhetskopier virker på tvers av versjoner i begge retninger. Fra en terminal: `finstats backup` og `finstats restore <file>`.

### Changed
- Innsamlingen sjekker hvert sekund mens noen ser på og hvert femte sekund mens ingen gjør det, der den før sjekket hvert femte sekund hele tiden, så pauser, hopp og sporbytter registreres på sekundet. Begge ligger under **Innstillinger → Innsamling**; et endret gammelt intervall må settes på nytt.

### Fixed
- En IP-adresse i detaljene for en avspilling ble delt over to linjer midt i adressen. Merket Lokal/Utenfra flyttes ned under når det ikke er plass.

## [0.9.1] - 2026-09-20

### Fixed
- Folk hjemme ble vist som utenfra. En enhet på ditt eget nettverk som når Jellyfin via det offentlige navnet, for eksempel gjennom en omvendt proxy eller et domene, kommer med husstandens offentlige IP-adresse, og bare private adresser regnet som lokale. Nå lærer FinStats den offentlige adressen og regner de avspillingene som lokale i hele historikken, og det husker tidligere adresser.

### Added
- **Innstillinger → Hjemmenettverk**: adressene FinStats regner som hjemme, legg til dine egne (en tidligere adresse, et annet hjem, en VPN-utgang), eller slå av oppslaget. Oppslaget spør en enkel «hva er IP-adressen min»-tjeneste hvert kvarter og prøver flere (Amazon, Cloudflare og andre), fordi reklameblokkerende DNS som Pi-hole ofte blokkerer dem, og én av dem trenger ingen DNS i det hele tatt. Forespørselen inneholder ingenting om deg eller serveren din, og med bryteren av snakker FinStats ikke med noe annet enn Jellyfin. `FINSTATS_PUBLIC_IP_URL` angir en egen tjeneste.

## [0.9.0] - 2026-09-20

Alle tabeller, i den rekkefølgen du vil

### Added
- Alle tabeller kan sorteres: klikk på en overskrift for å sortere etter den, en gang til for å snu rekkefølgen, og en tredje gang for den opprinnelige. Innholdet i cellene forstås, så «3 d 2 t» sorteres som tid og «1,4 GB» som størrelse, og tomme celler havner sist. Det gjelder brukerlisten, seerne, episoder, enheter, programtillegg, planlagte oppgaver, tabellvisningen av alle diagrammer, tillatelsesmatrisen og stolpelistene (kodeker, oppløsninger, klienter og resten), som får en smal overskrift av sine egne.
- Aktivitet og serverloggen sorterer på tvers av alle sidene sine, ikke bare radene på skjermen, og rekkefølgen lagres i adressen, så den overlever en ny innlasting eller en delt lenke.
- Tabeller med ti rader eller mer får et lite filterfelt over seg.

## [0.8.2] - 2026-09-20

### Changed
- Årsvelgeren i tilbakeblikket er den samme bryteren som alle andre tidsromvelgere i FinStats (7 d · 30 d · 90 d …).

## [0.8.1] - 2026-09-19

### Added
- Sider for skuespillere og regissører, fra tilbakeblikkets «Mest sette personer» eller den nye **Medvirkende**-raden på enhver film, serie eller episode: alt de er med i på serveren din, hvor mye som er sett og av hvem, og hva som fortsatt venter. Siden følger tidsrommet du velger, personer som bare får se sin egen statistikk, ser bare det de selv har sett der, og Esc tar deg tilbake.

## [0.8.0] - 2026-09-19

Tilbakeblikket har fått nytt utseende, og mer å fortelle

### Added
- **Mest sette personer**: skuespillerne du tilbrakte mest tid med og regissørene bak det du så, som en rad med portretter. Medvirkende leses sammen med biblioteket og dukker opp etter den første lesingen av biblioteket etter denne oppdateringen.
- **Året ditt i dager**: hver dag i året som en rute, lysere jo mer du så, med den lengste rekken din og den største dagen over. Hold pekeren over en dag, eller gå gjennom kalenderen med piltastene, for å se hva den inneholdt.
- **I tall**: avspillinger, visningstid, ulike titler, dager du så noe, lengste rekke og hvor mye som var **gjensyn**, pluss hvordan året fordeler seg mellom episoder, filmer og musikk.
- Sjangerkapittelet viser hvor mange sjangre du var innom og hvor stor andel den største tok.

### Changed
- Et nytt design: det åpner med året ditt «spilt av på nytt», overskriften ved siden av plakatene som fylte det, over en bølgeform av året med én stolpe per uke der de høyeste lyser, og du velger år rett over. Hvert kapittel har en overskrift med nøkkelordet i fiolett, en setning som bærer tallene, og kapittelets ord stort og svakt bak. Det slutter slik filmer gjør, med rulletekst: i hovedrollene, regi, vist på, spilletid.
- Timer, ukedager og måneder deler ett kapittel, **Aktivitetsmønstre**, med en bryter mellom dem, og overskriften følger det du ser på («Fredag tok kronen»).

### Fixed
- **Spilles nå** på oversikten var blank når ingen så på, så overskriften så ut til å høre til filtrene under. Boksen «Ingenting spilles akkurat nå» er tilbake.

## [0.7.10] - 2026-09-19

### Added
- Bibliotekbilder: hvert bibliotekkort, og bibliotekets egen side, viser bildet Jellyfin bruker for det, via FinStats' bildeproxy som plakatene. Et bibliotek uten bilde beholder ikonet sitt.

## [0.7.9] - 2026-09-19

### Changed
- Sjangre-kortet åpner som **liste** igjen, med radaren som det andre valget på bryteren. Et valg du allerede har gjort, beholdes.

## [0.7.8] - 2026-09-19

### Added
- En radarvisning for Sjangre-kortet, på oversikten og på profiler: én eike per sjanger, med bytte mellom **Radar** og **Liste** i hjørnet av kortet, og valget huskes. Å holde pekeren over et punkt, eller bruke piltastene, viser visningstiden og andelen for den sjangeren.

## [0.7.7] - 2026-09-19

### Fixed
- Filmer som hører til en samling, ble vist som «Ikke lenger i biblioteket». Med *grupper filmer i samlinger* slått på gir Jellyfin ut samlingen i stedet for filmene i den, så FinStats så dem aldri; på én server skjulte det 556 av 1 238 elementer i filmbiblioteket. Nå ber FinStats om selve filmene, og lesingen denne oppdateringen utløser, henter dem tilbake og kobler avspillingene deres til dem igjen.

## [0.7.6] - 2026-09-19

### Changed
- På profiler er **Serier**-kortet flyttet ned til rett over Sjangre, så en lang liste med serier ikke lenger skyver visningstiden, aktivitetsdiagrammet og topptitlene ut av syne. Rekkene med dager blir værende øverst, og en serie du har åpnet, forblir åpen når du endrer tidsrommet.

## [0.7.5] - 2026-09-19

### Changed
- Søket matcher ord for ord, i hvilken som helst rekkefølge og med hva som helst imellom, i stedet for å kreve nøyaktig frase, så «alya hides» finner *Alya Sometimes Hides Her Feelings in Russian*. Store og små bokstaver, aksenter og tegnsetting spiller ingen rolle, en innledende «The» ignoreres, og en feiltasting tilgis («mentalsit» finner *The Mentalist*); ord på tre bokstaver eller færre må fortsatt skrives riktig. Bedre treff rangeres først, og musikk finnes også via artisten.
- Søkefeltene i Aktivitet og serverloggen matcher ord for ord på tvers av alle kolonnene: «alya opera» finner avspillinger av den serien i Opera.

## [0.7.4] - 2026-09-19

### Changed
- Endringsloggen er gruppert etter serie: hver `0.x`-serie er én sammenleggbar gruppe med versjonene sine, hva serien handlet om, hvor mange det er og når. Bare den nyeste starter åpen, og en lukket gruppe viser fortsatt hvilken versjon du kjører.

## [0.7.3] - 2026-09-19

### Fixed
- Klokken i Spilles nå nølte med noen sekunders mellomrom: oppdateringen hvert femte sekund tegnet den også på nytt, så når serveren lå en brøkdel foran, flyttet tallet seg for tidlig og sto så stille i et sekund. Nå er det bare tikket én gang i sekundet som flytter den, med nøyaktig ett sekund (målt 999 til 1001 ms mellom), og oppdateringen retter fortsatt opp etter en pause, et hopp eller reell avdrift, og treffer tikket.

## [0.7.2] - 2026-09-19

### Added
- Se sammen, nå i sanntid: hvert kort i Spilles nå viser «Med maria» mens folk ser det samme sammen, der en gruppe før bare ble gjenkjent etter at avspillingene var slutt. I sanntid regnes folk som sammen når de er på samme tittel og enten startet innenfor gruppevinduet eller er på nesten samme posisjon, noe som fortsatt virker hvis FinStats ble startet på nytt midt i en strøm.

## [0.7.1] - 2026-09-19

### Changed
- Spilles nå teller hvert sekund: klokken tikker og linjen glir jevnt imellom, i stedet for å hoppe hver gang serveren blir spurt. Serveren spørres fortsatt hvert femte sekund og er fortsatt fasiten: visningen synkroniseres på nytt ved en pause, et hopp eller når serveren ligger foran, men ikke for en posisjon som bare er noen sekunder gammel, siden klienter rapporterer bare omtrent hvert tiende sekund. Strømmer på pause står stille.

## [0.7.0] - 2026-09-19

Hvem som ser sammen

### Added
- Se sammen: når ulike personer starter samme tittel samtidig, regner FinStats det som at de ser sammen. Et **Sett sammen**-kort på oversikten viser gruppene, titlene de deler og timene deres sammen, hver profil viser hvem den personen oftest ser sammen med, delte avspillinger har et lite personmerke i Aktivitet, og detaljene for avspillingen viser hvem den ble sett sammen med. Det virker på hele historikken din, importerte avspillinger inkludert.
- En innstilling for hvor tett startene må ligge, 60 sekunder som standard: omtrent en tredjedel av ekte gruppeøkter starter 6 til 60 sekunder fra hverandre. Folk må også fortsette å se side om side i et par minutter, så to personer som tilfeldigvis åpner samme episode, er ikke en gruppe.

### Changed
- Søket åpnes med **Ctrl + Space** i stedet for Ctrl/⌘ + K.

## [0.6.1] - 2026-09-19

### Added
- Administratorer kan åpne en annen persons tilbakeblikk, øverst på Tilbakeblikk-siden eller med **Åpne tilbakeblikk** på profilen deres. Da omtaler det personen ved navn i stedet for «du».

### Changed
- Alle andre ser fortsatt bare sitt eget tilbakeblikk, uansett hvilke tillatelser de har, og det finnes fortsatt ikke noe tilbakeblikk for hele serveren.

## [0.6.0] - 2026-09-19

Profilen din, og hvor du er i hver serie

### Added
- Seriefremdrift på profiler: hver serie du har vært innom, som en stolpe med ett segment per episode (sett, påbegynt eller ikke ennå), som kan åpnes sesong for sesong, sortert etter hvor nær du er å bli ferdig, med egne visninger for fullførte serier og alt.
- **Merk som sett** på din egen profil, per episode, sesong eller serie, for det du så mens ingenting registrerte. Jellyfins egne merker for avspilt brukes også. Merkene blir i FinStats og endrer aldri noe i Jellyfin; en registrert avspilling kan ikke avmerkes, men dine egne merker kan.
- Rekker med dager på profiler: den lengste rekken din med dager på rad med en avspilling, den nåværende rekken og hvor mange dager du har sett noe.
- **Min profil** i sidepanelet.
- Aktivitet viser hvor avspillingen stoppet (for eksempel `44:15 / 45:00`) under fremdriftslinjen, og dato og klokkeslett under «for 3 t siden». Avspillinger importert fra Jellystat har ingen stopposisjon, fordi Jellystat aldri registrerte noen.

### Changed
- Episoder som mangler eller ikke er sendt ennå, regnes ikke lenger som en del av biblioteket ditt. Jellyfin viser dem som plassholdere uten fil, og de talte tidligere med i antall episoder.

## [0.5.1] - 2026-09-19

### Added
- En logo: en play-knapp delt i tre diagramstolper, i sidepanelet og på innloggingsskjermene, som skarp SVG med en `favicon.ico` i flere størrelser for nettlesere og bokmerkelinjer som vil ha en, pluss et ikon for hjemskjermen på mobiler og nettbrett.

## [0.5.0] - 2026-09-19

Du bestemmer hvem som ser hva

### Added
- Tillatelser under Innstillinger → Tilgang, for alle på én gang eller person for person: **Se alle** (andres statistikk og historikk, Brukere-siden, alle direktestrømmer), **Se nettverk** (IP-adresser, enhets-ID-er, lokal eller utenfra), **Se server** (Server-siden, serverloggen, mislykkede innlogginger, filstier) og **Administrere** (innstillinger, oppgaver, import, sletting av avspillinger).
- Innlogging per person, så du kan slippe inn noen få uten å åpne FinStats for alle Jellyfin-brukere.
- Endringer gjelder umiddelbart, også når tilgang tas bort, og alt håndheves av serveren, ikke ved å skjule knapper.

### Changed
- Jellyfin-administratorer har fortsatt alt og er de eneste som kan endre tillatelser; den som får administrere FinStats, kan ikke gi noe, verken til seg selv eller andre.
- Tilbakeblikket for året forblir personlig uansett hva som gis.
- Ingenting endres for eksisterende installasjoner før du gir en tillatelse.

## [0.4.5] - 2026-09-19

### Changed
- FinStats er lisensiert under **GNU General Public License v3.0**. Ingen tidligere versjon ble noen gang publisert, så ingenting er gitt ut under andre vilkår. Lisensene for de medfølgende skriftene følger med dem.

## [0.4.4] - 2026-09-19

### Added
- Esc går tilbake: på et åpnet bibliotek, en bruker eller en tittel tar den deg tilbake dit du kom fra, med listen nøyaktig slik du forlot den. Det som er åpent, lukkes først (avspillingsdetaljer, søk, en nedtrekksliste, mobilmenyen), og Esc lar seg ikke bruke mens du skriver eller bruker et diagram med tastaturet.

## [0.4.3] - 2026-09-19

### Changed
- FinStats husker hva hver side sist viste: går du tilbake til en profil, et bibliotek eller et tidsrom du allerede har sett på, tegnes det umiddelbart, profilbilde og alt, mens ferske tall lastes bak, og siden tegnes bare på nytt hvis noe er endret. Det som huskes, ligger i nettleserfanen og forkastes når du logger ut eller endrer noe.
- Plassholdere for innlasting vises bare når innlastingen faktisk er treg. Før viste alle sider dem i minst en tredjedels sekund.

### Fixed
- Biblioteker som ble slettet i Jellyfin og aldri hadde en avspilling, vises ikke lenger. Et slettet bibliotek med historikk vises fortsatt, merket som fjernet.

## [0.4.2] - 2026-09-19

### Fixed
- På Brukere-siden lå **Admin**-merket på en egen linje og gjorde den raden høyere enn de andre. Nå står merkene ved siden av navnet.

## [0.4.1] - 2026-09-19

### Fixed
- Ordet «null» vises ikke lenger under tittelen på oversikten utenom desember og januar, når det ikke er noe tilbakeblikkbanner å vise.

## [0.4.0] - 2026-09-19

Se hva som er endret uten å forlate appen

### Added
- Fanen **Endringslogg** med alle versjoner, der versjonen du kjører, er merket.
- Versjonen i statuslinjen lenker dit, og fanen viser en prikk etter en oppdatering til du har sett på den.

### Changed
- Å koble historikk på nytt til elementer med nytt navn går langt raskere: fra 18 sekunder til en tredjedels sekund med 666 foreldreløse titler, så oppstarten forblir umiddelbar mens historikken vokser.

## [0.3.3] - 2026-09-19

### Changed
- Biblioteket leses på nytt rett etter at Jellyfins oppgave **Scan Media Library** er ferdig, aldri på en egen tidtaker og aldri mens en skanning pågår. FinStats har alltid bare lest fra Jellyfin, og det starter aldri en skanning der.
- En ukentlig sikkerhetsnett-lesing dekker servere som baserer seg på sanntidsovervåking, der skanneoppgaven kanskje aldri kjører.
- Det gamle intervallet er nå bare en reserve, for når følgingen er slått av eller serveren ikke melder noen skanneoppgave.

### Added
- Bryteren **Følg Jellyfins biblioteksskanning** under Innstillinger → Innsamling, på som standard.

## [0.3.2] - 2026-09-19

### Fixed
- Historikken overlever navneendringer. Jellyfin gir en fil med nytt navn en ny ID, noe som før etterlot de tidligere avspillingene under det rå mappenavnet (`Title (2010) [tmdbid-…] [imdbid-…]`) uten plakat. Nå kobles de avspillingene til det nåværende elementet i biblioteket: via TMDB-, IMDb- eller TVDB-ID når det gamle navnet har en, ellers via renset tittel og år, og episoder via serie pluss sesong- og episodenummer. Et treff må være entydig, så titler som virkelig er slettet, blir som de er.
- Direkte-TV-kanaler importert fra Jellystat ble talt som filmer, fordi Jellystat ikke registrerer elementtype. De gjenkjennes nå som direkte-TV, og nye avspillinger av direkte-TV registreres med Jellyfins egen type.

### Changed
- Direkte-TV holdes helt utenfor tilbakeblikket: topplister, totaler, personlighet, rekorder og rangering.

### Added
- `finstats relink` for å koble historikk på nytt ved behov. Den kjører også ved oppstart og etter hver lesing av biblioteket.

## [0.3.1] - 2026-09-19

### Changed
- Tilbakeblikket er strengt personlig: alle, administratorer inkludert, ser bare sitt eget, og visningen «Alle», brukervelgeren og serverkapittelet er borte.
- Tilbakeblikket åpner på året som er «klart»: inneværende år i desember, ellers fjoråret. Andre år og «Siste 12 måneder» er fortsatt ett klikk unna.

## [0.3.0] - 2026-09-19

Året ditt i tilbakeblikk

### Added
- Fanen **Tilbakeblikk**, en historie om et år å rulle gjennom: timer sett, toppserier, filmer, spor og sjangre, og en seerpersonlighet (nattugle, morgenfugl, helgekriger, maratonseer, filmentusiast, musikkelsker, vanedyr) med diagrammer over tid på døgnet og ukedager.
- Året måned for måned, med tittelen som preget hver måned.
- Rekorder: største dag, største maraton, lengste rekke med dager på rad, lengste enkeltavspilling, mest gjensette tittel, årets første avspilling og den eldste tittelen som ble sett.
- Oppdagelser: nye serier påbegynt, filmer og episoder fullført, og seriene som fikk nøyaktig én episode.
- Et banner på én linje på oversikten i desember og januar når et tilbakeblikk er klart.

## [0.2.0] - 2026-09-19

Mer enn Jellystat noen gang registrerte

### Added
- En tidslinje for hver avspilling: start, pause, gjenopptak, hopp, bytte av lyd og undertekster, direkteavspilling som går over til omkoding, og stopp. I tillegg antall pauser og hopp, hvor avspillingen ble gjenopptatt fra, og om seeren var på det lokale nettverket.
- Fanen **Server**: Jellyfin-versjon, ventende oppdateringer og omstarter, diskbruk per bibliotek (Jellyfin 10.11+), programtillegg, resultater fra planlagte oppgaver og alle registrerte enheter.
- Innsikt på **Avspilling**: samtidige strømmer over tid, hvilke klienter som tvinger frem omkoding, hvor langt folk kommer før de stopper, lokale avspillinger mot avspillinger utenfra og seervaner.
- Hva biblioteket ditt består av: oppløsninger, kodeker, dynamisk område, containere, titler per tiår, tillegg per måned, de største titlene og titlene ingen noen gang har sett, sjekket mot både FinStats' historikk og Jellyfins egne merker for avspilt.
- Oversikt: flest samtidige strømmer, anslått datamengde strømmet, andel avspillinger utenfra, sjangre etter visningstid og mislykkede innlogginger.
- Elementsider: lenker til IMDb og TMDB, studioer, bitdybde og bildefrekvens, og hvem som har tittelen merket som avspilt i Jellyfin.
- Brukersider: sjangre, og filmene, episodene og favorittene Jellyfin har registrert.

### Fixed
- Tidsetiketten på et Spilles nå-kort flyter ikke lenger utenfor kortet.
- Lange etiketter på Avspilling-siden ble kuttet for tidlig.
- En serie som har forlatt biblioteket, viser fortsatt sesong- og episodelisten sin.

## [0.1.0] - 2026-09-19

Første versjon

### Added
- Innsamling i sanntid fra Jellyfins økter: bruker, tittel, klient, enhet, IP-adresse, avspillingsmetode, årsaker til omkoding og maskinvareakselerasjon, detaljer om video, lyd og undertekster, og tid sett mot tid på pause.
- Innlogging med Jellyfin-kontoen din. Et oppsett i to steg lager FinStats' egen API-nøkkel, og passord lagres aldri. Brukere som ikke er administratorer, kan eventuelt logge inn for å se bare sin egen statistikk.
- Sidene Oversikt, Aktivitet, Brukere, Biblioteker, Avspilling og Serverlogg, en kommandopalett (Ctrl/⌘ K) og en statuslinje i sanntid.
- Import fra Jellystat: store sikkerhetskopier strømmes til disk og importeres på sekunder, og det er trygt å importere samme fil to ganger.
- Én liten binærfil med et innebygd nettgrensesnitt og én SQLite-fil, pluss et Docker-image.
