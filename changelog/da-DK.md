# Ændringslog

Alt, hvad der er ændret i FinStats, nyeste først. Denne fil bygges ind i programmet og er det, fanen
**Ændringslog** i appen viser.

Format: `## [version] - date`, eller `## [version] - first day to release day` for en version, der blev
lavet over flere dage, derefter `### Added`, `### Changed`, `### Security`, `### Performance`, `### Stability`,
`### Fixed` eller `### Removed` (Nyt, Ændret, Sikkerhed, Ydeevne, Stabilitet, Rettet, Fjernet) med ét punkt pr.
ændring. Sikkerhed er noget, nogen kunne have misbrugt, Ydeevne noget, der er blevet hurtigere eller lettere,
Stabilitet noget, der ikke længere kan gå ned, standse, lække eller miste data, og Rettet noget, der gav et
forkert svar. En `x.y.0`-version har en kort titellinje under overskriften: den titel bruger appen som
overskrift for hele serien.

## [2.3.1] - 2026-10-26 to 2026-10-28

### Security
- **Opsætningen beder om en engangskode**: indtil FinStats er sat op, skriver den en kode i sin log, når den starter (`docker logs finstats`), og guiden beder om den, før den tester en adresse eller sætter noget op. Ellers kunne den, der først nåede porten, pege FinStats mod sin egen Jellyfin og blive dens administrator eller bruge testen til at se sig om på netværket.
- **Adgang til biblioteker følger Jellyfin**: hver person ser kun de biblioteker, Jellyfin åbner for dem, og i dem kun de titler, Jellyfin viser dem inden for deres aldersgrænse, tags og regler for titler uden aldersgrænse, i bibliotekssider, søgning, medvirkende, billeder, Tilføjet for nylig, Min liste, Kommer snart, kalenderfeedet og notifikationer om nye titler. Alle, der var logget ind, kunne gennemse alle biblioteker.
- Et link kunne sende nogen fra velkomsten videre til et andet websted lige efter login. Nu følges kun FinStats’ egne sider.
- Serverloggen og de mislykkede logins i indsigterne gav alles login-adresser og aktivitet til alle med *Se serveren*. Andres linjer kræver *Se alles aktivitet* og en adresse *Se netværksdetaljer*, som alle andre steder.
- En personlig notifikationsmodtager kunne nå det lokale netværk gennem et navn, der svarede anderledes, anden gang det blev slået op. Der forbindes nu til den adresse, der blev kontrolleret, flere adresseområder, der aldrig når internettet, regnes som lokale, husstandens egen offentlige adresse afvises også, og et navn, der ikke kan slås op, afvises med de samme ord som et lokalt.
- Bag en proxy, der tilføjer sin egen `X-Forwarded-For`-linje, eller fra et IPv6-netværk kunne hvert loginforsøg komme fra en ny adresse, og grænsen trådte aldrig i kraft. Et IPv6-/48 får nu 50 forsøg på fem minutter, en fuld grænse afviser en ny adresse i stedet for at glemme en gammel, hele serveren stopper ved 60 fejl i minuttet, og et mislykket login revideres én gang pr. adresse og tidsrum.
- `FINSTATS_TRUST_PROXY` angiver de proxyer (adresser eller adresseområder), hvis `X-Forwarded-For`, `X-Forwarded-Host` og `X-Forwarded-Proto` troes, og `1` eller `true` tror nu kun på en modpart på loopback eller et privat netværk. Slået til kunne enhver modpart vælge sin egen adresse; slået fra bag en proxy låste ti forkerte gæt alle ude.
- Når brugerne er læst, besvares et login med et navn, der er ukendt, deaktiveret, fjernet eller ikke lukket ind i FinStats, som en forkert adgangskode uden at spørge Jellyfin, og ikke hurtigere, end Jellyfin ville have svaret. Sådanne navne kunne låse Jellyfin-konti, omgå Jellyfins regel om fjernadgang eller gemme sig bag FinStats’ adresse.
- Den, som Jellyfin holder til hjemmenetværket (*Allow remote connections* slået fra), logger kun ind i FinStats derfra, da Jellyfin vurderede indstillingen ud fra FinStats’ egen adresse, og en læsning af brugerne, der ikke siger, om nogen er deaktiveret, holder dem ude.
- En sessionscookie markeres som Secure, når siden, der logges ind på, bruger https, også bag en proxy, der ikke oplyser det.
- Et login, der lykkes, tilgiver ikke længere gættene før det. På https er sessionscookien en `__Host-`-cookie, som et websted på et nabonavn ikke kan erstatte, og som ikke tages imod under noget andet navn, og et log ud afslutter alle de sessioner, browseren sendte.
- Kun en kalendernøgle åbner kalenderfeedet fra dets adresse; alle andre nøgler skal i headeren.
- En API-nøgle kan ikke længere tildele tilladelser, ændre hvem der lukkes ind eller hvad der er offentligt, gendanne en sikkerhedskopi eller tilføje, ændre eller teste en tjeneste eller en notifikationsmodtager: hver af dem overlever nøglen og kræver derfor en browser, der er logget ind.
- Alle svar har `Cross-Origin-Opener-Policy: same-origin` og en `Permissions-Policy`, API-svar har `Cross-Origin-Resource-Policy: same-origin` og er `no-store`, medmindre de siger andet, og CSP’en tillader ikke længere `blob:`-billeder. En side, der åbnede FinStats, kunne senere gøre dens fane til sin egen login-side.
- Hvor FinStats kender det skema, den nås med (en betroet proxys `X-Forwarded-Proto` eller den offentlige adresse), afvises en skrivning fra et andet skema som cross-site.
- `/auth/me`, en titels side og Bibliotekssundhed linker til Jellyfin på dens offentlige adresse; den adresse, FinStats selv læser Jellyfin på, gives kun til administratorer.
- Et billede hentes kun fra Jellyfin for noget, FinStats har, også en offentliggjort profils plakater og kort, så andre bibliotekers billeder ikke kan hentes via id.
- En privat trackers nøgler fjernes nu også fra downloadkøens fejl, ud fra deres form såvel som deres navn, og den, der kun har *Se downloads*, får at vide, hvilken tjeneste der fejlede og hvordan, men aldrig dens interne adresse eller hvortil den omdirigerer.
- Siden Server nævner en enheds seneste bruger kun med *Se alles aktivitet*, og Ikke-tilknyttede medier kræver det ud over *Administrer FinStats*. Uden den tæller afsnittet Opgaver kun ens egne afspilninger og streams, et fjernet bibliotek vises ud fra ens egne afspilninger, og en ventende Tautulli-tavle kan ikke annulleres.
- Indstillingerne viser tjenesten for den offentlige adresse kun med dens vært og Jellyfins adresse og datamappen kun for administratorer, og en titel, biblioteket har mistet, beskrives kun ud fra afspilninger, læseren må se.
- En Tautulli-sikkerhedskopi, hvis tabeller ikke var almindelige tabeller, kunne lade alle andre skrivninger vente, indtil FinStats blev genstartet. En sådan fil afvises, og SQLite læser sikkerhedskopien defensivt.
- En upload på over 4 GB afvises, og en zippet Tautulli-sikkerhedskopi pakkes ikke længere ud end den størrelse, den angiver; begge dele kunne fylde den disk, FinStats’ egen database ligger på.
- Et svar fra Jellyfin, Sonarr, Radarr, Seerr, opslaget af den offentlige adresse eller GeoIP-downloaden læses ikke længere end det højst tilladte, talt efter udpakning.
- Enhver Jellyfin-bruger kunne få FinStats til at holde op med at registrere afspilninger for alle ved at melde tusindvis af korte afspilninger af én titel. At finde ud af, hvem der så sammen, sker nu i én gennemgang og holder ingen lås, mens det står på.
- Afspilninger lige efter hinanden, eller én titel afspillet på skift fra enhed efter enhed, samler ikke længere fremmede i én gruppe, og den, der kun må se sig selv, får kun besked om de personer, ens egen afspilning kørte side om side med, og kun for den tid, den gjorde.
- Et link kunne stadig sende nogen fra velkomsten videre til et andet websted efter login via en sti som `/.//`.
- Den, der administrerer FinStats, kan ikke længere ændre, hvor mange sikkerhedskopier der gemmes eller hvor ofte de skrives, sammenlægningsvinduet (nu højst en time) eller hvornår brugere, aktivitetsloggen og sikkerhedskopier læses, og sikkerhedskopier, som en sådan person tager, skubber aldrig en administrators ud; hjemmeadresser og rejsereglerne kræver det samme som advarslerne, og import af historik kræver også *Se alles aktivitet*.
- En gemt nøgle, et token eller en adgangskode sendes kun til den server, den blev gemt til, og en notifikationsmodtager oprettes eller ændres kun af en, der er logget ind, aldrig med en API-nøgle. En administrator må slå en andens personlige modtager fra eller fjerne den, men ikke flytte den, slå dens certifikatkontrol fra eller læse dens ntfy-emne eller Pushover-brugernøgle.
- Discord- og Slack-webhooks skal være `https`, da deres adresse er deres token.
- En strøm af mislykkede logins kan ikke længere skjule et ægte login eller begrave advarslerne: navne, der ikke er en konto, giver én notifikation i kvarteret, der ikke nævner nogen, advarsler sendes først, og Slack og Discord viser en beskeds indhold som tekst og pinger ingen.
- Placeringen i Tilbageblik, Kapacitet, en notifikation og en offentliggjort profil fortæller ikke mere end siderne: ingen placering eller andel af alles tid til den, der kun må se sig selv, travle øjeblikke kun som dage med tre streams eller flere uden *Se alles aktivitet*, intet mislykket job eller forbindelse til den, der ikke må administrere FinStats, og en offentlig profil ændrer sig én gang om dagen.
- En Tautulli-sikkerhedskopi med en genereret kolonne afvises, og et billede til en importeret afspilning hentes først, når biblioteket har dens titel.
- En gendannelse åbner aldrig FinStats mere, end den er: indstillinger kommer kun tilbage, når der bedes om det, login, tilladelser og offentlige profiler bliver det, både denne installation og sikkerhedskopien tillader, og de offentlige adresser forbliver denne installations. `finstats restore <file>` bringer kun historikken tilbage, som siden gør, og også indstillingerne med `--with-settings`.
- En gendannelse af historikken alene bringer ikke længere hjemmeadresser, dæmpede advarsler, revisionsposter, der udgiver sig for at være denne installations egne, eller placerede titler, biblioteket ikke har, tilbage, og ingen gendannelse bringer afspilninger, logrækker eller enheder dateret før 1990 eller efter i morgen tilbage. En gendannet afspilning regnes som importeret, uanset hvilken kilde den angiver, så en genstart fortsætter den aldrig.
- Datamappen tilhører kun FinStats’ egen bruger (0700, dens filer 0600, `umask 077` i imaget). FinStats nægter at køre som root over en datamappe, en anden bruger ejer, og nævner den `docker exec -u`, der skal bruges, og entrypointet ændrer kun ejeren af mappen og FinStats’ egne poster.
- En modtagers navn må ikke indeholde kontroltegn, og navnet på en enhed, en person eller en titel kan ikke længere forfalske en linje i loggen.
- Et links ruteparameter kan ikke længere nå en anden API-sti end dens sides egen, og FinUIs elementbygger sætter kun en adresse, når den er http(s) eller relativ, aldrig `srcdoc`.
- Søgning, offentliggjorte profiler og arbejde startet manuelt kan ikke længere holde FinStats optaget: en søgning stopper ved 200 tegn, en profil gemmes pr. sprog, kort venter i en kort kø, og et job, adresseopslaget, download af placeringsdatabasen og en testbesked starter kun med mellemrum, uanset hvordan der bedes om dem, testbeskeder talt pr. afsender. At hente placeringsdatabasen kræver også det, som læsning af siden Sikkerhed kræver.
- Indholdet i en forespørgsel skal blive ved med at komme, når de første 15 sekunder er gået, et svar ingen læser, opgives efter 30 sekunder, og én adresse holder højst 64 forbindelser, en betroet proxy undtaget. Nogle hundrede forespørgsler, der aldrig sendte deres indhold, kunne forhindre FinStats i at svare nogen.
- En mailserver læses ikke længere, end noget svar kan være (1 MB for et helt brev), og en testbesked stopper efter lige så lang tid, som en besked i køen må tage. En server, der aldrig afsluttede en linje, kunne få FinStats til at holde mere og mere hukommelse, til den blev stoppet.
- En læsning af et bibliotek vurderes ud fra de titler, læsningerne har gemt, ikke dem en import bragte, og en Jellystat-titel i et bibliotek, der allerede er læst, kommer ind som væk. En import af opdigtede titler kunne få hver læsning af et bibliotek til at stoppe FinStats ved hver opstart.
- Grænsen på 60 mislykkede logins i minuttet for hele serveren tæller kun forsøg, der sendes til Jellyfin, også dem der stadig venter på svar. Navne FinStats selv afviste, lod alle med nogle få adresser forhindre alle i at logge ind, og mange gæt sendt på én gang nåede alle Jellyfin.
- Den, Jellyfin holder på hjemmenetværket, holdes der ved hver forespørgsel, også deres session og nøgler, ikke kun når de logger ind; hjemme er et privat netværk eller husstandens opslåede offentlige adresse, aldrig en adresse skrevet ind som hjemme, og bag en proxy, `FINSTATS_TRUST_PROXY` ikke nævner, holdes de ude. En cookie eller nøgle taget ud af huset åbnede alt overalt fra, og en administrator kunne åbne huset for en adresse efter eget valg.
- En API-nøgle kan ikke længere slå det fra, der ville opdage, at den misbruges, eller de sikkerhedskopier, der ville rulle det tilbage, den gjorde: hvor mange sikkerhedskopier der gemmes og hvor ofte, sammenlægningsvinduet, hjemmeadresser og rejseregler, tidsplanerne for brugere, aktivitetsloggen og sikkerhedskopier, at dæmpe eller løse alle advarsler, en sikkerhedskopi i papirkurven eller at fjerne en notifikationsmodtager. Hver af dem holdt længere end nøglen, når den blev tilbagekaldt.
- Plakaterne fra Sonarr og Radarr og ønskelistens næste afsnit følger også biblioteksadgangen: en serie, Jellyfin skjuler for nogen, har hverken plakat eller dato for dem. At tælle tjenesternes id'er afslørede, hvilke skjulte serier der kom, og en liste nævnte næste afsnit af en skjult serie.
- Sammen-kapitlet i årsoversigten nævner kun dem, der så samtidig med dig, som Sammen-siden gør. En gruppe, du kom med i gennem en anden, nævnte folk, du aldrig så sammen med.
- En import fra Jellystat eller Streamystats læser hele filen, før den tager databasens skrivelås, og skriver derefter kun de rækker, den beholder. Læst under låsen kunne en fil med få ægte rækker og gigabytes af hvad som helst andet holde al anden skrivning ventende, afspilninger og logins også.
- En gendannelse holder én tilladelse pr. person, for ikke flere personer end en husstand mange gange, og tjekker en revisionspost for dubletter uden at læse hver post, den lige har skrevet. En konstrueret sikkerhedskopi kunne holde hukommelse og databasens skrivelås længere og længere under gendannelsen.
- En gendannelse med indstillingerne slår opslaget af den offentlige adresse og hentningen af placeringsdatabasen til kun dér, hvor denne installation også har dem slået til, og gemmer aldrig færre sikkerhedskopier eller skriver dem sjældnere, end denne installation gør. En sikkerhedskopi, der gemte én, kunne få næste kørsel til at fjerne alle andre.
- En filstørrelse, ingen disk kunne rumme, læses som ingen størrelse, og størrelser lægges sammen, så ingen sum kan løbe over. Én importeret størrelse fik bibliotekssiderne til at svare med en fejl for altid.
- Når FinStats ved, at den nås via https (den offentlige adresse er https, eller en betroet proxy siger det), tages en session kun fra dens `__Host-`-cookie, og med en offentlig https-adresse er hver ny session en https-session. Et login sendt uden `Origin` gennem en proxy, der intet sagde, lavede en almindelig session, som et nabowebsted kunne plante i nogens browser.
- Billedet bygges med hver GitHub-handling fastlåst til et commit og begge grundbilleder til et digest, kun flyttet af Dependabot, og en udgivelse bygges uden den delte byggecache. Et flyttet tag på en andens handling kunne have udgivet et hvilket som helst billede som FinStats.

### Performance
- En offentliggjort profil beregnes højst én gang i minuttet, ikke igen for hvert billede og kort på den.
- En persons historik gennemsøges for sikkerhedsadvarsler én ad gangen, uanset hvor mange afspilninger der begynder.
- At logge ind eller ud, tilbagekalde en nøgle eller ændre én persons tilladelser beder kun de liveforbindelser, det angår, om at se efter igen.
- Tautulli-tavlen læser historikken én gang, ikke én gang pr. Plex-bruger.
- Et billede, der mangler i cachen, hentes af én forespørgsel, mens de andre venter, og et, kilden ikke kunne give, spørges der ikke efter igen i 30 sekunder.
- Webbrugerfladens filer gzippes én gang og gemmes, og skrifttyper komprimeres ikke igen.

### Stability
- En sikkerhedskopi i Jellystats ældre format læses en række ad gangen i stedet for hele på én gang, og en linje, der er længere end nogen sikkerhedskopis, afvises.
- To Tautulli-uploads på samme tid skriver ikke længere i den samme fil.
- Et sidetal langt efter slutningen af et biblioteks titler eller af Bibliotekssundhed giver en tom side, ikke en fejl.
- En import eller gendannelse, hvis upload blev afbrudt, afviser ikke længere alle andre indtil en genstart, og hverken den eller en import, der blev opgivet, før den begyndte, efterlader en fil.
- En række i en sikkerhedskopi læses ikke længere end 8 MB, og der modtages én Tautulli-sikkerhedskopi ad gangen.
- En afspilning gemmer højst 2.000 hændelser, og en import springer en afspilning over, der ikke både starter og slutter mellem 1990 og i morgen.
- Ét afsnitsnummer langt under nul får ikke længere hver læsning af biblioteket til at hænge: Bibliotekssundhed (også dens oversigt), Kapacitet, gentilknytning og lokalisering lægger tal sammen og sammenligner dem uden at løbe rundt, og en release-build kontrollerer for overløb, så alt, der overses, stopper én opgave i stedet for at køre i ring.
- Et sæson-, afsnits- eller slutnummer gemmes kun, hvor Jellyfin kunne rumme det (0 til 2.147.483.647), fra Jellyfin, Jellystat og Tautulli ens.
- En Tautulli-sikkerhedskopi, hvis id’er gentages, afvises, da nogle få MB rækker under ét id fik processen til at løbe tør for hukommelse, og en sikkerhedskopi åbnes uforanderlig og efterlader ingen `-wal` eller `-shm`.
- En forbindelse skal sende sin forespørgsel inden for 15 sekunder, og højst 512 betjenes på én gang: nogle hundrede halvåbne sockets tog FinStats offline. En frame på livekanalen holdes til 4 KB.
- Nogle få kilobyte fra Sonarr, Radarr eller Seerr kan ikke længere blive til gigabyte hukommelse: en liste beholder ikke mere, end FinStats læser af den.
- En notifikationsmodtager, der aldrig svarer, holder kun sig selv tilbage: modtagere får sendt side om side, og en, der bliver ved med at time ud, lades i fred i ti minutter.
- En køs resterende tid og samlede hastighed, en gendannet afspilnings tal og den dag, en afspilning forlader papirkurven, kan ikke længere løbe over.
- En gendannelse læser hele filen, før den tager skrivelåsen, afviser en, der pakkes ud til langt mere end sin størrelse, og skriver i portioner, så indsamleren bliver ved med at registrere imens.
- Sikkerhedskopier navngives i UTC og skrives aldrig over en eksisterende fil, og kun FinStats’ egne kopier fra før en opdatering ryddes op.
- En række i serverloggen dateret i fremtiden stopper ikke længere læsningen af aktivitetsloggen: den læses fra dens nyeste række, der ikke ligger i fremtiden.
- Forespørgsler, der ankommer samtidig på et offentliggjort link, beregner dets svar én gang, og dets kort tegnes i deres egen bane, så en, der har linket, ikke længere kan få ejerens Tilbageblik-kort til at svare 503.
- Afspilningerne i et bibliotek, der mangler på Jellyfins liste over biblioteker, venter to dage, før de flyttes til et andet biblioteks kopi af samme titel. Én liste, der udelod et bibliotek, flyttede dem straks, og de forblev flyttet, da biblioteket kom tilbage.

### Fixed
- Mislykkede logins på en konto er én advarsel pr. tidsrum, der vokser til en alarm ved tyve, og et navn tæller kun mod en konto, når det er skrevet helt ("x alice" tæller ikke længere mod alice).
- Discord- og Slack-beskeder escapes først og afkortes bagefter: et enhedsnavn fuld af Markdown fik Discord til at afvise hver advarsel, der nævnte det.
- En offentlig IPv6-adresse accepteres for en personlig modtager.
- Bag en proxy, der selv logger folk ind (nginx `auth_basic`), afvises FinStats’ cookie ikke længere.
- Et visningsnavn, der indeholder `{{url}}`, vises som skrevet i forhåndsvisningen af en offentliggjort side.
- En opsummering for år 9999 viser standardåret i stedet for en fejl.
- En levende side, hvis tilladelse blev taget væk og givet tilbage, mens den var åben, holdes opdateret igen uden genindlæsning.

### Removed
- **Synnmørsk** er ikke længere et af FinStats' sprog: en browser, der valgte det, læser nynorsk, det skriftsprog det blev lavet ud fra.

## [2.3.0] - 2026-10-07 to 2026-10-26

Kapacitet, en livekanal, Enkel tilstand og sprog

### Added
- **Sprog**: FinStats taler 23 sprog. Britisk og amerikansk engelsk samt norsk bokmål og norsk nynorsk er skrevet og kontrolleret af vedligeholderen; dansk, tysk, spansk, finsk, fransk, hindi, islandsk, italiensk, japansk, koreansk, nederlandsk, polsk, portugisisk (Brasilien), russisk, svensk, tyrkisk, ukrainsk og kinesisk (forenklet og traditionelt) er oversættelser, som ingen her har kontrolleret, og siger på engelsk det, de endnu ikke har. Alle sider i begge layouts, den offentlige profil, Ændringsloggen og det, serveren svarer med ord (en fejlmeddelelse, kortene i Tilbageblik, hvad en opgave i Jellyfin gør), vises på det sprog, der er valgt under Indstillinger → Udseende → Sprog, pr. browser; indtil et sprog er valgt, afgør browserens eget sprog det. Tal, datoer, enheder og lister skrives, som det sprog skriver dem. Et script kan bede om et sprog med `Accept-Language`.
- **Enkel tilstand**: et andet layout for dem, der kommer fra Plex, bygget op som en klassisk statistikserver til Plex. En bjælke øverst i stedet for menuen og kun statistiksiderne: Hjem, Biblioteker, Brugere, Historik, Grafer og Indstillinger, med et tandhjul til din profil, Indstillinger, Ændringslog, temaet og log ud. Vælges pr. browser under Indstillinger → Udseende → Layout. Den indlæser kun sin egen halvdel af FinStats og starter derfor hurtigere; Tilbageblik, Sammen, På vej, Sikkerhed og serversiderne findes i FinStats-layoutet. Den bruger FinStats’ egne farver, lyse og mørke, og intet i den bevæger sig.
- **Hjem i Enkel tilstand**: det, der afspilles, med et kort pr. stream (hvordan den afspilles, hjemmefra eller udefra, hvornår den slutter), visningsstatistikken som rangerede kort efter antal afspilninger eller varighed de seneste N dage, hvad hvert bibliotek indeholder, og hvad der er kommet for nylig.
- **Grafer i Enkel tilstand**: afspilninger efter medietype (dag for dag, efter ugedag og time, de ti mest brugte apps og personer), efter hvordan de blev afspillet (dag for dag, flest streams på én gang hver dag, filens og den sendte opløsning, apps og personer) og måned for måned, efter antal afspilninger eller varighed, for alle eller for de valgte personer.
- **Biblioteker, Brugere og titler i Enkel tilstand**: tabeller over alle biblioteker og personer med det, der sidst blev afspillet, på hvad og hvorfra (adressen kun med *Se netværksdetaljer*), og en side for hvert bibliotek, hver person og hver titel med dens tal og dens egen historik.
- Til scripts: `/api/stats/split` (afspilninger efter app, person, opløsning, dag eller måned, opdelt efter medietype eller efter hvordan de blev afspillet); toplister efter antal forskellige seere (`sort=users`); aktivitet afgrænset til afspilninger, der regnes som set (`watched=1`); indsigt kun for de dele, der spørges om (`parts=`); spidsbelastningen opdelt i omkodninger, direkte streaming og direkte afspilning, også pr. dag; hvad hvert bibliotek indeholder, og hvad der sidst blev afspillet der; tilføjet for nylig efter type (`kind=`); varmekortet efter medietype; en persons seneste afspilning; om en stream afspilles hjemmefra.
- **En livekanal**: Oversigt, downloads i På vej og statuslinjen holder sig opdateret over én WebSocket pr. fane (`/api/live`), så det, der afspilles og downloades, ændrer sig på skærmen, i takt med at det sker, og ikke først ved næste kig. Bag en reverse proxy, der ikke sender WebSocket-opgraderinger videre, eller på en forbindelse, der faldt bort uden et ord, spørger siderne med få sekunders mellemrum som før. `/api/now-playing`, `/api/downloads` og `/api/summary` er uændrede, til scripts.
- **Server → Kapacitet**: kan denne server klare det, ud fra hvad FinStats har set? De travleste øjeblikke, der er registreret, og de seneste 90 dages: streams på én gang, og hvor mange der fik billedet omkodet med hardware, uden, eller uden at noget blev oplyst (en historik importeret fra Tautulli er Plex’ og holdes udenfor). Hver disk, bibliotekerne ligger på, og hvor hurtigt det, Jellyfin har tilføjet over seks måneder, fylder den (en disk, der er fyldt for nylig, måles fra en måned efter dens første fil, da den første måned er selve påfyldningen), og, hvor Jellyfin 10.11 eller nyere oplyser ledig plads, hvor længe den rækker i det tempo. FinStats siger aldrig, hvor mange streams serveren kunne klare.
- **Eksportér .ics** under På vej → Kommende: listen, som du har filtreret den, downloadet som en kalenderfil, du importerer én gang. Den opdaterer ikke sig selv; en kalendernøgle fra Indstillinger → API-nøgler giver stadig en kalender, der gør.
- **Værd at kigge på**: Kapacitet gør opmærksom på det, når hver omkodning, Jellyfin har rapporteret om i 90 dage, kørte uden hardwareacceleration, mens omkodning er almindelig, når Jellyfins biblioteksscanning kører en fjerdedel af tiden eller mere, og når en disk har plads til mindre end seks måneder mere. Hvert punkt kommer med sine tal og kan lægges til side ligesom fundene i Bibliotekssundhed.
- **Hvad dit bibliotek består af** kan afgrænses til ét bibliotek eller flere, under Biblioteker og på hvert biblioteks egen side (`library_id=a,b`).
- **Dato- og tidsformater** under Indstillinger → Udseende: en dato som 31.01.2026, 31/01/2026, 31-01-2026, 01/31/2026, 2026-01-31, 2026.01.31 eller 2026/01/31, og et klokkeslæt med 24 eller 12 timer, med eller uden nul foran, eller begge som sproget skriver dem. Gemmes i denne browser.
- **En velkomst**: første gang nogen logger ind, uanset browser, og til sidst i opsætningen, spørger FinStats om sprog, dato- og tidsformat og layout, med et billede af hver. Spring over beholder britisk engelsk, 31/01/2026, 13:05 og FinStats.

### Changed
- Logoet inde på en side er det rene mærke ved siden af navnet, sat i Inter, uanset hvilken skrifttype en forudindstilling vælger; hvor det står alene (faviconet), beholder det sin afrundede flise.
- En afspilning, der er omkodet i software, registreres som sådan, hvor Jellyfin oplyser det; før så den ud præcis som én fra en server, der ikke oplyser det. Siderne kalder stadig begge Software.
- En opdatering kører nu, én gang og ældste version først, det, en version skal gøre ved en database, som en ældre FinStats har skrevet, ved siden af sine migreringer. I 2.3.0 læses serverens detaljer ved første start, så Kapacitet er klar med det samme.

### Performance
- **En side, der står åben, spørger ikke om noget**: Oversigt lavede 30 forespørgsler i minuttet for at holde sig opdateret og downloads i På vej 18; med livekanalen laver ingen af dem nogen. Uanset hvor mange faner der er åbne, spørger FinStats ikke Jellyfin, Sonarr og Radarr om mere end før, og en skjult fane lader stadig downloadkøen blive læst sjældnere.
- En persons side læser kun den persons afspilninger: totalerne og den seneste afspilning blev beregnet over alle afspilninger på serveren og derefter afgrænset til personen, omkring fire sekunder ved en million afspilninger.
- Et biblioteks side læser kun det biblioteks titler og afspilninger, hvor den før talte alle bibliotekers op og beholdt ét: en tredjedel af tiden ved en million afspilninger.
- Når Jellyfins ur gik foran FinStats’, læste *Efter Jellyfins biblioteksscanning* hele biblioteket igen hvert minut, indtil forskellen var indhentet; nu følges hver scanning én gang.
- En GeoIP-fil, der ikke kunne åbnes, blev kopieret igen, hundrede megabyte og mere, og hver adresse blev placeret igen, hvert kvarter; nu prøves den først igen, når den ændrer sig.
- Bibliotekssundhed åbner med det samme, efter at markøren har hvilet på linket: det, der var hentet på forhånd, blev gemt et sted, hvor siden aldrig kiggede.

### Stability
- En opdatering, der fejlede, efter at en migrering var kørt, og som Docker straks genstartede, tog en ny kopi af den halvt opdaterede database ved hver start, og tre starter ryddede den ene kopi væk, som den forrige version kunne åbne. Kopien fra før opdateringen beholdes nu.
- En titel, der blev slettet i Jellyfin, mens biblioteket blev læst (en fil, som Radarr udskiftede), kunne få læsningen til at overse en anden titel, der stadig var der, og markere den som fjernet, og dens afspilninger kunne så blive flyttet over på en anden kopi af den. Det, en læsning ikke så, efterspørges nu først via id.
- Gendannelse af en sikkerhedskopi, der var taget, før en fil blev omdøbt eller opgraderet, gendannede hver afspilning, der siden var flyttet over på den nye fil, en gang til, så den blev fordoblet for altid.
- En anmodning i Seerrs svar, som FinStats ikke kunne læse (et felt, der fik nyt navn i en Seerr-opdatering), blev markeret som fjernet af den daglige fulde læsning.
- En opgave, der ramte en fejl, blev stående som »kører« for altid, og ingen udløser startede den igen, før FinStats blev genstartet; nu slutter den som mislykket.
- En beskadiget Tautulli-zip stoppede uploaden, før filen blev fjernet, så den blev liggende til oprydningen efter seks timer, med alle Plex-brugeres adgangstokens.
- En import eller gendannelse, der blev afbrudt, mens filen var ved at komme ind, efterlod filen i datamappen for altid, adresser og det hele; nu fjernes den ved næste start.
- Et billede kunne blive leveret halvt skrevet, mens det blev lagt i cachen, og browseren beholdt det så sådan.
- Tilbagekaldte API-nøgler blev gemt for altid, uanset hvor mange der blev oprettet og tilbagekaldt.
- En læsning af nogens afspillet- og favoritmærker, der kom tom eller ulæselig tilbage, ryddede dem alle indtil næste læsning; den afvises nu, som en tom læsning af et bibliotek bliver.
- Et afsnit med et slutnummer, intet afsnit kan have (en dato skrevet der), kunne holde Bibliotekssundhed i gang uden ende efter en læsning af biblioteket.

### Fixed
- Et punkt i ændringsloggen med kode inde i fed skrift (i 2.0.2 og 2.0.3) viste backticks i stedet for at vise koden.
- Et diagram over antal kunne nummerere sin akse 0, 3, 5, 8, 10 eller endda 0, 0, 1, 1, 1: trin på 2,5 skrevet afrundet. Antal stiger nu i hele trin.
- En kalendernøgle oprettet af en, der ikke må administrere FinStats, fik et link til browserens egen adresse i stedet for FinStats’ offentlige adresse, så den kun virkede indefra; siden bad også om indstillinger, den fik afslag på.
- En persons seneste afspilning kunne være et par sekunder for kort til at tælle som en afspilning andre steder og dermed nævne en titel, som personens Historik ikke viser.
- En gendannelse fra Slettet for nylig, der mislykkedes, sagde intet og lod knappen blive ved med at dreje; det samme gjorde en mislykket *Hent tilbage* i Bibliotekssundhed og en mislykket *Genåbn* eller *Markér alle som løst* i Sikkerhed.
- Kalenderfeedet hedder FinStats i en kalenderapp, ikke finstats.
- Downloadkøen viste, hvem der havde bedt om hver download, til alle, der måtte se den; andres anmodninger kræver nu *Se alles aktivitet*, som i På vej.
- Når nogen mistede tilladelsen *Få notifikationer*, blev deres modtagere ved med at få tilsendt notifikationer, og de kunne ikke længere slå dem fra.
- En afspilning, der blev sat på pause og genoptaget inden for sammenlægningsvinduet, blev meldt som stoppet én gang, mens dens genoptagelse og egentlige afslutning aldrig blev meldt.
- Et spors afspilninger kunne blive flyttet over på en anden kunstners spor med samme navn (»Intro«), når dets album forsvandt fra biblioteket.
- En modtager af notifikationer skrevet som en IPv4-adresse inde i IPv6 (NAT64, 6to4) blev godkendt som offentlig, selv om den nåede det lokale netværk.
- En Sonarr- eller Radarr-forbindelse, der blev peget mod en anden server, blev ved med at vise den gamle servers plakater i en uge.
- Advarslerne i Sikkerhed sagde, at intet behøvede et kig, når to eller flere personer var valgt, og en side, der blev tom, da dens sidste advarsel blev markeret som løst, havde ingen vej tilbage.
- Alle API-nøgler, der ikke var udløbet, sagde, at de udløb »lige nu«.
- En dialog blev liggende over siden efter Tilbage, og siden kunne ikke rulles, før den var lukket.
- Revisionsloggen viste otte slags poster (en afspilning eller sikkerhedskopi hentet tilbage, papirkurven tømt, offentlige profiler, et ændret udseende) med deres id’er og intet under Detaljer.
- At føje en titel til Min liste fra dens genvejsmenu mislykkedes uden et ord, når serveren sagde nej.
- *Set sammen* på Oversigt behandlede flere personer valgt på én gang som én person.
- Tilbageblikket skrev timer og dage under ti med browserens decimalkomma ved siden af sine egne tusindtalskommaer.
- Kapacitet sagde, at en disk ville blive fuld "i tempoet fra de sidste 0 måneder".
- På siden Sikkerhed talte adresserne uden sted alles afspilninger gennem hele tiden, uanset hvilken person og periode siden var afgrænset til.
- Nogen med *Administrer FinStats*, men ikke *Se alles aktivitet*, kunne lægge hvem som helsts afspilning i papirkurven.
- Et kort fra Tilbageblik eller en offentlig profil kunne komme fra browserens cache på det sprog, der var valgt før.
- De downloads, live-kanalen sendte, var på britisk engelsk, uanset hvilket sprog siden var på.
- Et login, der ændrede, hvad nogen må se, nåede deres andre åbne faner først ved næste kontrol efter et halvt minut.
- En fane, der gav sin live-forbindelse fra sig til en nyere, tog en tilbage på sin næste side, så én persons faner blev ved med at lukke hinandens.
- Tabeller sorterede størrelser fra 1.000 til 1.023 enheder som tekst på sprog, der grupperer cifre med mellemrum, og tyrkiske procenter ligeså.
- En titel med en pladsholder i navnet, som "Big {device} Bunny", blev udfyldt af serverens beskeder.
- Tabeller sorterede varigheder og antal ud af en helhed (5 af 26) som tekst på alle sprog undtagen engelsk og de nordiske.
- En fane, der stadig var åben på ét sprog, fik serverens svar på et andet, så snart en anden fane havde valgt det.
- Et ord ved siden af et tal fik den forkerte form på polsk, russisk og ukrainsk ("5 wieczory", "2 потоков").
- Kapacitet regnede et biblioteks første fyldning som vækst og delte det, hvert bibliotek tilføjede, med seks måneder, uanset hvor længe det var målt over; en fil, der tog en andens plads (en opgradering fra Radarr eller Sonarr, eller en fil Jellyfin gav et nyt id), talte som helt ny, når kun forskellen var ny.
- Folk, der så sammen med SyncPlay, blev ikke en gruppe, når én kom med nogle minutter efter den anden; afspilninger, der slutter samtidig samme sted, hører sammen nu, og historikken grupperes igen én gang.
- Nogen uden *Se alles aktivitet* kunne se af et biblioteks *Aldrig set*, at en anden havde afspillet en titel; nu tæller den kun deres egen visning.

## [2.2.0] - 2026-10-01 to 2026-10-07

Bibliotekssundhed og FinUI

### Added
- **Din egen menu på computeren**: Indstillinger → Udseende tilbyder otte: sidepanelet, som det var, en menu grupperet efter, hvad siderne bruges til, en skinne med ikoner, der åbner sig til sidepanelet under markøren, et sidepanel, der begynder med søgning, en dock, en kommandolinje, sider, du fastgør øverst, og farvede fliser. Sidepanelerne kan stå til højre. Docken glider væk, mens du ruller ned, og kommer tilbage, når du ruller op eller peger mod bunden af vinduet; med den flytter statuslinjen op øverst.
- **Din egen menu på telefonen**: en fanebjælke med Mere (den nye standard), et ark, der åbner halvvejs, hele skærmen, siderne foldet ud i en vifte om tommelfingeren, eller en adresselinje, der vokser til menuen. Hver af dem søger i sine sider og sender andre ord videre til søgningen.
- Begge huskes af den browser, de blev valgt i, så en telefon og en computer hver har sit eget valg; begge er FinUI-komponenter, `desktop-nav` og `mobile-nav`.
- **En genvejsmenu**: højreklik på en titel, en person eller et bibliotek hvor som helst i FinStats (en plakat på Oversigt, en række i en liste, et navn i en tabel, en film i Kommer snart, som ikke er i biblioteket endnu), hold fingeren på den, eller tryk på menutasten, så vokser en menu frem fra det punkt, du trykkede på: åbn, åbn i en ny fane, føj en film eller serie til Min liste, åbn i Jellyfin eller på TMDB og TVDB, kopiér linket. Skift med højreklik giver stadig browserens egen menu. FinUIs `context-menu`.
- **Søgning uden dialog**: søgningen vokser ud af din menu (et sidepanel udvider sig til den, docken rejser sig til den, kommandolinjens navn bliver til den, telefonens menu rummer den) eller tager sidens plads, mens menuen bliver stående, hvis du vælger det i Indstillinger → Udseende. Ctrl+Space åbner den som før; vinduet over alt andet er væk. FinUIs `search`.
- **Alt i et bibliotek**: kortet Tilføjet for nylig i et bibliotek tilbyder *Alt i det*: alle film, serier og album, det indeholder, som plakater, efter navn, år, dato for tilføjelse eller størrelse, afgrænset, mens du skriver. Flere kommer til, mens du ruller.
- **Server → Bibliotekssundhed**: FinStats sammenligner hver fil med sine naboer og viser det, der skiller sig ud, hvert fund med tallene bag. Den rapporterer kun; intet bliver ændret, slettet eller scannet igen i Jellyfin.
- **Huller**: episoder, der mangler mellem to filer i en sæson (»Sæson 2: episode 4 og 5 mangler mellem 3 og 6«). En fil, der indeholder to episoder, tæller som begge; specialafsnit og slutningen af en sæson holdes udenfor.
- **Sæsoner, der skiller sig ud** fra resten af serien i opløsning, HDR eller codec, og **afvigende episoder**: én fil, der er anderledes end resten af sæsonen, for eksempel en SD-episode i en 1080p-sæson. Opløsning går efter bredde eller højde, så en beskåret film i 1920×800 er 1080p.
- **Kopier og versioner**: den samme film eller episode to gange, efter de udbyder-id’er, som Min liste og På vej allerede går efter. To kopier i samme opløsning viser den plads, der bruges to gange; en 4K-kopi ved siden af en 1080p-kopi vises som to versioner og regnes aldrig som spild.
- **Tynde filer**: en bitrate langt under, hvad opløsningen og codec’et normalt kræver (»4K HEVC ved 1,1 Mbps«); en sæsons tynde episoder er ét fund.
- **Dubbing, der stopper**: et lydsprog, som nogle sæsoner har på hver episode og andre på ingen.
- **Aldrig identificeret**: film og serier, som Jellyfin har matchet uden TMDB-, TVDB- eller IMDb-id.
- **Afvis** et fund, der er sådan med vilje, med en note. Det kommer tilbage af sig selv, når dets filer ændrer sig. De, der må administrere FinStats og se serveren, kan afvise; afvisninger står i revisionsloggen og er med i en sikkerhedskopi.
- Et biblioteks side siger, hvor mange ting der er at kigge på der, og fører hen til dem.
- **FinUI**, de komponenter, FinStats er bygget af, er et selvstændigt projekt (`github.com/finstats/finui`): hver knap, hvert kort og hver kontrol i alle tilstande og begge temaer på `finui.finstats.no`, og **FinUI create** til at ændre farver, hjørneradius, tæthed og mere, mens du ser på, og tage det med hjem med én kommando. **Indstillinger → System** fører derhen.
- **Indstillinger → Udseende**: alle vælger selv, hvordan FinStats ser ud for dem, med koden til en FinUI-forudindstilling: grundfarve, accentfarve, diagramfarver, kontrast, hjørneradius, tæthed, kanter, kort, knapper, felter, tabeller, fremhævning, bevægelse, ikoner, menuen, baggrunden bag det hele, fokusringen, hvordan overskrifter sættes, og skrifttyperne til tekst, overskrifter og tal. Dit valg gælder kun dig; siden viser, hvad koden betyder, og fører til FinUI create, hvis du vil ændre den.
- **Stilarter**: elleve komplette udseender at starte fra med ét valg i FinUI create, hvis kode Indstillinger → Udseende tager imod. De er Washi (FinStats, som det leveres), Studio, Noir, Matinee, Console, Pastille, Ledger, Gazette, Arcade, Canopy og Harbour. FinStats selv leveres med ét udseende og tilbyder ingen liste over stilarter.
- **Ikoner, der bevæger sig**: hvert ikon i et link, en knap eller en fane laver sin egen bevægelse én gang, når du peger på det (opdateringsikonet drejer rundt, downloadpilen falder ned i sin bakke, papirkurven løfter låget), og en opdateringsknap, du har trykket på, bliver ved med at dreje, til svaret er kommet. De står stille med reduceret bevægelse og med Bevægelse: Fra.
- **Bevægelse, med FinMotion**: FinUIs komponenter bevæger sig nu: knappen på en kontakt kastes over, stregen under en fane glider hen til den næste fane, et felt, der afvises, ryster på hovedet, en kopiering bliver krydset af i ét strøg, meddelelser lægger sig som kort på hånden og spreder sig i vifte, når du peger på dem, et ark på telefonen giver efter som en elastik, når du trækker i det, rækkerne i en sorteret tabel flytter sig på plads, et tal, der ændrer sig, ruller som en kilometertæller, og diagrammer tegner sig selv, første gang de ses. Det hele følger valget under Bevægelse i Indstillinger → Udseende og står stille med reduceret bevægelse. FinMotion er et selvstændigt projekt (`github.com/finstats/finmotion`), lagt oven på FinUI.
- **Flere ikoner i FinUI**: 239 nu, op fra 71, hvert med sin egen bevægelse (lydstyrke op og ned, et headset og et dæmpet headset, en blyant, der skriver, en klokke, der ringer, en telefon, der vibrerer), og hvert af dem kan downloades fra FinUIs galleri som SVG, stillestående eller animeret.
- **Skrifttyper**: fireogtredive skriftfamilier med åben licens at vælge imellem (Geist, Roboto, Manrope, IBM Plex Sans, Montserrat, Lora, Playfair Display, Fira Code med flere), leveret med programmet ligesom Inter, aldrig hentet andre steder fra og kun indlæst, når de er valgt.
- **Flere grundfarver og accentfarver**: Nord, Solarized, Gruvbox, Catppuccin, Rosé Pine, Tokyo Night, Dracula, Everforest og Ocean; seksten nye accentfarver, ti af dem med én farve om dagen og en anden om natten. Hver grundfarve har nu sit eget papir om dagen: de grå lignede hinanden i det lyse tema.
- **Sletning kan fortrydes i 30 dage**: en slettet afspilning eller sikkerhedskopi venter i papirkurven, og **Fortryd** i meddelelsen, der følger, henter den tilbage. Aktivitet og Indstillinger → Sikkerhedskopier viser hver især, hvad der er slettet for nylig, og hvornår det forsvinder for altid, så længe der er noget. En afspilning i papirkurven er ude af alle totaler, diagrammer, tilbageblik og profiler, og en import eller gendannelse henter den ikke ind igen som ny; den, der så den sammen med nogen, regnes som alene, indtil den kommer tilbage.

### Changed
- Dark Reader og lignende udvidelser lader FinStats’ sider være i fred (`darkreader-lock`): FinStats tegner begge temaer selv, og en side, der blev farvet om, kæmpede imod dem.
- På telefonen glider menuen ikke længere ind fra venstre: fanebjælken står nederst, medmindre en anden er valgt.
- FinStats læser dit bibliotek én gang efter denne opdatering for at lære, hvor filer, der indeholder flere episoder, slutter.
- På en titels side er **Føj til Min liste** lige så høj som **Åbn i Jellyfin** ved siden af; den var en størrelse mindre.
- Filtrene for tidsrum og personer bliver stående øverst på skærmen, mens siden rulles, så du kan skifte fra 7 d til Alt nederst på en lang side; det samme gør filtrene for anmodninger og kommende titler i På vej.
- Knappen, der ruller til toppen, holder sig fri af den menu, der har bunden af vinduet: lavt i hjørnet ved siden af docken, lige over telefonens fanebjælke, tommelknap eller adresselinje og ellers over statuslinjen. Det er FinUIs `to-top`.
- At slette en afspilning eller en sikkerhedskopi flytter den til papirkurven i stedet for at fjerne den med det samme (`PUT` med `{"deleted": true}`; `DELETE` er fjernet fra API’et). Sikkerhedskopier ud over det antal, der beholdes, fjernes stadig for altid.
- **Ændringsloggen** er ét ark i stedet for en stak kort: hvert punkt er en linje for sig med sin type (Nyt, Ændret, Rettet og resten) og typens ikon i en kolonne foran, under en overskrift for hver serie. Listen over versioner står stadig til højre.

### Performance
- Når FinStats åbnes, hentes kun det, der er ændret: et stylesheet, et script eller en skrifttype, som browseren allerede har, bekræftes med nogle få bytes i stedet for at blive sendt igen. Før hentede hvert besøg alle FinStats’ stylesheets og moduler i deres helhed, selv om hver af dem havde et mærke, der skilte versionerne fra hinanden.

### Stability
- Gendannelse af en sikkerhedskopi læser kun indstillingerne fra indstillingsrækken: en fil fra en nyere FinStats, der gemmer mere der, mistede sine indstillinger uden et ord.
- En åben fane vokser ikke længere for hver side, der besøges: hver side, der blev forladt, blev liggende i hukommelsen bag den, der var på skærmen (omkring 2.600 elementer pr. besøg), fastholdt af FinMotions liste over det, den bevæger, og af filterbjælkens rullelytter, indtil fanen blev lukket.
- En fejlstavet kommando (`finstats bakcup`) eller `restore` uden fil opretter eller opgraderer ikke længere en database, før den siger, at den ikke ved, hvad den skal gøre.

### Fixed
- Det lille diagram i en flise løber ikke længere ud over flisen, når tallet ved siden af er bredt (»56 d 23 t«), eller når fire fliser deler en smallere række: diagrammet viger, og tallet beholder sin bredde.
- I det lyse tema blev en primær knap violet under markøren; nu beholder den seglets farve.
- I Indstillinger står en kontakt på linje med indstillingens navn og hjælpetekst; den hang øverst i rækken.
- I en rangeret liste uden noget under et navn, som Topbrugere på Oversigt, sad navnet øverst i rækken ved siden af et centreret billede; nu er det centreret.
- En række statistikfliser i en bredde mellem telefon og computer (fire fliser, tre i bredden) efterlod den sidste alene på en række for sig; fire fliser står nu fire, to eller én i bredden, og tre står tre eller én.
- Læsningen af serveren talte en enhed med, som ingen havde logget ind på, og som den ikke gemmer: opgaven sagde tre enheder, hvor siden Server viste to.
- Ét sekund, ét minut, én time: advarsler sagde »1 sekunder fra hinanden« og »om 1 minutter«, og en Jellyfin-opgave med et interval på ét minut kørte »hvert 1 minutter«.
- `finstats --help` nævner alle de variabler, FinStats læser; `FINSTATS_PUBLIC_IP_URL`, `FINSTATS_ALLOW_LIBRARY_SHRINK` og `FINSTATS_SKIP_PREUPDATE_BACKUP` manglede.
- Opsætningsguiden sagde om en adresse, der ikke var en adresse, at den »ikke kunne oprette forbindelse«, i en sætning, der løb ind i den næste; nu siger den, at adressen ikke er en adresse.
- I Indstillinger → Sikkerhedskopier forsvandt et spørgsmål, der blev stillet, mens en sikkerhedskopi blev skrevet, da sikkerhedskopien var færdig.
- Med menuen som en skinne, der åbner, viste en lukning af søgningen den åbne søgning og den lukkede skinne på samme tid og lod derefter skinnen stå åben, indtil markøren kom og gik. Esc giver nu tastaturets fokus tilbage dertil, hvor det var, da søgningen blev åbnet, og slipper det fokus, et klik gav.
- På telefonen blev siden ved med at være dæmpet, når søgningen blev lukket, indtil søgningen var væk, og skiftede så til fuld lysstyrke i ét billede; nu toner dæmpningen ud sammen med den.
- Trinene til Tautulli-importen sagde, at sikkerhedskopien downloades efter **Backup Database**. Tautulli gemmer den på sin egen maskine: trinene siger nu hvor (dens mappe `backups`, `/config` med Docker), hvilken fil du skal tage, og at du skal kopiere den over.
- En søgning, der blev lukket, før den var færdig med at åbne, efterlod skinnen, der åbner, uden animation: derefter åbnede og lukkede den i ét nu.
- Skinnen, der åbner, blev stående åben, efter at en side var valgt fra den, indtil der blev klikket et andet sted, og i nogle browsere, eller med zoom, tegnede den en rullebjælke under sine ikoner. Nu åbner den kun for markøren og tastaturet og tegner ingen rullebjælke, mens den er lukket.
- På siden Sikkerhed tæller en adresse, som databasen placerer i et land, men ikke på kortet, nu med under det land i stedet for blandt adresserne uden kendt sted.

## [2.1.2] - 2026-10-01

### Added
- **Import fra Tautulli**: tag din Plex-historik med over. Upload Tautullis databasesikkerhedskopi (`.db`-filen eller den `.zip`, der indeholder den) under **Indstillinger → Import**; dens afspilninger får deres egen plads i filteret *Registreret af* i Aktivitet.
- **En koblingstavle til at sige, hvem der er hvem**: Plex-brugere på den ene side, Jellyfin-brugere på den anden. Træk en ledning fra hver Plex-bruger til den, de er nu, eller klik på den ene og så den anden, eller brug tastaturet. Den, der står uden ledning, importeres ikke, og to Plex-konti kan gå ind i én person.
- Film og episoder matches med dit bibliotek efter navn (en film efter titel og år, en episode efter serie og nummer), og en, der ikke er på din server endnu, gemmes og tilknyttes, når den kommer. En genoptaget visning er én afspilning; musik importeres ikke.
- Sikkerhedskopien indeholder Plex’ adgangstokens: FinStats læser dem aldrig og fjerner filen, så snart du har importeret eller begynder forfra.
- **Ikke-tilknyttede medier** under **Indstillinger → Data**: hver titel fra Tautulli, Jellystat eller Streamystats, der ikke matcher noget i dit bibliotek, med hvor den kom fra, og hvor den højst sandsynligt allerede findes. **Find** tilknytter den ud fra de bedste gæt eller en søgning (en film kan findes som et specialafsnit af en serie), og FinStats husker valget, så den samme historik tilknytter sig selv, når den importeres igen. Valgene er med i en sikkerhedskopi.

### Performance
- **Tilbageblikket åbner tre til fem gange hurtigere**: årets mest sete skuespillere og instruktører blev beregnet ved at gennemgå året igen én gang for hver person i det. Målt på en rigtig historik gik et år fra 0,29 s til 0,09 s og de seneste 12 måneder fra 0,53 s til 0,12 s.

### Fixed
- **Importeret historik finder titler, der går under et andet navn**: en afspilning matches også med titlens originalsprogede navn, så en serie, Plex eller Tautulli kendte som 오징어 게임, tilknyttes Squid Game, og *Im Westen nichts Neues* tilknyttes *All Quiet on the Western Front*. Dit biblioteks originaltitler læses ved næste scanning.
- Navne sammenlignes, som søgningen sammenligner dem (store og små bogstaver, accenter, tegnsætning, et årstal skrevet ind i navnet, et indledende »The«), og et udgivelsesår må være ét ved siden af, så »Star Wars: Episode I - The Phantom Menace« finder »… – The Phantom Menace«, og en Kingsman fra 2014 finder den fra 2015. En titel, der kendes under et helt andet navn, bliver stadig aldrig gættet.
- En mislykket import fra Streamystats eller gendannelse blev meldt i notifikationerne med sit interne navn.
- Knappen *Del året som kort* i tilbageblikket og seerplaceringen ved siden af havde intet mellemrum imellem sig.

## [2.1.1] - 2026-10-01

### Changed
- **Min liste tilbydes overalt, hvor der er en titel**: på kortene i Kommer snart på Oversigt, som i På vej → Kommende, og på siden for en episode eller en sæson, hvilket sætter dens serie på listen.
- På en plakat på Oversigt vises Min liste-knappen, mens du peger på kortet eller når det med tastaturet, i stedet for at dække hjørnet af hver plakat; på en berøringsskærm er den der altid.

### Stability
- At gemme en Sonarr-, Radarr- eller Seerr-forbindelse, mens nogen havde **Indstillinger → Udgående forbindelser** åben, kunne få begge til at vente på hinanden for altid.
- **Indstillinger, der ikke kan læses, stopper FinStats ved start** og nævner den indstilling, der er noget galt med, i stedet for i al stilhed at nulstille alle indstillinger til standard.
- **To ændringer af indstillingerne på samme tid** (for eksempel indstillingssiden og en opgaves tidsplan) ophæver ikke længere hinanden.
- En pause, en spoling eller et sporskift, der blev set, mens databasen var optaget, skrives ved næste gennemløb i stedet for at gå tabt.
- En læsning af biblioteket fejler ikke længere på en Min liste med flere poster, der viser sig at være samme titel.

### Fixed
- **En udskiftet fil er ikke »nu på serveren«**: når filen til en film på nogens Min liste opgraderes, får de ikke længere besked om, at den lige er kommet.
- Én mislykket læsning af Jellyfins plugins, planlagte opgaver eller lagerplads tømmer ikke længere den liste på siden Server indtil næste læsning.
- **En afspilning fra Streamystats stopper, hvor den stoppede, på alle sider**: en titels frafaldskurve og en episodes »færdig« gik i stedet efter dens længde og regnede den som anslået.

## [2.1.0] - 2026-09-29 to 2026-09-30

Min liste

### Added
- **Din egen liste**: føj en film eller en serie til Min liste fra dens side, fra Kommende, fra Tilføjet for nylig eller fra søgningen (Ctrl+Enter), og find den under **Min liste** på din egen profil.
- **Hver post siger, hvor den står nu**: på serveren, »5 af 26 episoder«, anmodet om, »Kommer fredag«, ikke på serveren, forsvundet fra biblioteket eller set. Det beregnes, hver gang du kigger, med den samme forståelse af »set« som fremskridtet i dine serier. Sete titler bliver, i en gruppe for sig, indtil du fjerner dem.
- **En titel, der ikke er på din server endnu**, kan komme på listen direkte fra det, Sonarr eller Radarr venter på. Den tilknyttes titlen af sig selv, når den kommer, og en omdøbt fil tager sin post med sig.
- **»På Min liste: nu tilgængelig«**: en notifikation til dine egne modtagere, når noget, du ventede på, kommer. Slået fra, indtil du sætter flueben ved den.
- **Ingen andre ser din liste**, heller ikke en administrator, og dens notifikation går kun til dine modtagere. Intet skrives til Jellyfin, Seerr, Sonarr eller Radarr.
- Min liste er med i en sikkerhedskopi, og en gendannelse fletter listerne sammen uden at fordoble noget.

### Changed
- Poster i `/api/upcoming` har deres titels TVDB- og TMDB-id’er, og `/api/auth/me` siger, om serveren har Min liste.

## [2.0.4] - 2026-09-29

### Added
- **Planlægning af opgaver, som Jellyfin gør det**: hver opgave, FinStats udfører af sig selv, har udløsere (dagligt på et klokkeslæt, ugentligt på en dag, med et interval, ved start eller efter Jellyfins biblioteksscanning), hver med en valgfri tidsgrænse. Klik på en opgave for at se og ændre dens tidsplan; *Tilføj udløser* åbner den samme dialog, som Jellyfin har.
- **Opgaver er sin egen sektion** i Indstillinger, med hver opgaves seneste kørsel i ord (»Kørte sidst for 38 minutter siden og tog 6 minutter«) og en Kør-knap; før lå den øverst i System.
- **Metadataændringer**: hver time læser FinStats det, Jellyfin har gemt siden sidst (navne, beskrivelser, genrer, bedømmelser, medvirkende, fildetaljer og billeder), så en redigering i Jellyfin dukker op uden at vente på en biblioteksscanning.
- Sikkerhedskopiering og geolokationsdatabasen kan også køres fra Opgaver; Esc går fra en opgaves tidsplan tilbage til listen.

### Changed
- **Temaet er en kontakt med tre stop**, Lyst · Enhed · Mørkt, hvor knappen, der skiftede mellem dem, sad før: klik på en tredjedel af den, træk knappen, eller brug piletasterne. Knappen bærer valgets ikon, som glider ind, når valget ændres.
- **Log ud ligger under Indstillinger → Konto**, som viser, hvem der er logget ind her; sidepanelet beholder temakontakten.
- **De gamle indstillinger for tidsplaner er nu udløsere**: *Følg Jellyfins biblioteksscanning* med sit reserveinterval, *Tag sikkerhedskopi hver N. dag* og GeoIP-kontakten *Hold databasen opdateret* blev til de tilsvarende udløsere på deres opgaver, med de værdier, du havde. Indsamling, Sikkerhedskopier og Sikkerhed linker i stedet til tidsplanen.
- En start kører ikke længere alle opgaver igen: hvornår hver af dem sidst blev færdig, gemmes.
- En Jellyfin-opgave uden et estimat endnu viser »ca. … tilbage«, med prikker, der kører i ring, hvor tiden skal stå, i stedet for »ETA…«.

### Performance
- **Kortet i Sikkerhed hakker ikke længere**: når du trækker og zoomer, flyttes det kort, der allerede er tegnet, og det tegnes én gang, når bevægelsen er færdig, og pulsen for det, der afspilles nu, får ikke længere hele kortet til at blive tegnet igen 120 gange i sekundet, mens ingen rører det. Et træk gik fra 34 til 60 billeder i sekundet på en langsom computer.

### Fixed
- **En plakat eller et portræt, der udskiftes i Jellyfin, dukker op i FinStats**: billeder blev cachet en uge på serveren og en uge mere i browseren, uanset hvad Jellyfin havde fået siden. Nu caches de under Jellyfins billedmærke, så et nyt billede er en ny fil, og browseren tjekker, at dens kopi er aktuel, hver gang den viser et. Et udskiftet portræt gemmer kun personen igen, så Metadataændringer læser også personerne.
- **Server → Opgaver viser et estimat, så snart siden åbner**: den resterende tid blev kun målt ud fra aflæsninger, som siden selv foretog, mens den var åben, så hvert besøg begyndte på »ETA…«, indtil den havde set procenten flytte sig. Nu tager FinStats selv tid på en kørende Jellyfin-opgave, hvert 10. sekund, mens den kører, og siden får estimatet serveret. En opgave, som Jellyfin melder på 0 % hele tiden (første fase af en biblioteksscanning), beholder sine prikker, indtil den flytter sig: intet gættes.
- **Værktøjstips på kortet vises ved punktet**: i Firefox kunne værktøjstippet for punktet, der afspiller nu, havne langt væk, og hvor punkter overlapper (nogen, der afspiller hjemme), afhang det af en enkelt pixel, hvilket værktøjstip der blev vist. Nu nævner ét værktøjstip alt under markøren.
- **Statuslinjen lyser op på én måde**: hvert link (streams, Repo, versionen) lyser op som den samme lille boks, og prikkerne mellem elementerne sidder midt i mellemrummene. Når markøren var over versionen, lyste prikken og mellemrummet foran også op.
- **En filsti fylder hele bredden af sit kort**: på en titels Fil-kort og for Jellyfins mapper på siden Server blev en sti klemt ind i én smal celle og løb ned over ni linjer ved siden af tom plads.
- En dialog, der toner ud, sluger ikke længere et klik på siden under den.

## [2.0.3] - 2026-09-28

### Security
- Siden Server gav hver enheds id til alle med *Se serveren*; id’er kræver også *Se netværksdetaljer*, som alle andre steder.
- To tilladelseskontakter, der blev slået om, før den første var gemt, sendte den samme startliste to gange, og den anden ophævede den første: en tilladelse, der blev taget væk, var stadig givet, og en del af en offentlig profil, der blev slået fra, var stadig offentliggjort, mens begge kontakter viste det, der var valgt.

### Stability
- **Kun én import eller gendannelse ad gangen**: to uploads sendt samtidig kom begge igennem kontrollen, og en gendannelse kiggede aldrig efter importer; den anden ventede databasen ud og fejlede efter flere minutters upload.
- **En adresse med en ødelagt `%`-kodning åbner siden for ukendte adresser** i stedet for at efterlade en spinner for altid.
- **Et stort sidetal i en liste giver en tom side**: tallet løb over og kom tilbage som første side igen.

### Fixed
- **Tabeller sorterer tal rigtigt i en tysk eller norsk browser**: »1,204« blev læst som lidt over én i stedet for 1.204, og alt over en million som tekst.
- **Et diagrams tabelvisning sorterer datoer, ugedage og positioner efter tid**, ikke efter bogstaverne.
- **Downloadhistorikken nævner, hvor importer kom fra**: Sonarr og Radarr oplyser kun indekser og protokol ved hentningen, så de fordelinger var altid tomme. En mislykket sæsonpakke er én fejl, ikke én pr. episode.
- **Når en afspilning, der blev set sammen, slettes**, finder FinStats titlens grupper igen i stedet for at efterlade en »gruppe« på én.
- **Tid sammen for én person er aldrig længere end den tid, vedkommende selv var med**: siden Sammen tilskrev personen hele aftenen.
- **En aften i selskab hen over midnat nytårsaften** (eller kanten af »seneste N dage«) hører i sin helhed til den periode, den begyndte i, i stedet for at blive til to aftener alene.
- **Importerede afspilninger af episoder får episodens navn**, hvor Jellystat kun gemte seriens.
- **Geolokation finder bydatabasen ved siden af land- og ASN-databaserne**, som `geoipupdate` skriver, i stedet for at slå sig selv fra.
- **Det sted, der oftest spoles tilbage, følger sidens filtre**, ligesom tællingerne gør.
- **»Dit år er klar«** går kun til personer, hvis tilbageblik har noget i sig.
- **At markere en dæmpet advarsel som løst igen holder parret dæmpet**; kun en genåbning ophæver en dæmpning.
- **En udløbet API-nøgle tæller ikke længere med blandt de tyve.**
- **Grænsen for sessioner lader den ældste session gå**, selv når mange logins lander i samme sekund.
- **En hjemmeadresse, som opslaget fandt, bliver**, når den samme fjernes fra den liste, du selv har skrevet.
- **Billeder i revisionsloggen og listen over nøgler**, hvor der kun var initialer.
- **En administrators liste over notifikationer viser, hvis en personlig modtager er**, i stedet for »Din« på alles; **Sendt for nylig** viser alle modtagere af den seneste hændelse.
- **Esc på en titel eller en person forlader aldrig FinStats** og lukker kun den øverste dialog.
- **En side tegnes, selv når en lagring annullerer den forhåndshentning, den ventede på.**
- **Afspilles nu viser en seer, der spoler under en pause**, der hvor vedkommende er nu, ikke først når der trykkes på afspil.
- **At fjerne en forbindelse opdaterer, hvilke sider der findes**, ligesom at gemme en gør.
- **Geolokationsdatabasens byggedato beholder sit årstal** på amerikansk engelsk.

## [2.0.2] - 2026-09-28

### Security
- Bag en proxy talte grænsen for logins den første `X-Forwarded-For`-adresse, som er den, klienten selv sender: en ny opdigtet adresse for hvert forsøg, og grænsen trådte aldrig i kraft. Nu er det den sidste, altså den, proxyen skrev.
- *Medtag IP-adresser og steder* sendte adresser til en personlig modtager, hvis ejer mangler *Se netværksdetaljer*, og som ingen side ville vise vedkommende. Andres mislykkede logins kræver nu også *Se serveren*, som på siden Sikkerhed.

### Stability
- **Et stop går aldrig tabt**: når en læsning af biblioteket blev afvist i de første øjeblikke efter start, før serveren lyttede efter det, blev afvisningen noteret, og FinStats kørte videre i stedet for at stoppe.
- **En Jellyfin, der svarer i en form, FinStats ikke kan læse, bliver afvist**, ikke troet: hver bruger, eller hver titel i et bibliotek, der kom under nøgler, FinStats ikke kender (`id` for `Id`), slap igennem vagten med fuldt antal og blev så markeret som fjernet.
- **Afslutningen på en afspilning går aldrig tabt**: når databasen var optaget ud over ventetiden (en import, der holdt den), blev en afspilning, der sluttede i det øjeblik, stående som igangværende indtil næste genstart. Nu skrives den ved næste gennemløb.
- **Indstillinger, som en gendannelse ikke kan læse, bliver ikke rørt**: en sikkerhedskopi, hvis indstillinger ikke længere passede til denne version, nulstillede alle indstillinger ved gendannelse: sammenlægningsvindue, tilladelser, hjemmeadresser, tidsplan.
- **Et kontroltegn i et navn eller en titel ødelægger ikke længere kortene**: profilkortet, hvert historiekort, der viste den titel, og ZIP-filen med dem svarede med en fejl.

### Fixed
- **Notifikationer stopper for den, der er deaktiveret eller slettet i Jellyfin**: deres personlige modtagere fik fortsat alt, hvad de gamle tilladelser gav adgang til.
- **En notifikation i kø tjekkes igen, før den sendes**: en modtager, der var slået fra, eller en ejer, der havde mistet en tilladelse, fik stadig det, der ventede på et nyt forsøg.
- **En modtager, der altid svarer `Retry-After`, bliver opgivet** efter fem forsøg som alle andre i stedet for at blive prøvet igen i tredive dage.
- **Enheds-id’er på en persons side** vises kun med *Se netværksdetaljer*, som alle andre steder.
- **Et offentliggjort år venter et døgn med alt**: dets færdige og opgivne serier fulgte hver afspilning, så et tryk på afspil i en opgivet serie ændrede den offentlige side inden for sekunder.
- **En serie, der er anmodet om, tæller som set**, når en af dens episoder er: hver serie, der var anmodet om, blev læst som aldrig set, i kapitlet Anmodninger og på kortet Ønsker.
- **Tilbageblikkets kapitel Sammen udelader live-tv** ligesom resten af året: en kanal, der stod tændt, mens folk så sammen, kunne blive årets toptitel, og andelen kunne komme over 100 %.
- **Kortet Dage for »de seneste 12 måneder« slutter på den nyeste dag**: gitteret mistede de seneste tre-fire uger.
- **En series nye episoder meldes én gang pr. aften**: episoder på hver sin side af midnat UTC blev meldt to gange på en server øst eller vest for den.
- **Når Seerr forbindes, meldes gamle anmodninger ikke som klar til at blive set**: en anmodning, der allerede var tilgængelig ved første læsning, uden andre spor af hvornår, blev læst, som om den kom i det øjeblik.
- **Titlerne i et bibliotek, der slettes i Jellyfin, forsvinder med det**: de blev liggende i søgningen, på hylden Tilføjet for nylig og i totalerne.
- **En remux forbliver direkte streaming**, når en aflæsning et øjeblik mangler sine omkodningsdetaljer (ved en spoling), i stedet for at blive en omkodning resten af afspilningen.
- **»Jellyfin er nede« betyder, at læsninger er mislykkedes flere gange i træk**: spredte timeouts med dages mellemrum, på en server, der stod stille på socketen, blev lagt sammen til det.
- **En import fra Streamystats viser sit fremskridt på sit eget kort**, ikke oven på det seneste resultat fra Jellystat-importen.
- **En adresse, hvis værtsnavn begynder med »web«** (`http://web:8096`), læses som et værtsnavn og bliver ikke afkortet, som om det var stien til Jellyfins webklient.
- **En køfejl skjuler alle loginoplysningerne** i en adresse, også når adgangskoden indeholder en skråstreg.

## [2.0.1] - 2026-09-28

### Changed
- **En titels sprog står ikke længere i vejen**: det øverste felt nævner de første tre lyd- og undertekstsprog, én linje hver, og et nyt **Sprog**-kort viser dem alle: de komplette samlet, de delvise med hvor mange episoder der har dem. En lang series to dusin undertekstsprog skubbede før baggrundsbilledet en hel skærm ned.

### Fixed
- **En titels plakat bliver ved siden af dens navn**: når teksten ved siden af blev længere end plakaten (et langt resumé, mange studier), sank plakaten ned i bunden af det øverste felt under tom grafik.
- **En version, FinStats ikke kan læse, kaldes netop det**: en `app_version`, der var redigeret i hånden til noget ulæseligt, fik FinStats til at nægte at starte som »den ældre FinStats 2.0.1«, hvilket den ikke var. Den nægter stadig og siger nu, hvilken værdi den ikke kunne læse, og hvordan den skal skrives.

## [2.0.0] - 2026-09-25 to 2026-09-28

Tilbageblik 2026, offentlige profiler og Sammen

### Added
- **Hvem du så sammen med**, i tilbageblikket: aftener i selskab, tiden brugt sammen med andre, den titel, der samlede folk mest, og op til tre ledsagere, nævnt ved navn i appen og med link til deres profiler.
- **Serier set til ende**, i tilbageblikket: den sidste episode af hver serie, der blev set færdig det år, efter samme forståelse af »set« som profilens fremskridtsbjælker, og også dem, der blev påbegyndt og lagt til side, under halvt set og uden noget afspillet i to måneder.
- **Det, du bad om**, i tilbageblikket: anmodninger i Seerr fra årets løb, hvor mange der kom, og hvor mange du så bagefter. Kun hvor der findes anmodninger.
- **Året mod året før**: timer, afspilninger og dage, og hvor meget mere eller mindre.
- **Året som en historie**: hvert kapitel som et kort på 1080×1920, det format, historier på telefonen bruger (året, tallene, topserier, -film og -musik, genrer, personligheden, timer og dage, hver dag i året, rekorder, selskab, færdige serier, anmodninger og året før). Gem ét, eller dem alle som en ZIP, fra »Del året som kort«.
- **Et kort nævner aldrig andre**: kortene tegnes ud fra en kopi af året uden felt til en anden person, en placering eller en app. I appen bærer et kort slet intet navn.
- **Serverens år**, for Jellyfin-administratorer: alle titler og totaler og husets personlighed, uden at nogen bliver rangeret eller nævnt. En kontakt på siden Tilbageblik, `scope=server` i API’et.
- **»Dit tilbageblik på året er klar«**: en notifikation i december, én gang til hver person, der så noget det år.
- **Offentlige profiler**: en person kan offentliggøre en del af sin profil via et link, som alle kan åbne uden konto (totaler og toptitler, stimer og hvornår de ser, hele året som dets historie i kort, de ti seneste afspilninger), og hver del er slået fra, indtil ejeren slår den til under Indstillinger → Offentlig profil. Enheder, adresser, apps, afspilningsmåder, filstier og andre personer offentliggøres aldrig, uanset hvad der er slået til, og det viste navn er et, ejeren selv skriver, aldrig vedkommendes brugernavn i Jellyfin.
- **Intet på en offentliggjort side er nyere end et døgn**, hverken seneste afspilninger eller nogen total, så en side, som nogen holder øje med, aldrig røber, hvem der ser noget lige nu.
- **Kort, der kan deles**: hver offentliggjort profil har et billede på 1200×630, af profilen og af året, som Discord, Signal, Mastodon og resten viser, når linket indsættes, og som kan downloades og deles. Tegnes på serveren ud fra det, siden viser, og intet mere.
- **Et link, du kan trække tilbage**: 22 tilfældige tegn, nulstillet med ét klik; det gamle link svarer så præcis som et, der aldrig har eksisteret.
- **Offentlige profiler er slået fra, indtil en administrator tillader dem**: én kontakt for hele serveren, en liste over, hvem der offentliggør hvad, og en måde at fjerne en profil på. Offentliggørelser, ændringer, nulstillinger og fjernelser står i revisionsloggen, og en nøgle kan ikke gøre noget offentligt.
- **Sammen**, en side for sig: hvem der ser med hvem, hvor meget, og hvordan det har ændret sig. Tid sammen, aftener og andelen af visningstiden i selskab, hver sammenholdt med perioden før; timer i selskab mod timer alene pr. dag eller uge; hvert par af personer med deres aftener, tid sammen, hvornår de sidst så noget, og hvad de ser mest; de titler, der oftest ses sammen; de seneste aftener; og, for den, der må se alle, hver persons tid alene mod tiden i selskab.
- Hvert par tæller: en aften med tre tæller for hvert af dens tre par, hvor kortet før kun talte præcis den samme gruppe personer. Et pars tid sammen er det korteste af de to ophold.
- Kortet **Set sammen** på Oversigt og på en profil linker til siden, og en profils kort viser, hvor stor en andel af personens visningstid der var i selskab.
- `GET /api/stats/groups` får `pairs`, `series`, `people`, `previous`, `bucket` samt `watch_s` og `share` i totalerne; de nøgler, den havde, er uændrede.
- **Hvor folk stopper**, på hver film- og episodeside: andelen af afspilninger, der stadig ser med ved hvert minut, fra alle i starten til dem, der nåede slutningen, med de afspilningers tilbagespolinger og tilkoblinger af undertekster markeret på samme akse. Kortet siger, hvor mange stop der er målt (afspilninger, FinStats registrerede), og hvor mange der er anslået (importerede afspilninger, ud fra hvor længe de varede), og kræver tre afspilninger, før det tegner noget.
- **Hvem der bliver ved med at se**, på en serieside: hvor mange personer der startede hver episode, i rækkefølge, og en **Færdig**-kolonne pr. episode. »Alle giver op efter episode tre« er folk, der aldrig trykker afspil på episode fire.
- **Filer, der aldrig afspilles**, under Afspilning: en film eller episode, der er startet tre gange eller flere og aldrig er kommet forbi tredive sekunder, med hvem der prøvede, og i hvilke apps. Jellyfins log indeholder ingen afspilningsfejl, så afspilningerne er vidnet; mindste afspilningslængde gælder ikke her, da det netop er den, de afspilninger aldrig når.
- **Mest spolet tilbage** og **Undertekster slået til**, under Afspilning: titler rangeret efter tilbagespolinger pr. afspilning med det minut, de samler sig om, og efter andelen af afspilninger, hvor undertekster blev slået til inden for de første ti minutter. Alle tre lister kræver *Se alles aktivitet*. Spring kortere end 20 sekunder blev aldrig registreret, så en kort gentagelse af én replik er usynlig med vilje.
- En spoling husker, hvor den kom fra, som et tal ved siden af sin etiket (`from_s` i en afspilnings tidslinje). Hver spoling, der allerede er registreret, får sit tal fra etiketten, og det samme gør en gendannet sikkerhedskopi fra før dette.
- `GET /api/stats/files` til de tre lister; `GET /api/items/{id}` får `insights`, og en series episoder får `users` og `finished`.
- **API-nøgler**: alle, der er logget ind, kan oprette nøgler til sig selv under Indstillinger → API-nøgler, til et script, et dashboard eller en telefon. En nøgle sendes som `Authorization: Bearer fs_…`, vises én gang, gemmes hashet, kan have en udløbsdato og tilbagekaldes med et klik; en administrator ser alles og kan tilbagekalde dem. Den svarer til den samme person med de samme tilladelser som et login, læst på ny ved hver forespørgsel, så den, der mister retten til at logge ind, mister alle sine nøgler på én gang. En nøgle kan hverken oprette eller tilbagekalde nøgler.
- **Et kalenderfeed**: `GET /api/calendar.ics?key=…` er det, Sonarr og Radarr har på vej, som en iCalendar, en telefon kan abonnere på. En nøgle med omfanget *kalender* åbner feedet og intet andet; feedet læser nøglen fra adressen, fordi en abonneret kalender ikke kan sende en header, aldrig fra cookien, og nævner ingen.
- **En revisionslog** under Server → Revisionslog, for Jellyfin-administratorer: hvert login og hvert mislykket forsøg med sin adresse, hver indstilling eller tilladelse, der blev ændret, og til hvad, hver nøgle, der blev oprettet, brugt eller tilbagekaldt, hver forbindelse, modtager, sikkerhedskopi og import, hver afspilning, der blev slettet, og hver advarsel, der blev markeret som løst, med hvem, hvorfra, via hvilken nøgle, og om det lykkedes. Gemmes et år og er med i sikkerhedskopier. `GET /api/audit` bladrer igennem den.
- **Import fra Streamystats**, sin egen sektion under Indstillinger ved siden af Jellystat. Tag dens fil fra **Settings → Backup & Import → Download Backup**, og slip den ind: års historik kommer på et par sekunder, og intet, du allerede har, tælles igen. `finstats import-streamystats <file>` gør det uden brugerflade, `POST /api/import/streamystats` gennem API’et; én import kører ad gangen, uanset slags.
- **Brugte du begge sporingsværktøjer? Importér begge filer.** En afspilning genkendes som en, FinStats allerede har, på sporingsværktøjets eget id for den og ellers på, at den samme person ser det samme element, og at en af afspilningens ender ligger tæt på en i historikken. Derfor kan en eksport fra Jellystat og en fra Streamystats af de samme aftener begge importeres, i vilkårlig rækkefølge, og ingen af dem importeres oven i en aften, FinStats selv registrerede. »Tæt på« er sammenlægningsvinduet under Indstillinger → Indsamling.
- Importerede rækker siger, hvad de ikke kan sige. En afspilning, Streamystats selv registrerede, har intet om filen (intet codec, ingen opløsning, bitrate eller container), så de felter står tomme i stedet for at blive udfyldt med et gæt, og de kort, der ville vise dem, skjuler sig. En afspilning, den havde hentet fra Jellystat, har hele sessionen med, og den læses i stedet: dateret efter det øjeblik, den sluttede, det eneste øjeblik, nogen af sporingsværktøjerne gemte, og beskrevet som en omkodning, kun når sessionen virkelig var det.
- Rækker, som Streamystats skrev, fordi Jellyfin meldte et element set, for en visning, den aldrig så, tælles og springes over: de er lige så lange som hele filmen og ville blive til visningstid en aften, hvor ingen så noget. Resultatet siger, hvor mange der var.
- Hvordan data fra Streamystats tolkes, felt for felt, lært af en rigtig eksport og ikke af dokumentation: `docs/streamystats-import.md`.
- **Hvilket sporingsværktøj hver afspilning kom fra**, som et filter i Aktivitet: det, FinStats selv registrerede, det, der kom fra Jellystat, og det, der kom fra Streamystats. Kun dem, din historik faktisk indeholder, tilbydes, og filteret er afgrænset som alle andre lister, så den, der kun må se sine egne afspilninger, kun får at vide, hvor vedkommendes egen historik kom fra.
- **Filtrene i Aktivitet er rullelister, du sætter flueben i**, så du kan bede om film *og* episoder uden musik, to afspilningsmåder eller to personer på én gang i stedet for én af hver eller det hele. Medietype har fået **Andet**, nu hvor den kan kombineres.
- Flere personer på én gang, overalt hvor brugerfilteret findes (Oversigt, Afspilning, Biblioteker, Sikkerhed og resten), og kortet og al statistik følger med. Et brugerfilter indsnævrer stadig kun det, nogen må se: uden *Se alles aktivitet* låses en forespørgsel til den, der spørger, uanset hvem og hvor mange adressen nævner.
- **Et lyst tema**: washi-papir med blæksort tekst og en cinnoberrød accentfarve. FinStats følger enhedens lyse eller mørke indstilling, indtil du selv vælger.
- **En temaknap ved siden af Log ud** skifter mellem Enhed, Lyst og Mørkt. Valget gemmes i denne browser og tages i brug, før siden tegnes, så det andet tema aldrig blinker forbi.
- **Åbn i Jellyfin**: hver side for en film, serie eller episode har en knap, der åbner titlens side i Jellyfin i en ny fane.
- **Jellyfins adresse for brugerne** under Indstillinger → Jellyfin: der, hvor de knapper fører hen, når den adresse, FinStats forbinder til, ikke er en, som en browser kan nå. Tom beholder den adresse, FinStats forbinder til; kun en Jellyfin-administrator kan angive den.

### Changed
- **FinStats er flyttet**: `github.com/finstats/finstats`, og imaget er `ghcr.io/finstats/finstats`. Skift imagenavnet i din `docker run` eller compose-fil for at blive ved med at få opdateringer; datamappen forbliver, som den er.
- **Indstillinger viser én sektion ad gangen.** Siden var fjorten kort i én kolonne og seks hundrede ords hjælpetekst. Nu har hver sektion sin egen adresse (`/settings/collection`), en liste til venstre skifter mellem dem, og på en telefon brydes listen op i chips. Links til de gamle ankre virker stadig.
- **En indstilling er én række**: hvad den er, én linje, der siger reglen, og kontrollen til højre. Tal i en sektion gemmes samlet med én knap nederst; kontakter gemmer stadig sig selv. Statuskortene (Opgaver, Udgående forbindelser, Database) og linket til licenserne ligger under **System**, og de to importværktøjer har deres egen sektion.
- **Find en indstilling** fra feltet i toppen (`/` sætter fokus i det) med et hvilket som helst ord i dens navn; det viser, hvilken sektion indstillingen ligger i, og Enter fører dig til rækken.
- **Indstillinger er for alle, der er logget ind**: de sektioner, der styrer serveren, forbliver hos dem, der må styre den, og en person, der kun havde lov til notifikationer, kan endelig nå sine egne modtagere.
- **Siden Server viser én sektion ad gangen**, ligesom Indstillinger: Oversigt (system og lagerplads), Opgaver, Enheder, Plugins, Log og Revisionslog, hver på sin egen adresse. Links til `/server#jobs` virker stadig.
- **Serverloggen er en sektion på siden Server** (`/server/log`) i stedet for en side for sig i menuen; `/events` sender videre dertil med sine filtre.
- En notifikation om en mislykket opgave, en mislykket sikkerhedskopi eller en forbindelse, der holdt op med at svare, linker til den indstillingssektion, det drejer sig om, i stedet for til toppen af siden.
- Det, dit bibliotek kan fortælle om en importeret afspilning (hvilken slags indhold det var, hvilken episode, spilletiden, hvilket bibliotek den hører til), udfyldes efter **hver** læsning af biblioteket, ikke kun i det øjeblik, en import sker. Historik importeres som regel, før FinStats overhovedet har læst biblioteket, så de afspilninger forblev gættede eller ukendte for altid.
- Gendannelse af en sikkerhedskopi fra FinStats tilføjer ikke længere en afspilning, der kom fra et sporingsværktøj under et andet id, og beholder stadig to ægte visninger af det samme med få minutters mellemrum.
- En afspilnings detaljer nævner det sporingsværktøj, der bragte den ind, i stedet for at kalde alt importeret for en Jellystat-afspilning.

### Performance
- **En genstart går næsten øjeblikkeligt, uanset hvor lang historikken er.** Hver start beregnede, hvem der så sammen, over hele historikken og afgjorde igen, hvilke afspilninger der var lokale, begge dele uden at ændre noget: 24 s og 5,5 s ved ti millioner afspilninger. Nu gør en start kun det, den forrige kørsel efterlod, og det hele igen kun efter en opdatering, eller når den bagvedliggende indstilling er ændret.
- **Mange seere, der starter på én gang, registreres på sekunder.** Hver ny afspilning, hver lagring af fremskridt og hver enhed blev skrevet for sig, og hver afsluttet afspilning blev lukket med sin egen gennemgang af titlens hele historik; nu skriver et gennemløb hver slags samlet. 100.000 afspilninger, der starter samtidig, tager 3 s, hvor 25.000, der sluttede samtidig, før tog over fire minutter.
- **Afspilles nu forbliver hurtig med tusindvis af seere.** For at finde, hvem der ser sammen, blev hver stream sammenlignet med hver anden, og hver ledsager blev nævnt ved navn: fyrre sekunder ved 10.000 streams og gigabytes af hukommelse oveni. Nu sammenlignes streams inden for én titel, og hver viser nogle få navne og hvor mange andre.
- Beregningen af, hvem der så sammen, omskrev hver grupperet afspilning i historikken, hver gang FinStats startede, importerede eller så en afspilning slutte; nu skriver den kun det, der er ændret. At finde ud af, hvilke afspilninger der var afsluttet, bliver ikke længere langsommere med kvadratet på antallet af igangværende afspilninger.
- **Genkendelse af en afspilning, som historikken allerede har, går via et indeks**, så en import af 150.000 afspilninger tager sekunder, og fornyet tilknytning efter en læsning af biblioteket går fra titel til titel i stedet for at læse hver afspilning, mens databasen holdes.
- **Indsigt i afspilning og Oversigt er hurtigere over al tid**: de uafhængige dele beregnes side om side, samtidighedsdiagrammet slår den lokale dag op én gang pr. kvarter i stedet for én gang pr. afspilning, og genrer tælles pr. titel. Ved en million afspilninger: 4,1 s til 2,4 s og 1,6 s til 1,1 s.

### Stability
- **Et tv-bibliotek læses ikke længere som et bibliotek, der er blevet tømt.** At læse et bibliotek tager to gennemløb (hvert element, derefter et mindre, kun for medvirkende i film og serier), og kontrollen, der afgør, om en læsning er pålidelig nok til at fjerne det, den ikke så, kiggede på det andet. I et serie- eller musikbibliotek ligger de to tal langt fra hinanden, så kontrollen afviste hver læsning og stoppede FinStats, og installationen genstartede og gjorde det samme igen. Intet blev nogensinde fjernet ved en fejl, men ingen læsning af biblioteket kunne blive færdig. Hvis dit serie- eller musikbibliotek ikke har fået nye titler med, er det grunden.
- En import kunne fejle med »database is locked«, når en anden skrivning (en afspilning, der blev opdateret) landede mellem dens første læsning og dens første skrivning. Begge importværktøjer tager skrivelåsen, når de begynder, og databasens journal forbliver ikke længere lige så stor som den største import, der nogensinde er lavet.
- **En databasefil, der er beskadiget, afkortet, tilhører en anden eller er skrivebeskyttet, afvises med det samme**, med filens navn og besked om, at intet blev ændret, i stedet for 30 sekunders stilhed; der skrives aldrig i et andet programs database.
- En sikkerhedskopi eller et øjebliksbillede før opdatering, der afbrydes halvvejs, efterlader ikke længere en fil, som kunne tage pladsen fra et godt øjebliksbillede blandt de tre, der beholdes.
- **At overskrive stedsdatabasen, mens FinStats kører, får det ikke længere til at gå ned**: FinStats læser fra en privat kopi af filen.
- En Jellyfin-app, der meldte et nyt enheds-id hver gang, kunne få FinStats’ hukommelsesforbrug til at vokse, så længe det kørte; enheder, der ikke er set i fem minutter, glemmes nu.
- Én person kan højst have 30 aktive loginsessioner; den ældste logges ud ved næste login.

### Fixed
- **Den samme aften tælles ikke længere to gange, når et element har fået nyt navn.** Når historik blev tilknyttet igen til en titel, der var tilføjet på ny i Jellyfin, kunne en afspilning importeret fra et sporingsværktøj blive en dublet af en, der allerede fandtes, fordi kontrollen, der ville have fanget det, kørte, før id’et blev flyttet. Nu kører den igen overalt, hvor id’er tilknyttes igen. Kun importerede afspilninger fjernes nogensinde, aldrig en, FinStats selv registrerede, og hvor mange der forsvandt, står i loggen. Første start af 2.0 gennemgår hele historikken én gang for dem, der allerede findes.
- **En session følger nu personen, som Jellyfin ser vedkommende nu.** En administrator, som Jellyfin fratog rettighederne, beholdt fuld adgang til FinStats resten af sin 30 dage lange session, og den, der var deaktiveret eller slettet i Jellyfin, forblev logget ind. Nu mister de adgangen, så snart FinStats læser brugerne fra Jellyfin (hvert 15. minut eller med det samme fra Indstillinger), som API-nøgler altid har gjort.
- Et login skriver det, Jellyfin siger om personen, ind i FinStats’ brugerliste, så en importeret sikkerhedskopi fra et sporingsværktøj ikke længere kan gøre nogen til administrator, før den første læsning af brugerne retter det.
- Den, der er slettet i Jellyfin, markeres som fjernet, også når to læsninger af brugerne falder i samme sekund.
- **Downloadfejl viser aldrig en downloadklients adgangskode eller en indeksers API-nøgle**, som Sonarr og Radarr kan citere i dem.
- Afspilninger, der blev tilknyttet igen til en omdøbt titel under en læsning af biblioteket, blev ikke tjekket for fælles visning før næste genstart; nu bliver de det med det samme.

## [1.6.5] - 2026-09-24

### Added
- **Der tages automatisk en fuld sikkerhedskopi før hver opgradering.** Første gang en nyere FinStats åbner din database, skriver den en komplet kopi af den (hele din historik, indstillinger og det hele) til `data/pre-update-backups/`, *før* den ændrer noget. Hvis en opgradering nogensinde ødelægger noget, er dine data i sikkerhed: stop FinStats, læg kopien tilbage i stedet for `finstats.db`, start den version, du havde, og rapportér fejlen. Intet går tabt. De nyeste kopier beholdes; sæt `FINSTATS_SKIP_PREUPDATE_BACKUP=1` for at slå det fra, hvis der er knap diskplads.

## [1.6.4] - 2026-09-24

### Added
- **Fem nye steder at sende notifikationer til**: Telegram, Slack, Pushover, Pushbullet og e-mail, ved siden af de modtagere til webhook, Discord, ntfy og Gotify, der allerede fandtes. Hver beder kun om det, den har brug for (et chat-id, en brugernøgle, en postkasse), og de tre, der hører til én tjeneste (Telegram, Pushover, Pushbullet), kan kun nås der, så et token aldrig kan sendes til en vært, der bare ligner. Mail sendes over TLS fra første byte eller med `STARTTLS`, og der er ingen tredje mulighed.
- **Licenser** under **Indstillinger**: FinStats’ egen GPL og hver tredjepartslicens, programmet er bygget på (253 Rust-pakker, de to medfølgende skrifttyper, kortets omrids og geolokationsdatabasen), hver tekst i sin helhed, som dens eget projekt skrev den. Vælg en komponent, så åbner dens licens i vinduet ved siden af listen; en pakke, der leveres med flere filer, skifter mellem dem på stedet. Listen dannes ud fra de pakker, FinStats faktisk er bygget af, så en afhængighed ikke kan tilføjes, uden at dens licens bliver registreret.

## [1.6.3] - 2026-09-24

### Fixed
- Sikkerhedsvagten griber nu også ind, når Jellyfin viser *ingen* biblioteker eller *ingen* brugere overhovedet (en ødelagt læsning, aldrig en almindelig dag), hvilket justeringen i 1.6.2 havde ladet slippe igennem. At miste nogle biblioteker regnes stadig som almindeligt; at miste dem alle på én gang gør ikke.

## [1.6.2] - 2026-09-24

### Fixed
- Vagten for dataintegritet, der kom i 1.6.1, var for ivrig: et bibliotek, der virkelig mistede de fleste af sine elementer (for eksempel en håndfuld klip, hvis filer forsvandt), blev taget for en ødelagt læsning, så FinStats nægtede at opdatere det og stoppede. Nu griber den kun ind ved en *tydeligt* ødelagt læsning: et bibliotek, der er stort nok til at betyde noget, og som kommer helt tomt tilbage, eller et stort bibliotek, der er tømt til næsten ingenting. Almindelige ændringer gennemføres som før, og tilfældet »Jellyfin returnerede ingenting« fanges stadig.

## [1.6.1] - 2026-09-24

### Added
- **FinStats stoler ikke længere på en læsning fra Jellyfin, der ville slette din historik.** Hvis et bibliotek, som FinStats har tusindvis af elementer for, pludselig kommer tomt eller næsten tomt tilbage (en Jellyfin-opgradering, der ændrede API’et, en fejl forklædt som et tomt resultat), beholder FinStats de data, det har, markerer synkroniseringen som mislykket (og giver besked) og stopper pænt med en tydelig meddelelse i stedet for at markere alt som fjernet. Genstart, når Jellyfin er sig selv igen, eller sæt `FINSTATS_ALLOW_LIBRARY_SHRINK=1`, hvis du virkelig tømte et bibliotek. Den samme vagt dækker listen over biblioteker og listen over brugere.

### Security
- En `X-Forwarded-Host`-header sendt af en klient kunne gå god for en fremmed `Origin` og få en skrivning på tværs af websteder forbi same-origin-vagten. Den header stoler FinStats nu kun på bag en betroet proxy (`FINSTATS_TRUST_PROXY`), den samme regel, som opslaget af klientens adresse allerede følger. Sessionscookien med `SameSite=Lax` var og forbliver det første forsvar.
- En personlig modtager af notifikationer kunne rettes mod `0.0.0.0` eller `::`, som når denne maskine. Kontrollen, der holder personlige modtagere rettet mod offentlige adresser, regner nu også disse som lokale sammen med loopback og de private adresseområder.

### Fixed
- Siden **Afspilning** fejler ikke længere for et bibliotek med en afspilning uden registreret afspilningsmåde (en række gendannet fra en sikkerhedskopi, der er ældre end kolonnen); sådan en afspilning tæller simpelthen som direkte afspilning.
- Et meget langt ord i en søgning eller et filter giver ikke længere en serverfejl: SQLite afviser et `LIKE`-mønster på over 50.000 tegn, så filterord afkortes til en fornuftig længde før forespørgslen.
- Siden **Indstillinger** ruller ikke længere sidelæns på en telefon, når et kort melder en hel sætning tilbage, som resultatet af en gendannelse.

## [1.6.0] - 2026-09-24

Notifikationer

### Added
- Kortet **Notifikationer** under Indstillinger: tilføj en modtager, sæt flueben ved det, den skal høre om, og tryk på Test. Ingen modtager betyder, at der ikke sendes nogen forespørgsel til noget.
- Fire slags modtagere: en almindelig **webhook** (JSON, dokumenteret i `docs/api.md`, valgfri `Authorization`-header), **Discord** (en embed via en kanals webhook-URL), **ntfy** (ntfy.sh eller din egen, til et emne) og **Gotify** (din egen server, med et applikationstoken). Titler sendes i selve beskeden og ikke i en header.
- Elleve slags hændelser, hver med sit eget flueben: umulig rejse, en konto set i et nyt land, en bølge af mislykkede logins, en opgave, der mislykkedes, en sikkerhedskopi, der ikke kunne skrives, noget, der holdt op med at svare (Jellyfin, Sonarr, Radarr eller Seerr), og når det svarer igen, nye titler i biblioteket (samlet pr. serie og dag, som hylden på Oversigt samler dem), en anmodning, der blev klar til at blive set, og en afspilning, der begynder eller slutter. De to sidste er slået fra som standard.
- En anmodning, der blev klar til at blive set, nævner titlen, hvem der bad om den, og hvor længe de ventede.
- Personlige modtagere: giv **Få notifikationer**, så kan personen tilføje sin egen, der kun får det, vedkommende allerede må se: egne anmodninger og advarsler altid, andres med *Se alles aktivitet*, adresser med *Se netværksdetaljer*. En personlig modtager skal pege på en offentlig adresse; kun en administrator må rette en ind i det lokale netværk.
- **Medtag IP-adresser og steder**, pr. modtager og slået fra som standard: ellers nævner en besked om umulig rejse kun steder (»Oslo, Norge og London, Storbritannien«).
- **Sendt for nylig** i samme kort: hvad der blev sagt, til hvilken modtager, og hvad den anden side svarede, når beskeden ikke kom frem. En mislykket besked prøves igen fire gange i løbet af omkring en time, og der ventes på en modtager, der begrænser antallet af beskeder.
- **FinStats’ adresse**, en indstilling for administratorer, er det, en notifikations link bygges ud fra. Tom betyder, at beskederne ikke har noget link.
- Jellyfins egne planlagte opgaver på siden Server: hvad hver af dem gør, i ord i stedet for navnet i koden, og om den er en af de tunge. En opgave, der kører, viser en fremskridtsbjælke, en procent og hvor meget der er tilbage; resten viser, hvornår de kører næste gang, hvornår de sidst kørte, og om det gik godt. Opgaver, som Jellyfin skjuler på sit eget kontrolpanel, er med, og intet i FinStats kan starte eller stoppe en opgave.
- Hvor meget der er tilbage af en opgave, måles ud fra den hastighed, procenten faktisk har bevæget sig med, sammenholdt med den nyeste aflæsning, der ligger langt nok tilbage til at sige noget, i stedet for gennemsnittet af hele kørslen, så en opgave, der bliver hurtigere eller langsommere, beskrives med det tempo, den har nu.
- De opgavelister, FinStats allerede læser af andre grunde (tjekket af biblioteksscanningen hvert 5. minut, serverdetaljerne hvert 15.), fodrer den samme måling, så en opgave, der har kørt en time, som regel har et estimat, i det øjeblik siden åbner. Jellyfin bliver ikke spurgt om noget ekstra for det.
- En kørende opgave, hvor intet er målt endnu, viser **ETA** og en ellipse, der kører i ring, i stedet for et tal, og en opgave, hvis procent ikke har flyttet sig i et halvt minut, viser »68 % i 4 min« ved siden af. Intet træder i stedet for et estimat, der ikke er målt.
- En liste over alle versioner ved siden af ændringsloggen: ét klik fører til en hvilken som helst version, den markerer den, du læser, og på en smal skærm bliver den til en række af serier over noterne.

### Changed
- Ændringsloggen foldes ikke længere sammen. Hver serie står åben, og versionslisten er måden at komme rundt på i stedet for én gruppe åben ad gangen og et klik for at nå en anden.
- **Indstillinger → Udgående forbindelser** viser hver modtager af notifikationer sammen med Jellyfin og forbindelserne til tjenester: hvad den er, dens vært, om den er slået til, og hvornår den sidst tog imod en besked.
- Intet ved indsamlingen, historikken eller noget tal på skærmen ændrer sig. FinStats *læser* stadig kun fra Jellyfin, Sonarr, Radarr og Seerr.

## [1.5.2] - 2026-09-22

### Changed
- En læsning af Jellyfins sessionsliste og nulstillingen af det ur, der bad om den, er ét kald, så de to ikke længere kan komme ud af trit. Det var formen på den fejl med løbske forespørgsler, der blev fundet, mens 1.5.0 blev bygget.
- At bevise det tager nu en brøkdel af et sekund i stedet for syv minutter mod en rigtig server, så det tjekkes, hver gang noget ændres. Samme forespørgsler, samme historik, samme tal på skærmen.

## [1.5.1] - 2026-09-22

### Fixed
- En omkodning registreres én gang, ikke én gang i sekundet. Når en afspilning starter med omkodning og falder tilbage til direkte afspilning, lader Jellyfin omkodningsdetaljerne blive på sessionen; hver aflæsning blev sammenlignet med en værdi, FinStats lige havde overskrevet, så hver aflæsning lignede en ændring, og siden Aktivitet blev fyldt med »Omkodning«-linjer med »Direkte afspilning: …« nedenunder.
- Linjer, der allerede er skrevet, ryddes op ved start: hvor flere i træk siger det samme, beholdes den første. En afspilning, der krævede omkodning, tæller stadig som en omkodning.

## [1.5.0] - 2026-09-22

Sessionssporing over WebSocket

### Changed
- FinStats holder én forbindelse åben til Jellyfin, fra det øjeblik det starter, og det er sådan, det hører om en afspilning. I 1.4.0 kom det som en indstilling, slået fra som standard, for at bevise sig mod rigtige servere; nu er det altid slået til. De samme afspilninger, pauser og spring, beregnet af den samme kode.
- Intet afspilles: FinStats spørger slet ikke Jellyfin om noget. En aften, hvor ingen ser noget, koster ingen forespørgsler.
- Noget afspilles: sessionslisten læses én gang i sekundet. Det er det, der giver en pause, en spoling eller et sporskift sin præcision, og det er det, der afslutter en afspilning, hvis klient forsvandt uden at sige farvel.
- Alt, der er indlæst, har stået på pause i tre aflæsninger i træk: spørgsmålene stopper, og forbindelsen tager over. Før kostede en film på pause det samme spørgsmål hvert sekund, omkring 16 MB i timen. Genoptages noget, eller starter en anden noget, fanges det inden for omkring et sekund, og tid på pause tæller stadig aldrig som visningstid.
- Hver udgående læsning beder om komprimerede svar og pakker dem ud (Jellyfin, Sonarr, Radarr og Seerr komprimerer alle, når de bliver bedt om det), hvilket skærer mere end halvdelen af de to største læsninger: sessionslisten under en afspilning og læsningen af biblioteket. Geolokationsdatabasen downloades stadig, som den er gemt.
- Jellyfin-kortet i Indstillinger viser, hvilken af de to halvdele der kører, og live-prikken følger, om forbindelsen bærer, ikke hvilken halvdel du er i.
- Kun én af de to kører ad gangen: mens FinStats spørger, beder det Jellyfin om at holde op med at sende og beder den om at begynde igen, i samme øjeblik den sidste afspilning slutter. Forbindelsen holdes åben hele tiden.
- En Jellyfin, der svarer på abonnementet med ingenting, bliver troet, så længe den svarer på keep-alives: FinStats siger det én gang, læser selv sessionslisten imens og forbliver abonneret, indtil det første push afgør sagen. »Kan ikke pushe« og »var ved at genstarte, da vi spurgte« ser ens ud, så stilhed er aldrig en dom.
- Reserveløsningen er urørt: en forbindelse, der lukker, bliver stille eller viser sig ikke at tale dette, falder tilbage til at spørge med faste mellemrum allerede ved næste gennemløb og bliver ved med at oprette forbindelse igen, med intervallerne **Mens nogen ser med, tjek hvert** og **Mens intet afspilles, tjek hvert**.

### Added
- Hvad indsamleren laver, på `GET /api/status`: hvilken af de fire ting den er i gang med (`idle_socket`, `playing_poll`, `paused_socket`, `fallback`), om forbindelsen er åben og abonneret, hvor tit den spørger, hvor mange sessionslæsninger der gik ud i det seneste minut, og hvornår det sidst ændrede sig. Besvares fra hukommelsen (ingen forespørgsel, ingen databaseforespørgsel) og indeholder ingen navne, titler eller antal.
- De felter holdes op mod hinanden ved hvert gennemløb, og en uoverensstemmelse, der varer ved, skrives i loggen.
- Én begrænser under hver sessionslæsning i processen: højst 2 i sekundet og 70 i minuttet, hvor alt derover bliver udsat, talt og logget højst én gang i minuttet. Almindelig brug holder sig et godt stykke under.

### Fixed
- Liveopdateringer erstatter nu virkelig de gentagne forespørgsler. Når 1.4.0 var slået til, bad den stadig om sessionslisten hvert femte sekund og åbnede forbindelsen igen hvert tyvende til fyrretyvende sekund, fordi den bedømte forbindelsen efter, hvor nylig en liste var kommet, og Jellyfin sender en, når noget *ændrer sig*, og intet derimellem. Nu afgøres det, om forbindelsen lever, af om Jellyfin overhovedet svarer, tjekket med en keep-alive omkring to gange i minuttet.
- En sessionsliste, der kom, mens FinStats var mellem to gennemløb, blev smidt væk ud fra en antagelse om, at en ny var halvandet sekund bagefter. Den gemmes og bruges.
- **Sidst tjekket** under Indstillinger og kortet Udgående forbindelser bliver ikke længere forældede, mens forbindelsen er stille: Jellyfins svar på en keep-alive er det, de viser.

### Removed
- Indstillingen **Lad Jellyfin sende, hvad der afspilles** under Indstillinger → Indsamling og `live_socket` fra indstillings-API’et.
- `socket_enabled` fra indsamlerens status: der findes ikke længere noget »slået fra«.
- `last_reconcile_at` fra indsamlerens status, erstattet af `socket_live`.

## [1.4.1] - 2026-09-22

### Changed
- Seerr læses kun i sin helhed, når noget er ændret: hvert femte minut beder FinStats om én række, den senest ændrede anmodning, og gennemløbet slutter der, hvis det allerede kender den. Det er omkring en kilobyte mod omkring tres for en side med halvtreds. Anmodninger, der stadig er på vej, kigges på hvert kvarter, alt listes én gang i døgnet, og en ny anmodning dukker stadig op inden for fem minutter.
- Køerne i Sonarr og Radarr læses hvert femte sekund, mens en side viser dem, hvert minut, mens der er noget i køen, og hvert femte minut, mens den er tom, i stedet for hvert minut døgnet rundt. At åbne siden eller forbinde en tjeneste opdaterer dem med det samme, og det samme gør en ny anmodning i Seerr.

## [1.4.0] - 2026-09-21

Valgfri WebSocket-sessioner og alt, hvad FinStats kan nå

### Added
- **Lad Jellyfin sende, hvad der afspilles** under Indstillinger → Indsamling, slået fra, indtil du slår det til: én forbindelse bærer sessionslisten, og omkring 17.000 forespørgsler i døgnet bliver til næsten ingen. Afspilninger, pauser, spring og grupper beregnes af præcis den samme kode, som beregnede dem før. Mens den er aktiv, laver FinStats stadig én almindelig forespørgsel i minuttet, *mens noget afspilles*, og ingen, når intet gør. En forbindelse, der lukker, er stille i femten sekunder eller viser sig ikke at tale det, falder tilbage til de faste intervaller ved næste gennemløb og bliver ved med at oprette forbindelse igen.
- **Udgående forbindelser**, et kort i Indstillinger for Jellyfin-administratorer: hvert sted, FinStats kan nå (din Jellyfin, tjenesten »hvad er min IP«, DB-IP’s database og hver Sonarr, Radarr eller Seerr), med hvad den bruges til, om den er slået til, og hvornår den sidst svarede. Bygget ud fra det, FinStats allerede ved, og det viser kun adresser: aldrig en nøgle, aldrig en grundsti.
- **Slå op nu** under Indstillinger → Hjemmenetværk, til den dag din offentlige adresse ændrer sig.

### Changed
- Opslaget »hvad er min IP« kører én gang, første gang FinStats har brug for en adresse, i stedet for hvert kvarter.
- Jellyfin-kortet i Indstillinger viser, hvordan indsamleren får besked, og, når forbindelsen er slået til, men ikke bærer, hvorfor ikke.

## [1.3.0] - 2026-09-20

Sonarr, Radarr og Seerr på én side

### Added
- **Forbindelser** under Indstillinger, for Jellyfin-administratorer: **Sonarr**, **Radarr** og **Seerr** (eller Jellyseerr/Overseerr), flere af hver slags, hver testet, før den gemmes. Din downloadklient kræver ingen opsætning, fordi Sonarr og Radarr allerede taler med den, og FinStats læser det, de ved.
- **Anmodninger**: hvem der bad om hvad, hvor lang tid det tog at komme, og om de nogensinde så det. Fliser for titler, der venter, og den typiske ventetid, et diagram over, hvordan det har ændret sig måned for måned, og lister over det, der kom for flere uger siden og aldrig blev afspillet, og hvem der beder om mest.
- **Kommende**: nye episoder og filmudgivelser fra Sonarr og Radarr som en dagsorden pr. dag, markeret med, om *du* ser den serie, og, for dem, der må se alle, hvem der ellers gør. En serie regnes som set, når nogen har afspillet en episode af den inden for de seneste fire måneder. En »Kommer snart«-række på Oversigt, én pr. profil for de serier, den person ser, og hvad der kommer næste gang for en titel på dens egen side.
- **Downloads**: livekøen i hver Sonarr og Radarr (hvad det er, hvor langt det er kommet, hvor hurtigt det går, hvad der gik galt ved import), med den person, der bad om det, ved siden af. En sæsonpakke er én linje, uanset hvor mange episoder den indeholder, og det, der kræver opmærksomhed, står øverst. Under den det, der kom ind over en uge, en måned eller et år efter indekser, kvalitet og downloadklient, og hvilke downloads der mislykkedes. Oversigt viser den samme liste, forkortet.
- **Se downloads**, en ny tilladelse. Uden den ser folk stadig, hvor langt deres *egen* anmodning er kommet, og hvor lang tid der er tilbage, og intet andet: ingen udgivelsesnavne, ingen hastigheder, ingen andre downloads. Andres anmodninger kræver *Se alles aktivitet*.

### Changed
- FinStats sender `GET` til disse tjenester og intet andet: der findes ingen kode i FinStats, der kan godkende en anmodning, starte en søgning eller tilføje, sætte på pause eller fjerne en download.
- Deres API-nøgler gemmes i FinStats’ egen database, sendes aldrig tilbage til browseren, skrives aldrig i en log og er aldrig med i en sikkerhedskopi. Ingen omdirigering følges, så en nøgle ikke kan rejse et sted hen, du ikke har skrevet, og certifikater verificeres, medmindre du slår det fra for en enkelt forbindelse.

## [1.2.2] - 2026-09-20

### Added
- Lyd- og undertekstsprog på hver film og episode (»Lyd: japansk · engelsk«).
- Hvor langt en dubbing går: en serie eller sæson viser, hvor mange af dens episoder der har hvert sprog (»Engelsk: 13 af 26 episoder«), og episodelisten har en **Lyd**-kolonne.
- **Lydsprog** og **Undertekstsprog** på bibliotekssiderne: hvor mange filer der kan afspilles på hvert sprog.
- FinStats læser biblioteket igen én gang efter denne opdatering for at få sprogene med. Det siger aldrig »dubbet«, fordi Jellyfin ikke oplyser en titels originalsprog, og et spor uden sprogmærke vises som »Ukendt«.

## [1.2.1] - 2026-09-20

### Changed
- Billederne i README er taget igen i denne version, med et nyt af siden **Sikkerhed**. Intet i appen ændrer sig.

## [1.2.0] - 2026-09-20

Hvor folk ser fra, på et kort

### Added
- **Siden Sikkerhed**, for dem, der må se netværksdetaljer og alles aktivitet: et verdenskort med et punkt for hvert sted, dine brugere ser fra, større jo mere der sker der. Hjemme er grønt, steder ude er lilla, igangværende streams pulserer, og mislykkede logins udefra er røde. Træk for at flytte, zoom med knapperne, tasterne eller Ctrl + hjul, og punkter, der overlapper, smelter sammen, indtil du zoomer ind. Under kortet hvert sted med dets personer, afspilninger, visningstid og logins, og landene efter antal afspilninger.
- **Umulig rejse**: en advarsel, når én konto ses to steder, som intet fly forbinder, eller to steder langt fra hinanden på samme tid, med begge observationer, afstanden, tiden imellem dem og den hastighed, det ville have krævet. **Vis på kortet** tegner rejsen.
- **Nyt land**: første gang nogen afspiller eller logger ind fra et land, de ikke er set i før.
- Markér en advarsel som løst med en note, eller dæmp et par steder for den person, til en VPN eller en telefon, hvis operatør holder til i hovedstaden. Advarsler, der findes i gammel historik, efter en import eller i en ny database, lægges væk som løste i stedet for at oversvømme listen. Advarslerne og det, du har besluttet om dem, er med i sikkerhedskopier.
- Stederne kommer fra en bydatabase, der læses lokalt: ingen adresse sendes nogensinde nogen steder hen, og kortet tegnes ud fra omrids, der følger med FinStats, ikke fra en korttjeneste. Download DB-IP’s gratis database fra siden Sikkerhed eller **Indstillinger → Sikkerhed**, lad FinStats opdatere den hver måned (slået fra, indtil du slår det til), eller læg din egen `.mmdb` (DB-IP, MaxMind GeoLite2-City) i `data/geoip/`. `FINSTATS_GEOIP_DB` angiver en fil et andet sted.
- **Indstillinger → Sikkerhed**: den database, der bruges, den månedlige opdatering, og hvor hurtigt (900 km/t) og hvor langt fra hinanden (500 km) to observationer skal være for at tælle som umulig rejse. Bydatabaser rammer ofte et par hundrede kilometer ved siden af, og det er det, afstanden er til for.

## [1.1.3] - 2026-09-20

### Added
- Ctrl+Space søger også blandt de medvirkende i alt i dit bibliotek, under **Medvirkende**, med deres foto, om de spiller med eller instruerer, og i hvor mange af dine titler de er med; vælger du én, åbnes deres side. Lige så tilgivende som titelsøgningen (ord i vilkårlig rækkefølge, accenter ignoreres, en slåfejl tilladt), og når flere matcher lige godt, kommer den, der er med i flest af dine titler, først.

## [1.1.2] - 2026-09-20

### Added
- **Tilføjet for nylig** på Oversigt, over diagrammet Aktivitet: de 30 nyeste tilføjelser som en række plakater, du kan rulle sidelæns, med pile på en computer. Hver viser, hvornår den kom, og hvad den er (en film, et album eller »Sæson 4 · 3 episoder«), og en enkelt ny episode viser hvilken, med sæsonens egen plakat, når den har en.
- **Skift + musehjul** og piletasterne flytter én plakat, Page Up og Page Down en skærm ad gangen, Home og End går til hver sin ende, og man kan swipe. Det almindelige hjul ruller stadig siden.
- Nye episoder af den samme serie, der kommer samme dag, deler én post, og en hel serie, der tilføjes på én gang, er én post (»3 sæsoner · 60 episoder«), så én stor import ikke skubber alt andet ud af listen.
- Rækken ser bort fra Oversigts tidsrum og person, da den handler om biblioteket og ikke om afspilninger, og skjuler sig, så længe biblioteket er tomt.

## [1.1.1] - 2026-09-20

### Added
- Sider åbner med det samme: et øjeblik efter, at FinStats er indlæst, henter det i stilhed det, du sandsynligvis åbner næste gang (Oversigt, din profil og tidslinje, Aktivitet, Brugere, Biblioteker og hvert bibliotek, Afspilning, serversiderne, tilbageblikket og de mest aktive personer), så en side vises med det samme, når du åbner den, og stadig opdateres i baggrunden.
- Film, serier og medvirkende hentes kun, når du viser interesse, da der er tusindvis af dem: at lade markøren hvile på et link, røre ved det eller nå det med tastaturet henter den ene side.
- Det venter, til den side, du åbnede, er færdig med at indlæse, tager én side ad gangen, mens browseren er ledig, holder pause i en fane i baggrunden, gør intet på en datasparende eller meget langsom forbindelse og glemmes, når du logger ud.
- Et lille **Repo**-link med GitHub-logoet i statuslinjen ved siden af versionen.

### Fixed
- **Tidslinje** glemte, hvilke biblioteker der var slået fra, når siden blev genindlæst eller åbnet fra et bogmærke, selv om valget stod i adressen.

## [1.1.0] - 2026-09-20

En tidslinje over alt, hvad du har set

### Added
- **Tidslinje**, ved siden af **Oversigt** på hver profil: det, du har set, som ét spor fra i dag og tilbage til den første afspilning, FinStats kender til, med en aften med episoder samlet i ét stop. Hvert stop er en plakat med, hvad der blev set, og hvornår (»Sæson 2 · Episode 3–6«, en film og om den tog mere end én omgang, eller et album og hvor mange spor), og åbner titlen.
- Sporet følger vinduets bredde: tre stop pr. række på en bred skærm, to på en smallere, hver anden række løber baglæns med et sving, der forbinder den med den næste, helt ned til én lige linje på en telefon. Det første stop i hver måned bærer måneden.
- Ældre stop indlæses af sig selv, mens du ruller, helt tilbage til der, hvor historikken begynder.
- **Biblioteker**-kontakterne over sporet udelader det, du ikke vil se, for eksempel musik, og valget er en del af adressen.
- Noget, der stadig afspilles, viser **Afspilles nu**.
- Du ser din egen tidslinje; andres kræver tilladelsen »Se alles aktivitet«.

## [1.0.5] - 2026-09-20

### Added
- Beskyttelse mod nedgradering: databasen husker den nyeste FinStats-version, der har åbnet den, og en ældre version nægter at starte på den i stedet for i al stilhed at arbejde med data, den ikke helt forstår. Meddelelsen siger, hvad du skal gøre. Versioner til og med 1.0.4 udkom, før dette tjek fandtes.

## [1.0.4] - 2026-09-20

### Fixed
- I en afspilnings detaljer var kopiknappen ved siden af et langt **Enheds-id** faldet ned på en linje for sig.
- På siden Afspilning blev den sidste kolonne i **Hvilke klienter der omkoder** skåret af (»Omkod…«); overskrifter på to ord ombrydes nu.

### Changed
- Skærmbillederne i README er aktuelle igen; de viste stadig version 0.5.0.

## [1.0.3] - 2026-09-20

### Added
- En episodeside viser episodens eget billede, det, Jellyfin viser i sin episodeliste, i stedet for seriens plakat. En episode, som Jellyfin ikke har noget billede af, beholder plakaten.

## [1.0.2] - 2026-09-20

### Fixed
- I **Ændringslog** manglede grupperne v0.9 og v0.10 en overskrift ved siden af deres version. En gruppes overskrift er nu altid den første sætning om, hvad dens første version handlede om, så en lang indledning ikke skæres af midt i en sætning.

## [1.0.1] - 2026-09-20

### Fixed
- Det udgivne image ville ikke starte på en ny maskine: »unable to open database file: /data/finstats.db«, efter at have hængt i et halvt minut. Docker opretter en manglende `data`-mappe som root, og FinStats kører som en almindelig bruger. Containeren gør nu mappen til sin egen og skifter derefter til den bruger, så `docker run` virker i første forsøg. Sæt `PUID` og `PGID` for filer, der ejes af en anden end bruger 1000; `--user` virker som før.
- En datamappe, der virkelig ikke kan skrives til, meldes med det samme, med den kommando, der løser det, i stedet for 30 sekunder senere som en databasefejl.

## [1.0.0] - 2026-09-20

Klar til alle

### Added
- Et færdigt image, nu på `ghcr.io/finstats/finstats`, til 64-bit Intel/AMD- og ARM-maskiner (en Raspberry Pi 4 eller 5 virker): `:latest` er den nyeste version, et versionsmærke bliver, hvor det er, og `:edge` er udviklingsversionen.
- Hver version vises på projektets udgivelsesside på GitHub med denne ændringslog.

### Changed
- Versionsnumre betyder nu det, de siger: opdateringer i 1.x ødelægger ikke dine data, dine sikkerhedskopier eller dine indstillinger.
- Docker Compose-filen og alle vejledninger bruger det udgivne image. Et selvbygget `finstats:latest` kræver kun, at imagenavnet ændres; `data`-mappen følger med uændret.

## [0.10.2] - 2026-09-20

### Fixed
- På en telefon kunne Indstillinger og en series side rulle sidelæns: en etiket, der kun var ment for skærmlæsere, slap ud af sin tabel, og episodelisterne og rækken med medvirkende gjorde det samme. Nu ruller de inde i sig selv.
- Tilbageblikket ruller ikke længere sidelæns, når et kapitels baggrundsord er bredere end siden.
- En lang »ser på …«-linje under **Afspilles nu** slutter med en ellipse i stedet for at blive skåret af midt i et bogstav.
- Plakaten foran hver serie på en profil var et ekstra, navnløst link for skærmlæsere; den er skjult for dem, og titlen ved siden af er linket.

## [0.10.1] - 2026-09-20

### Fixed
- Download af en sikkerhedskopi mislykkedes i browseren (»the source file could not be read« i Firefox), uden at noget på siden sagde hvorfor: FinStats komprimerede en sikkerhedskopi, der allerede er komprimeret, en gang til, og den dobbeltpakkede overførsel brød af før slutningen. Sikkerhedskopier sendes, som de er, med deres størrelse oplyst på forhånd, så browseren kan vise det reelle fremskridt.

## [0.10.0] - 2026-09-20

Sikkerhedskopier og flytning til en ny installation

### Added
- En sikkerhedskopi hver uge, de fem nyeste beholdes: hver afspilning med sin tidslinje over pauser og spring, markeringer som set, tilladelser, hjemmeadresser, serverloggen og dine indstillinger, i én lille fil, nogle hundrede kB for tusindvis af afspilninger. **Indstillinger → Sikkerhedskopier** viser dem med en **Download**-knap til hver, laver en efter behov og bestemmer, hvor ofte og hvor mange. De ligger i mappen `backups` i din datamappe. En sikkerhedskopi indeholder ingen API-nøgle til Jellyfin og ingen loginsession, men den er en komplet visningshistorik med IP-adresser, så opbevar den et privat sted.
- Gendannelse fletter i stedet for at overskrive: afspilninger, der allerede findes, springes over, så det er sikkert at gøre to gange eller ind i en installation, der har kørt et stykke tid. Fjern fluebenet ved »Gendan også indstillinger og tilladelser« for kun at få historikken. Sikkerhedskopier virker på tværs af versioner i begge retninger. Fra en terminal: `finstats backup` og `finstats restore <file>`.

### Changed
- Indsamlingen tjekker hvert sekund, mens nogen ser med, og hvert 5. sekund, mens ingen gør, hvor den før tjekkede hvert 5. sekund hele tiden, så pauser, spring og sporskift registreres på sekundet. Begge findes under **Indstillinger → Indsamling**; et ændret gammelt interval skal indstilles igen.

### Fixed
- En IP-adresse i en afspilnings detaljer blev delt over to linjer midt i adressen. Mærket Lokal/Udefra flytter ned under adressen, når der ikke er plads.

## [0.9.1] - 2026-09-20

### Fixed
- Folk derhjemme blev vist som udefra. En enhed på dit eget netværk, der når Jellyfin via dets offentlige navn, for eksempel gennem en reverse proxy eller et domæne, kommer med husstandens offentlige IP-adresse, og kun private adresser talte som lokale. Nu lærer FinStats den offentlige adresse og regner de afspilninger som lokale i hele historikken, og det husker tidligere adresser.

### Added
- **Indstillinger → Hjemmenetværk**: de adresser, FinStats regner som hjemme, hvor du kan tilføje dine egne (en tidligere adresse, et andet hjem, en VPN-udgang) eller slå opslaget fra. Opslaget spørger en simpel »hvad er min IP«-tjeneste hvert kvarter og prøver flere (Amazon, Cloudflare og andre), fordi reklameblokerende DNS som Pi-hole ofte blokerer dem, og en af dem kræver slet ingen DNS. Forespørgslen indeholder intet om dig eller din server, og med kontakten slået fra taler FinStats ikke med andet end Jellyfin. `FINSTATS_PUBLIC_IP_URL` angiver en tjeneste efter eget valg.

## [0.9.0] - 2026-09-20

Hver tabel i den rækkefølge, du vil have

### Added
- Hver tabel kan sorteres: klik på en overskrift for at sortere efter den, igen for at vende rækkefølgen og en tredje gang for den oprindelige rækkefølge. Cellernes betydning forstås, så »3 d 2 t« sorteres som tid og »1,4 GB« som størrelse, og tomme celler kommer sidst. Det gælder brugerlisten, seerne, episoder, enheder, plugins, planlagte opgaver, tabelvisningen af hvert diagram, tilladelsesmatricen og søjlelisterne (codecs, opløsninger, klienter og resten), som får deres egen slanke overskrift.
- Aktivitet og serverloggen sorterer på tværs af alle deres sider i stedet for kun rækkerne på skærmen, og rækkefølgen gemmes i adressen, så den overlever en genindlæsning eller et delt link.
- Tabeller med ti rækker eller flere får et lille filterfelt over sig.

## [0.8.2] - 2026-09-20

### Changed
- Tilbageblikkets årsvælger er den samme kontakt som alle andre vælgere af tidsrum i FinStats (7 d · 30 d · 90 d …).

## [0.8.1] - 2026-09-19

### Added
- Sider for skuespillere og instruktører, fra tilbageblikkets »Mest sete personer« eller den nye **Medvirkende**-række på hver film, serie og episode: alt, hvad de er med i på din server, hvor meget der er set og af hvem, og hvad der stadig venter. Siden følger det tidsrum, du vælger, folk, der kun må se deres egen statistik, ser kun det, de selv har set, der, og Esc fører dig tilbage.

## [0.8.0] - 2026-09-19

Tilbageblikket har fået et nyt udseende og mere at fortælle

### Added
- **Mest sete personer**: de skuespillere, du brugte mest tid sammen med, og instruktørerne bag det, du så, som en række portrætter. Medvirkende læses sammen med biblioteket og dukker op efter den første læsning af biblioteket efter denne opdatering.
- **Dit år i dage**: hver dag i året som ét felt, lysere jo mere du så, med din længste stime og din største dag ovenover. Hold markøren over en dag, eller gå gennem kalenderen med piletasterne, for at se, hvad den indeholdt.
- **I tal**: afspilninger, visningstid, forskellige titler, dage med visning, længste stime og hvor meget der var **gensyn**, plus hvordan året fordeler sig mellem episoder, film og musik.
- Genrekapitlet siger, hvor mange genrer du var omkring, og hvor stor en andel den største tog.

### Changed
- Et nyt design: det åbner med dit år »afspillet igen«, overskriften ved siden af de plakater, der fyldte det, over en bølgeform af året med én søjle pr. uge, hvor de kraftigste lyser, og du vælger året lige ovenover. Hvert kapitel har en overskrift med sit nøgleord i violet, en sætning, der bærer tallene, og kapitlets ord stående stort og svagt bagved. Det slutter, som film gør, med rulleteksterne: medvirkende, instrueret af, vist på, spilletid.
- Timer, ugedage og måneder deler ét kapitel, **Aktivitetsmønstre**, med en kontakt imellem dem, og overskriften følger det, du kigger på (»Fredag tog kronen«).

### Fixed
- **Afspilles nu** på Oversigt stod tom, når ingen så noget, så overskriften så ud til at høre til filtrene under den. Dens boks med »Intet afspilles lige nu« er tilbage.

## [0.7.10] - 2026-09-19

### Added
- Biblioteksgrafik: hvert bibliotekskort, og bibliotekets egen side, viser det billede, Jellyfin bruger til det, gennem FinStats’ billedproxy ligesom plakaterne. Et bibliotek uden billede beholder sit ikon.

## [0.7.9] - 2026-09-19

### Changed
- Kortet Genrer åbner som **liste** igen, med radaren som det andet valg på kontakten. Et valg, du allerede har truffet, beholdes.

## [0.7.8] - 2026-09-19

### Added
- En radarvisning til kortet Genrer på Oversigt og på profiler: én eger pr. genre, med et skift mellem **Radar** og **Liste** i kortets hjørne, og valget huskes. Hold markøren over et punkt, eller brug piletasterne, for at se genrens visningstid og andel.

## [0.7.7] - 2026-09-19

### Fixed
- Film, der hører til en samling, blev vist som »Ikke længere i biblioteket«. Med *gruppér film i samlinger* slået til udleverer Jellyfin samlingen i stedet for filmene i den, så FinStats så dem aldrig; på én server skjulte det 556 af 1.238 elementer i filmbiblioteket. Nu beder FinStats om selve filmene, og den læsning, som denne opdatering udløser, bringer dem tilbage og tilknytter deres afspilninger igen.

## [0.7.6] - 2026-09-19

### Changed
- På profiler er kortet **Serier** rykket ned lige over Genrer, så en lang liste af serier ikke længere skubber visningstiden, aktivitetsdiagrammet og toptitlerne ud af syne. Stimerne af dage bliver øverst, og en serie, du har åbnet, forbliver åben, når du ændrer tidsrummet.

## [0.7.5] - 2026-09-19

### Changed
- Søgningen matcher ord for ord, i vilkårlig rækkefølge og med hvad som helst imellem, i stedet for at kræve den præcise frase, så »alya hides« finder *Alya Sometimes Hides Her Feelings in Russian*. Store og små bogstaver, accenter og tegnsætning er ligegyldige, et indledende »The« ignoreres, og en slåfejl tilgives (»mentalsit« finder *The Mentalist*); ord på tre bogstaver eller færre skal stadig skrives rigtigt. Bedre match rangeres først, og musik findes også via kunstneren.
- Søgefelterne i Aktivitet og serverloggen matcher ord for ord på tværs af alle deres kolonner: »alya opera« finder afspilninger af den serie i Opera.

## [0.7.4] - 2026-09-19

### Changed
- Ændringsloggen er grupperet efter serie: hver `0.x`-serie er én sammenfoldelig gruppe, der rummer dens versioner, hvad serien handlede om, hvor mange der er, og hvornår. Kun den nyeste starter åben, og en lukket gruppe viser stadig, hvilken version du kører.

## [0.7.3] - 2026-09-19

### Fixed
- Uret i Afspilles nu tøvede med få sekunders mellemrum: opdateringen hvert 5. sekund tegnede det også igen, så hver gang serveren var en brøkdel foran, flyttede tallet sig for tidligt og stod så stille i et sekund. Nu er det kun tikket én gang i sekundet, der flytter det, med præcis ét sekund (målt 999 til 1.001 ms fra hinanden), og opdateringen retter stadig til efter en pause, et spring eller reel afdrift og lander på tikket.

## [0.7.2] - 2026-09-19

### Added
- At se sammen, live: hvert kort i Afspilles nu siger »Med maria«, mens folk ser det samme sammen, hvor en gruppe før først blev genkendt, når afspilningerne var slut. Live regnes folk som sammen, når de er på samme titel og enten startede inden for gruppevinduet eller er på næsten samme position, hvilket stadig virker, hvis FinStats blev genstartet midt i en stream.

## [0.7.1] - 2026-09-19

### Changed
- Afspilles nu tæller hvert sekund: uret tikker, og bjælken glider ind imellem i stedet for at hoppe, hver gang serveren bliver spurgt. Serveren bliver stadig spurgt hvert 5. sekund og er stadig facit: visningen synkroniseres igen ved en pause, et spring, eller når serveren er foran, men ikke for en position, der blot er et par sekunder gammel, da klienter kun melder omkring hvert tiende sekund. Streams på pause står stille.

## [0.7.0] - 2026-09-19

Hvem der ser sammen

### Added
- At se sammen: når forskellige personer starter samme titel på samme tid, regner FinStats det som at se sammen. Et **Set sammen**-kort på Oversigt viser grupperne, de titler, de deler, og deres timer sammen, hver profil viser, hvem den person oftest ser sammen med, delte afspilninger har et lille personmærke i Aktivitet, og afspilningens detaljer siger, hvem den blev set sammen med. Det virker på hele din historik, importerede afspilninger medregnet.
- En indstilling for, hvor tæt starterne skal ligge, 60 sekunder som standard: omkring en tredjedel af ægte gruppesessioner starter 6 til 60 sekunder fra hinanden. Folk skal også blive ved med at se side om side et par minutter, så to personer, der tilfældigvis åbner den samme episode, ikke er en gruppe.

### Changed
- Søgningen åbnes med **Ctrl + Space** i stedet for Ctrl/⌘ + K.

## [0.6.1] - 2026-09-19

### Added
- Administratorer kan åbne en anden persons tilbageblik, øverst på siden Tilbageblik eller med **Åbn tilbageblik** på vedkommendes profil. Det omtaler så personen ved navn i stedet for »du«.

### Changed
- Alle andre ser stadig kun deres eget tilbageblik, uanset hvilke tilladelser de har, og der er stadig intet tilbageblik for hele serveren.

## [0.6.0] - 2026-09-19

Din profil, og hvor du er i hver serie

### Added
- Fremskridt i serier på profiler: hver serie, du har været inde på, som en bjælke med ét segment pr. episode (set, påbegyndt eller endnu ikke), der kan åbnes sæson for sæson, sorteret efter hvor tæt du er på at blive færdig, med separate visninger for færdige serier og alt.
- **Markér som set** på din egen profil, pr. episode, sæson eller serie, til det, du så, mens intet registrerede. Jellyfins egne markeringer som afspillet bruges også. Markeringer bliver i FinStats og ændrer aldrig noget i Jellyfin; en registreret afspilning kan ikke afmarkeres, men det kan dine egne markeringer.
- Stimer af dage på profiler: din længste række af dage i træk med en afspilning, din nuværende stime, og hvor mange dage du har set noget.
- **Min profil** i sidepanelet.
- Aktivitet viser, hvor afspilningen stoppede (for eksempel `44:15 / 45:00`), under fremskridtsbjælken samt dato og klokkeslæt under »for 3 t siden«. Afspilninger importeret fra Jellystat har ingen stopposition, fordi Jellystat aldrig registrerede en.

### Changed
- Episoder, der mangler eller endnu ikke er sendt, behandles ikke længere som en del af dit bibliotek. Jellyfin viser dem som pladsholdere uden fil, og de talte før med i antallet af episoder.

## [0.5.1] - 2026-09-19

### Added
- Et logo: en afspilningsknap skåret i tre diagramsøjler, i sidepanelet og på loginskærmene, som en skarp SVG med en `favicon.ico` i flere størrelser til browsere og bogmærkelinjer, der vil have en, plus et ikon til hjemmeskærmen på telefoner og tablets.

## [0.5.0] - 2026-09-19

Du bestemmer, hvem der ser hvad

### Added
- Tilladelser under Indstillinger → Adgang, for alle på én gang eller person for person: **se alles aktivitet** (andres statistik og historik, siden Brugere, alle igangværende streams), **se netværksdetaljer** (IP-adresser, enheds-id’er, lokal eller udefra), **se serveren** (siden Server, serverloggen, mislykkede logins, filstier) og **administrer FinStats** (indstillinger, opgaver, import, sletning af afspilninger).
- Login pr. person, så du kan lukke nogle få ind uden at åbne FinStats for alle Jellyfin-brugere.
- Ændringer gælder med det samme, også når adgang tages væk, og alt håndhæves af serveren i stedet for ved at skjule knapper.

### Changed
- Jellyfin-administratorer har stadig alt og er de eneste, der kan ændre tilladelser; den, der må administrere FinStats, kan ikke give noget, hverken til sig selv eller andre.
- Årets tilbageblik forbliver personligt, uanset hvad der gives.
- Intet ændrer sig for eksisterende installationer, før du giver noget.

## [0.4.5] - 2026-09-19

### Changed
- FinStats er licenseret under **GNU General Public License v3.0**. Ingen tidligere version blev nogensinde udgivet, så intet er udgivet under andre vilkår. Licenserne for de medfølgende skrifttyper følger med dem.

## [0.4.4] - 2026-09-19

### Added
- Esc går tilbage: på et åbnet bibliotek, en bruger eller en titel fører den dig tilbage dertil, hvor du kom fra, med listen præcis, som du forlod den. Det, der er åbent, lukkes først (afspilningsdetaljer, søgning, en rulleliste, mobilmenuen), og Esc gør ingenting, mens du skriver eller bruger et diagram med tastaturet.

## [0.4.3] - 2026-09-19

### Changed
- FinStats husker, hvad hver side sidst viste: går du tilbage til en profil, et bibliotek eller et tidsrum, du allerede har kigget på, tegnes siden med det samme, profilbillede og det hele, mens friske tal indlæses bagved, og den tegnes kun igen, hvis noget har ændret sig. Det, der huskes, ligger i browserfanen og smides væk, når du logger ud eller ændrer noget.
- Pladsholdere for indlæsning vises kun, når indlæsningen faktisk er langsom. Før viste hver side dem i mindst en tredjedel af et sekund.

### Fixed
- Biblioteker, der blev slettet i Jellyfin og aldrig havde en afspilning, vises ikke længere. Et slettet bibliotek med historik vises stadig, markeret som fjernet.

## [0.4.2] - 2026-09-19

### Fixed
- På siden Brugere sad mærket **Admin** på sin egen linje og gjorde den række højere end de andre. Mærkerne sidder nu ved siden af navnet.

## [0.4.1] - 2026-09-19

### Fixed
- Ordet »null« vises ikke længere under titlen på Oversigt uden for december og januar, når der ikke er noget tilbageblik-banner at vise.

## [0.4.0] - 2026-09-19

Se, hvad der er ændret, uden at forlade appen

### Added
- Fanen **Ændringslog** med hver version, hvor den version, du kører, er markeret.
- Versionen i statuslinjen linker dertil, og fanen viser en prik efter en opdatering, indtil du har kigget.

### Changed
- Fornyet tilknytning af historik til omdøbte elementer går langt hurtigere: fra 18 sekunder til en tredjedel af et sekund med 666 forældreløse titler, så starten forbliver øjeblikkelig, mens historikken vokser.

## [0.3.3] - 2026-09-19

### Changed
- Biblioteket læses igen lige efter, at Jellyfins opgave **Scan Media Library** er færdig, aldrig på et separat ur og aldrig, mens en scanning kører. FinStats har altid kun læst fra Jellyfin, og det starter aldrig en scanning der.
- En ugentlig læsning som sikkerhedsnet dækker servere, der bruger overvågning i realtid, hvor scanningsopgaven måske aldrig kører.
- Det gamle interval er nu kun en reserve, til når opfølgningen er slået fra, eller serveren ikke melder nogen scanningsopgave.

### Added
- Kontakten **Følg Jellyfins biblioteksscanning** under Indstillinger → Indsamling, slået til som standard.

## [0.3.2] - 2026-09-19

### Fixed
- Historikken overlever omdøbninger. Jellyfin giver en omdøbt fil et nyt id, hvilket før efterlod dens tidligere afspilninger under det rå mappenavn (`Title (2010) [tmdbid-…] [imdbid-…]`) uden plakat. Nu tilknyttes de afspilninger igen til den nuværende post i biblioteket: via TMDB-, IMDb- eller TVDB-id, når det gamle navn indeholder et, ellers via renset titel og år, og episoder via serie plus sæson- og episodenummer. Et match skal være entydigt, så titler, der virkelig er slettet, forbliver, som de er.
- Live-tv-kanaler importeret fra Jellystat blev talt som film, fordi Jellystat ikke registrerer elementtypen. De genkendes nu som live-tv, og nye afspilninger af live-tv registreres med Jellyfins egen type.

### Changed
- Live-tv holdes helt uden for tilbageblikket: toplister, totaler, personlighed, rekorder og placering.

### Added
- `finstats relink` til at tilknytte historik igen efter behov. Den kører også ved start og efter hver læsning af biblioteket.

## [0.3.1] - 2026-09-19

### Changed
- Tilbageblikket er strengt personligt: alle, administratorer medregnet, ser kun deres eget, og visningen »Alle«, brugervælgeren og serverkapitlet er væk.
- Tilbageblikket åbner på det år, der er »klar«: det aktuelle år i december, ellers det foregående. Andre år og »Seneste 12 måneder« er fortsat ét klik væk.

## [0.3.0] - 2026-09-19

Dit år i tilbageblik

### Added
- Fanen **Tilbageblik**, en historie om et år at rulle igennem: timer set, topserier, film, spor og genrer og en seerpersonlighed (natteravn, morgenmenneske, weekendkriger, maratonseer, filmentusiast, musikelsker, vanedyr) med diagrammer over tid på døgnet og ugedag.
- Året måned for måned med den titel, der prægede hver måned.
- Rekorder: største dag, største maraton, længste stime af dage, længste enkelte afspilning, mest gensete titel, årets første afspilning og den ældste titel, der blev set.
- Opdagelser: nye serier påbegyndt, film og episoder set færdige og de serier, der fik præcis én episode.
- Et banner på én linje på Oversigt i december og januar, når et tilbageblik er klar.

## [0.2.0] - 2026-09-19

Mere end Jellystat nogensinde registrerede

### Added
- En tidslinje for hver afspilning: start, pause, genoptagelse, spring, skift af lyd og undertekster, direkte afspilning, der bliver til en omkodning, og stop. Plus antal pauser og spring, hvorfra afspilningen blev genoptaget, og om seeren var på det lokale netværk.
- Fanen **Server**: Jellyfin-version, ventende opdateringer og genstarter, diskforbrug pr. bibliotek (Jellyfin 10.11+), plugins, resultater af planlagte opgaver og hver registreret enhed.
- Indsigt i **Afspilning**: samtidige streams over tid, hvilke klienter der tvinger omkodning, hvor langt folk når, før de stopper, lokale afspilninger mod afspilninger udefra og seervaner.
- Hvad dit bibliotek består af: opløsninger, codecs, dynamisk område, containere, titler pr. årti, tilføjelser pr. måned, de største titler og de titler, ingen nogensinde har set, holdt op mod både FinStats’ historik og Jellyfins egne markeringer som afspillet.
- Oversigt: flest samtidige streams, anslået mængde data streamet, andel af afspilninger udefra, genrer efter visningstid og mislykkede logins.
- Elementsider: links til IMDb og TMDB, studier, bitdybde og billedfrekvens, og hvem der har titlen markeret som afspillet i Jellyfin.
- Brugersider: genrer og de film, episoder og favoritter, som Jellyfin har registreret.

### Fixed
- Tidsetiketten på et kort i Afspilles nu løber ikke længere ud over kortet.
- Lange etiketter på siden Afspilning blev skåret af for tidligt.
- En serie, der har forladt biblioteket, viser stadig sin liste over sæsoner og episoder.

## [0.1.0] - 2026-09-19

Første version

### Added
- Live-indsamling fra Jellyfins sessioner: bruger, titel, klient, enhed, IP-adresse, afspilningsmåde, årsager til omkodning og hardwareacceleration, detaljer om video, lyd og undertekster, og tid set mod tid på pause.
- Login med din Jellyfin-konto. En opsætning i to trin opretter FinStats’ egen API-nøgle, og adgangskoder gemmes aldrig. Brugere, der ikke er administratorer, kan eventuelt logge ind for kun at se deres egen statistik.
- Siderne Oversigt, Aktivitet, Brugere, Biblioteker, Afspilning og Serverlog, en kommandopalet (Ctrl/⌘ K) og en live-statuslinje.
- Import fra Jellystat: store sikkerhedskopier streames til disken og importeres på sekunder, og det er sikkert at importere den samme fil to gange.
- Én lille binærfil med en indbygget webbrugerflade og én SQLite-fil, plus et Docker-image.
