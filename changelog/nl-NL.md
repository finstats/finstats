# Wijzigingslog

Alles wat er in FinStats is veranderd, nieuwste eerst. Dit bestand wordt in het programma
ingebouwd en is wat het tabblad **Wijzigingslog** in de app toont.

Opmaak: `## [version] - date`, of `## [version] - first day to release day` voor een versie die
over meerdere dagen is gemaakt, daarna `### Added`, `### Changed`, `### Security`, `### Performance`, `### Stability`,
`### Fixed` of `### Removed` (Toegevoegd, Gewijzigd, Beveiliging, Prestaties, Stabiliteit, Opgelost, Verwijderd) met
één punt per wijziging. Beveiliging is iets wat iemand had kunnen misbruiken, Prestaties iets wat sneller of lichter is
geworden, Stabiliteit iets wat niet meer kan crashen, vastlopen, lekken of gegevens kwijtraken, en Opgelost iets wat een
verkeerd antwoord gaf. Een `x.y.0`-versie heeft onder de kop een korte titelregel: die titel gebruikt de app als kop van
de hele reeks.

## [2.3.1] - 2026-10-26 to 2026-10-28

### Security
- **De installatie vraagt om een eenmalige code**: zolang FinStats niet is ingesteld, schrijft het bij het starten een code naar zijn log (`docker logs finstats`), en de wizard vraagt erom voordat hij een adres test of iets instelt. Wie de poort als eerste bereikte, kon FinStats anders naar een eigen Jellyfin laten wijzen en er beheerder van worden, of de test gebruiken om in het netwerk rond te kijken.
- **Toegang tot bibliotheken volgt Jellyfin**: iedereen ziet alleen de bibliotheken die Jellyfin voor hem openstelt, en daarin alleen de titels die Jellyfin voor hem toont onder zijn leeftijdsgrens, tags en regels voor titels zonder classificatie, in bibliotheekpagina’s, zoeken, cast, afbeeldingen, Onlangs toegevoegd, de kijklijst, Binnenkort, de agendafeed en meldingen van nieuwe titels. Iedereen die was aangemeld kon elke bibliotheek doorbladeren.
- Een link kon iemand vanuit het welkom direct na het aanmelden naar een andere site sturen. Nu worden alleen de eigen pagina’s van FinStats gevolgd.
- Het serverlog en de mislukte aanmeldingen in de inzichten gaven de aanmeldadressen en activiteit van iedereen aan iedereen met *De server zien*. Regels van een ander vragen *Activiteit van iedereen zien* en een adres *Netwerkdetails zien*, zoals overal elders.
- Een persoonlijke meldingsbestemming kon het lokale netwerk bereiken via een naam die bij de tweede keer opzoeken een ander antwoord gaf. Er wordt verbonden met het adres dat is gecontroleerd, meer bereiken die nooit het internet bereiken tellen als lokaal, het eigen publieke adres van het huishouden wordt ook geweigerd, en een naam die niet kan worden opgezocht wordt in dezelfde woorden geweigerd als een lokale.
- Achter een proxy die een eigen `X-Forwarded-For`-regel toevoegt, of vanaf een IPv6-netwerk, kon elke aanmeldpoging van een nieuw adres komen en gold de limiet nooit. Een IPv6-/48 krijgt nu 50 pogingen in vijf minuten, een volle limiet weigert een nieuw adres in plaats van een oud te vergeten, de hele server stopt bij 60 mislukte pogingen per minuut, en een mislukte aanmelding wordt eenmaal per adres en tijdvenster in het auditlog gezet.
- `FINSTATS_TRUST_PROXY` noemt de proxy’s (adressen of bereiken) waarvan `X-Forwarded-For`, `X-Forwarded-Host` en `X-Forwarded-Proto` worden geloofd, en `1` of `true` gelooft nu alleen een peer op loopback of op een privénetwerk. Ingeschakeld kon elke peer zijn eigen adres kiezen; uitgeschakeld achter een proxy sloten tien verkeerde gokpogingen iedereen buiten.
- Zodra de gebruikers zijn gelezen, wordt een aanmelding voor een naam die onbekend, uitgeschakeld, verwijderd of niet tot FinStats toegelaten is, zonder Jellyfin te vragen beantwoord als een verkeerd wachtwoord, en niet eerder dan Jellyfin zou hebben geantwoord. Zulke namen konden Jellyfin-accounts blokkeren, de regel van Jellyfin voor toegang op afstand omzeilen of zich verschuilen achter het adres van FinStats.
- Wie door Jellyfin tot het thuisnetwerk wordt beperkt (*Externe verbindingen toestaan* uit), meldt zich alleen van daaruit aan bij FinStats, omdat Jellyfin de schakelaar beoordeelde op het eigen adres van FinStats, en een gebruikersuitlezing die niet zegt of iemand is uitgeschakeld, houdt die persoon buiten.
- Een sessiecookie wordt als Secure gemarkeerd zodra de aanmeldpagina op https staat, ook achter een proxy die dat niet meldt.
- Een geslaagde aanmelding vergeeft niet langer de gokpogingen ervoor. Op https is de sessiecookie een `__Host-`-cookie die een site op een naburige naam niet kan vervangen en die onder geen andere naam wordt aangenomen, en afmelden beëindigt elke sessie die de browser meestuurde.
- Alleen een agendasleutel opent de agendafeed vanuit het adres; elke andere sleutel hoort in de header.
- Een API-sleutel kan niet langer rechten toekennen, wijzigen wie erin komt of wat openbaar is, een back-up herstellen, of een dienst of meldingsbestemming toevoegen, wijzigen of testen: elk daarvan overleeft de sleutel, dus het vraagt een aangemelde browser.
- Elk antwoord draagt `Cross-Origin-Opener-Policy: same-origin` en een `Permissions-Policy`, API-antwoorden dragen `Cross-Origin-Resource-Policy: same-origin` en zijn `no-store` tenzij ze anders zeggen, en de CSP staat geen `blob:`-afbeeldingen meer toe. Een pagina die FinStats had geopend, kon haar tabblad later veranderen in een eigen aanmeldpagina.
- Waar FinStats het schema kent waarop het wordt bereikt (de `X-Forwarded-Proto` van een vertrouwde proxy, of het publieke adres), wordt een schrijfactie vanaf een ander schema geweigerd als cross-site.
- `/auth/me`, de pagina van een titel en Bibliotheekgezondheid linken naar Jellyfin op zijn publieke adres; het adres waarop FinStats zelf Jellyfin leest, gaat alleen naar beheerders.
- Een afbeelding wordt alleen bij Jellyfin opgehaald voor iets wat FinStats heeft, de posters en kaarten van een gepubliceerd profiel inbegrepen, zodat de afbeeldingen van een andere bibliotheek niet op id kunnen worden opgevraagd.
- De sleutels van een private tracker worden ook uit de fouten van de downloadwachtrij weggelaten, op hun vorm en op hun naam, en wie alleen *Zien wat er downloadt* heeft, hoort welke dienst faalde en hoe, nooit het interne adres of waarheen die doorverwijst.
- De pagina Server noemt de laatste gebruiker van een apparaat alleen met *Activiteit van iedereen zien*, en Niet-gekoppelde media vraagt dat ook, naast *FinStats beheren*. Zonder dat recht telt het onderdeel Taken alleen de eigen afspelingen en streams, wordt een verwijderde bibliotheek vermeld aan de hand van de eigen afspelingen, en kan een wachtend Tautulli-schakelbord niet worden geannuleerd.
- De instellingen tonen de dienst voor het publieke adres alleen met zijn host en het adres van Jellyfin en de datamap alleen aan beheerders, en een titel die de bibliotheek kwijt is wordt alleen beschreven aan de hand van afspelingen die de lezer mag zien.
- Een Tautulli-back-up waarvan de tabellen geen gewone tabellen waren, kon elke andere schrijfactie laten wachten tot FinStats opnieuw werd gestart. Zo’n bestand wordt geweigerd, en SQLite leest de back-up defensief.
- Een upload van meer dan 4 GB wordt geweigerd, en een gezipte Tautulli-back-up wordt niet verder uitgepakt dan de grootte die hij opgeeft; beide konden de schijf vullen waarop de eigen database van FinStats staat.
- Een antwoord van Jellyfin, Sonarr, Radarr, Seerr, het opzoeken van het publieke adres of de GeoIP-download wordt niet verder gelezen dan het hoogstens mag zijn, geteld na het uitpakken.
- Elke Jellyfin-gebruiker kon FinStats voor iedereen laten stoppen met het vastleggen van afspelingen door duizenden korte afspelingen van één titel te melden. Uitzoeken wie samen keek gebeurt nu in één doorgang en houdt daarbij geen vergrendeling vast.
- Afspelingen die direct na elkaar zijn bewaard, of één titel die beurtelings op apparaat na apparaat wordt afgespeeld, voegen geen vreemden meer samen tot één groep, en wie alleen zichzelf mag zien, hoort alleen van de mensen naast wie de eigen afspeling liep, en voor de tijd dat dat zo was.
- Een link kon iemand na het aanmelden toch nog vanuit het welkom naar een andere site sturen, via een pad als `/.//`.
- Wie *FinStats beheren* heeft, kan niet langer wijzigen hoeveel back-ups worden bewaard of hoe vaak ze worden geschreven, het samenvoegvenster (nu hooguit een uur) of wanneer gebruikers, het activiteitenlog en back-ups worden gelezen, en de back-ups van zo iemand verdringen nooit die van een beheerder; thuisadressen en de reisregels vragen wat de waarschuwingen vragen, en geschiedenis importeren vraagt ook *Activiteit van iedereen zien*.
- Een opgeslagen sleutel, token of wachtwoord gaat alleen naar de server waarvoor het is opgeslagen, en een meldingsbestemming wordt alleen aangemaakt of gewijzigd wanneer je bent aangemeld, nooit met een API-sleutel. Een beheerder mag de persoonlijke bestemming van een ander uitschakelen of verwijderen, maar niet verplaatsen, de certificaatcontrole ervan niet uitzetten en het ntfy-topic of de Pushover-gebruikerssleutel niet lezen.
- Webhooks van Discord en Slack moeten `https` zijn, omdat hun adres hun token is.
- Een stroom mislukte aanmeldingen kan een echte niet langer verbergen of de waarschuwingen ondersneeuwen: namen die geen account zijn geven één melding per kwartier die niemand noemt, waarschuwingen gaan eerst, en Slack en Discord tonen wat een bericht zegt als tekst en pingen niemand.
- De rang in de Terugblik, Capaciteit, een melding en een gepubliceerd profiel vertellen niet meer dan de pagina’s: geen rang of aandeel in ieders tijd aan wie alleen zichzelf mag zien, drukke momenten zonder *Activiteit van iedereen zien* alleen als dagen met drie streams of meer, geen mislukte taak of verbinding aan wie FinStats niet mag beheren, en een openbaar profiel verandert eens per dag.
- Een Tautulli-back-up met een gegenereerde kolom wordt geweigerd, en een afbeelding wordt voor een geïmporteerde afspeling pas opgehaald als de bibliotheek de titel heeft.
- Een herstel zet FinStats nooit verder open dan het is: instellingen komen alleen terug als daarom wordt gevraagd, aanmelden, rechten en openbare profielen worden wat zowel deze installatie als de back-up toestaat, en de publieke adressen blijven die van deze installatie. `finstats restore <file>` brengt alleen de geschiedenis terug, zoals de pagina, en met `--with-settings` ook de instellingen.
- Een herstel van alleen de geschiedenis brengt niet langer thuisadressen terug, gedempte waarschuwingen, auditregels die voor die van deze installatie zelf doorgaan of gekoppelde titels die de bibliotheek niet heeft, en geen herstel brengt afspelingen, logregels of apparaten terug die gedateerd zijn vóór 1990 of na morgen. Een herstelde afspeling telt als geïmporteerd, welke bron ze ook opgeeft, zodat een herstart haar nooit voortzet.
- De datamap is alleen van de eigen gebruiker van FinStats (0700, de bestanden 0600, `umask 077` in de image). FinStats weigert als root te draaien boven een datamap van een andere gebruiker en noemt de `docker exec -u` die je moet gebruiken, en het entrypoint wijzigt de eigenaar alleen van de map en van de eigen items van FinStats.
- De naam van een bestemming mag geen stuurtekens bevatten, en de naam van een apparaat, een persoon of een titel kan geen regel in het log meer vervalsen.
- De routeparameter van een link kan geen ander API-pad meer bereiken dan dat van de eigen pagina, en de elementbouwer van FinUI zet een adres alleen als het http(s) of relatief is, nooit `srcdoc`.
- Zoeken, gepubliceerde profielen en met de hand gestart werk kunnen FinStats niet langer bezighouden: een zoekopdracht stopt bij 200 tekens, een profiel wordt per taal bewaard, kaarten wachten in een korte wachtrij, en een taak, het opzoeken van het adres, de download van de locatiedatabase en een testbericht starten maar zo vaak, hoe er ook om wordt gevraagd, testberichten geteld per afzender. Het ophalen van de locatiedatabase vraagt ook wat het lezen van de pagina Beveiliging vraagt.
- De inhoud van een verzoek moet blijven binnenkomen zodra de eerste 15 seconden voorbij zijn, een antwoord dat niemand leest wordt na 30 seconden opgegeven, en één adres houdt hoogstens 64 verbindingen, een vertrouwde proxy uitgezonderd. Een paar honderd verzoeken die hun inhoud nooit stuurden, konden FinStats ervan weerhouden wie dan ook te antwoorden.
- Een mailserver wordt niet verder gelezen dan een antwoord kan zijn (1 MB voor een hele brief), en een testbericht stopt na zo lang als een bericht in de wachtrij mag duren. Een server die nooit een regel afsloot, kon FinStats steeds meer geheugen laten vasthouden tot het werd gestopt.
- Het lezen van een bibliotheek wordt afgemeten aan de titels die eerdere leesbeurten opsloegen, niet aan die van een import, en een Jellystat-titel in een al gelezen bibliotheek komt binnen als verdwenen. Een import van verzonnen titels kon elke leesbeurt FinStats laten stoppen, bij elke start.
- De grens van 60 mislukte aanmeldingen per minuut voor de hele server telt alleen pogingen die naar Jellyfin gaan, ook die nog op antwoord wachten. Namen die FinStats zelf weigerde, lieten iedereen met een paar adressen alle aanmeldingen tegenhouden, en veel gelijktijdig verstuurde gokken bereikten allemaal Jellyfin.
- Wie Jellyfin tot het thuisnetwerk beperkt, blijft daartoe beperkt bij elk verzoek, sessie en sleutels inbegrepen, niet alleen bij het aanmelden; thuis is een privénetwerk of het opgezochte openbare adres van het huishouden, nooit een adres dat als thuis is ingevuld, en achter een proxy die `FINSTATS_TRUST_PROXY` niet noemt blijft diegene buiten. Een cookie of sleutel die het huis uit ging, opende alles vanaf overal, en een beheerder kon het huis openzetten voor een adres naar keuze.
- Een API-sleutel kan niet langer uitzetten wat misbruik ervan zou opmerken, of de back-ups die zouden terugdraaien wat hij deed: hoeveel back-ups worden bewaard en hoe vaak, het samenvoegvenster, thuisadressen en reisregels, de schema's voor gebruikers, het activiteitenlogboek en back-ups, alle meldingen dempen of afhandelen, een back-up in de prullenbak of een meldingsbestemming verwijderen. Elk daarvan duurde langer dan de sleutel zodra die werd ingetrokken.
- De posters van Sonarr en Radarr en de volgende aflevering op een verlanglijst volgen ook de bibliotheektoegang: een serie die Jellyfin voor iemand verbergt, heeft voor diegene poster noch datum. Door de id's van de diensten af te tellen zag je welke verborgen series eraan kwamen, en een lijst noemde de volgende aflevering van een verborgen serie.
- Het hoofdstuk Samen in het jaaroverzicht noemt alleen wie naast jou keek, zoals de pagina Samen dat doet. Een groep waarin je via iemand anders terechtkwam, noemde mensen met wie je nooit keek.
- Een import uit Jellystat of Streamystats leest het hele bestand voordat hij de schrijfvergrendeling van de database neemt, en schrijft daarna alleen de rijen die hij bewaart. Onder de vergrendeling gelezen hield een bestand met een paar echte rijen en gigabytes aan iets anders elke andere schrijfactie aan het wachten, afspelingen en aanmeldingen inbegrepen.
- Een herstel houdt één recht per persoon vast, voor niet meer mensen dan een huishouden vele malen, en controleert een auditregel op dubbels zonder elke regel te lezen die het net schreef. Een gemaakte back-up kon tijdens het herstel geheugen en de schrijfvergrendeling van de database steeds langer vasthouden.
- Een herstel met de instellingen zet het opzoeken van het openbare adres en het downloaden van de locatiedatabase alleen aan waar deze installatie ze ook aan heeft, en bewaart nooit minder back-ups of schrijft ze nooit minder vaak dan deze installatie. Een back-up die er één bewaarde, kon de volgende run alle andere laten verwijderen.
- Een bestandsgrootte die geen schijf kan bevatten, telt als geen grootte, en groottes worden zo opgeteld dat geen totaal kan overlopen. Eén geïmporteerde grootte liet de bibliotheekpagina's voorgoed met een fout antwoorden.
- Zodra FinStats weet dat het via https bereikt wordt (het openbare adres is https, of een vertrouwde proxy zegt het), wordt een sessie alleen uit de `__Host-`-cookie genomen, en met een openbaar https-adres is elke nieuwe sessie een https-sessie. Een aanmelding zonder `Origin` via een proxy die niets zei, gaf een gewone sessie die een naburige site in iemands browser kon planten.
- De image wordt gebouwd met elke GitHub-action vastgezet op een commit en beide basisimages op een digest, alleen verschoven door Dependabot, en een release bouwt zonder de gedeelde buildcache. Een verschoven tag van andermans action had elke image als FinStats kunnen publiceren.
- Het stijlblad van FinUI, getekend in de eigen weergave van de bezoeker, mag alleen door FinStats' eigen pagina's worden geladen. Een pagina op een andere poort van dezelfde host kon eruit lezen of de bezoeker was aangemeld en welke voorinstelling die koos.
- Elke geweigerde aanmelding wordt beantwoord op één moment, gerekend vanaf het versturen, of FinStats of Jellyfin hem nu weigerde, en een naam wordt herkend ongeacht hoofd- of kleine letters, ook buiten A tot Z. Hoe snel een weigering kwam, verried welke namen iemand waren, en ‘Åse’ die `åse` typte, werd geweigerd zonder dat Jellyfin werd gevraagd.
- Downloadfouten verbergen clientadressen voor niet-beheerders. Wie de wachtrij kon bekijken, kon eerder interne hosts, poorten en paden zien.

### Performance
- Een gepubliceerd profiel wordt hooguit eens per minuut opgebouwd, niet opnieuw voor elke afbeelding en kaart erop.
- De geschiedenis van een persoon wordt één keer tegelijk op beveiligingswaarschuwingen doorzocht, hoeveel afspelingen er ook beginnen.
- Aan- of afmelden, een sleutel intrekken of de rechten van één persoon wijzigen laat alleen de liveverbindingen die het betreft opnieuw kijken.
- Het Tautulli-schakelbord leest de geschiedenis één keer, niet één keer per Plex-gebruiker.
- Een afbeelding die in de cache ontbreekt, wordt met één verzoek opgehaald terwijl de andere wachten, en een die de bron niet kon geven, wordt 30 seconden niet opnieuw gevraagd.
- De bestanden van de webinterface worden één keer met gzip ingepakt en bewaard, en lettertypen worden niet nog eens gecomprimeerd.

### Stability
- Een back-up in het oudere formaat van Jellystat wordt rij voor rij gelezen in plaats van in zijn geheel, en een regel die langer is dan die van welke back-up ook wordt geweigerd.
- Twee Tautulli-uploads tegelijk schrijven niet langer in één bestand.
- Een paginanummer ver voorbij het einde van de titels van een bibliotheek of van Bibliotheekgezondheid geeft een lege pagina, geen fout.
- Een import of herstel waarvan de upload werd afgebroken, weigert niet langer elke andere tot een herstart, en noch die, noch een import die werd opgegeven voordat hij begon, laat een bestand achter.
- Eén rij van een back-up wordt niet verder gelezen dan 8 MB, en er komt één Tautulli-back-up tegelijk binnen.
- Een afspeling houdt hooguit 2.000 gebeurtenissen, en een import slaat een afspeling over die niet tussen 1990 en morgen begint en eindigt.
- Eén afleveringsnummer ver onder nul laat niet langer elke bibliotheekuitlezing hangen: Bibliotheekgezondheid (ook de samenvatting), Capaciteit, opnieuw koppelen en lokaliseren tellen op en vergelijken getallen zonder overloop, en een release-build controleert op overloop, zodat iets wat gemist is één taak stopt in plaats van in een lus te blijven.
- Een seizoens-, afleverings- of eindnummer wordt alleen bewaard waar Jellyfin het zou kunnen bevatten (0 tot 2.147.483.647), uit Jellyfin, Jellystat en Tautulli gelijk.
- Een Tautulli-back-up waarvan de id’s zich herhalen, wordt geweigerd, omdat een paar MB aan rijen onder één id het proces zonder geheugen liet, en een back-up wordt onveranderlijk geopend en laat geen `-wal` of `-shm` achter.
- Een verbinding moet haar verzoek binnen 15 seconden sturen en er worden er hooguit 512 tegelijk bediend: een paar honderd half-open sockets haalden FinStats offline. Een frame op het livekanaal mag hooguit 4 KB zijn.
- Een paar kilobyte van Sonarr, Radarr of Seerr kunnen niet langer gigabytes aan geheugen worden: een lijst houdt niet meer dan wat FinStats ervan leest.
- Een meldingsbestemming die nooit antwoordt, houdt alleen zichzelf op: naar bestemmingen wordt naast elkaar gestuurd, en een die steeds een time-out geeft, wordt tien minuten met rust gelaten.
- De resterende tijd en totale snelheid van een wachtrij, de getallen van een herstelde afspeling en de dag waarop een afspeling de prullenbak verlaat, kunnen niet langer overlopen.
- Een herstel leest het hele bestand voordat het de schrijfvergrendeling neemt, weigert een bestand dat uitpakt tot veel meer dan zijn grootte, en schrijft in batches, zodat de collector intussen blijft vastleggen.
- Back-ups worden in UTC benoemd en nooit over een bestaand bestand geschreven, en alleen de eigen kopieën van FinStats van vóór een update worden opgeruimd.
- Een regel in het serverlog die in de toekomst is gedateerd, houdt het lezen van het activiteitenlog niet langer tegen: het wordt gelezen vanaf de nieuwste regel die niet in de toekomst ligt.
- Verzoeken die tegelijk op een gepubliceerde link binnenkomen, bouwen het antwoord één keer op, en de kaarten ervan worden in een eigen baan getekend, zodat iemand met de link de Terugblik-kaarten van de eigenaar niet langer met 503 kan laten antwoorden.
- De afspelingen van een bibliotheek die in Jellyfins lijst van bibliotheken ontbreekt, wachten twee dagen voordat ze naar de kopie van dezelfde titel in een andere bibliotheek gaan. Eén lijst die een bibliotheek wegliet, verplaatste ze meteen, en ze bleven verplaatst toen de bibliotheek terugkwam.
- Een schijf die Jellyfin groter opgeeft dan een getal kan bevatten, laat niet langer elke lezing van de servergegevens mislukken.
- Het uitlezen van Jellyfin stopt als een pagina zich herhaalt zonder nieuwe ID’s.

### Fixed
- Mislukte aanmeldingen op een account geven één waarschuwing per tijdvenster die bij twintig uitgroeit tot een alarm, en een naam telt alleen tegen een account als die volledig is getypt ("x alice" telt niet langer tegen alice).
- Berichten aan Discord en Slack worden eerst ge-escaped en daarna ingekort: een apparaatnaam vol Markdown liet Discord elke waarschuwing weigeren die hem noemde.
- Een publiek IPv6-adres wordt geaccepteerd voor een persoonlijke bestemming.
- Achter een proxy die zelf aanmeldt (nginx `auth_basic`), wordt de cookie van FinStats niet langer geweigerd.
- Een weergavenaam met `{{url}}` wordt in de voorvertoning van een gepubliceerde pagina getoond zoals hij is geschreven.
- Een terugblik voor het jaar 9999 toont het standaardjaar in plaats van een fout.
- Een live pagina waarvan het recht werd ingetrokken en teruggegeven terwijl ze open stond, blijft weer bij zonder herladen.

### Removed
- **Synnmørsk** is geen taal van FinStats meer: een browser die het had gekozen, leest Nynorsk, de schrijftaal waaruit het is gemaakt.

## [2.3.0] - 2026-10-07 to 2026-10-26

Capaciteit, een livekanaal, eenvoudige modus en talen

### Added
- **Talen**: FinStats spreekt 23 talen. Brits en Amerikaans Engels en Noors Bokmål en Noors Nynorsk worden door de beheerder geschreven en nagekeken; Deens, Duits, Spaans, Fins, Frans, Hindi, IJslands, Italiaans, Japans, Koreaans, Nederlands, Pools, Portugees (Brazilië), Russisch, Zweeds, Turks, Oekraïens en Chinees (vereenvoudigd en traditioneel) zijn vertalingen die niemand hier heeft nagekeken, en zeggen in het Engels wat ze nog niet hebben. Elke pagina van beide indelingen, het openbare profiel, het Wijzigingslog en wat de server in woorden antwoordt (een foutmelding, de kaarten van de Terugblik, wat een taak van Jellyfin doet) verschijnen in de taal die onder Instellingen → Uiterlijk → Taal is gekozen, per browser; tot er een is gekozen, beslist de eigen taal van de browser. Getallen, datums, eenheden en opsommingen worden geschreven zoals die taal ze schrijft. Een script kan om een taal vragen met `Accept-Language`.
- **Eenvoudige modus**: een tweede indeling voor wie van Plex komt, opgebouwd zoals een klassieke statistiekserver voor Plex. Een balk bovenaan in plaats van het menu, en alleen de statistiekpagina’s: Start, Bibliotheken, Gebruikers, Geschiedenis, Grafieken en Instellingen, met een tandwiel voor je profiel, Instellingen, het Wijzigingslog, het thema en afmelden. Te kiezen per browser onder Instellingen → Uiterlijk → Indeling. Hij laadt alleen zijn eigen helft van FinStats en start daardoor sneller; Terugblik, Samen, Onderweg, Beveiliging en de serverpagina’s blijven in de FinStats-indeling. Hij draagt de eigen kleuren van FinStats, licht en donker, en niets erin beweegt.
- **Start in de eenvoudige modus**: wat er speelt, met een kaart per stream (hoe hij afspeelt, thuis of van buitenaf, wanneer hij afloopt), de kijkstatistieken als gerangschikte kaarten naar aantal weergaven of duur over de afgelopen N dagen, wat elke bibliotheek bevat en wat er onlangs is binnengekomen.
- **Grafieken in de eenvoudige modus**: weergaven per mediatype (per dag, per weekdag en uur, de tien belangrijkste apps en personen), naar hoe ze afspeelden (per dag, de meeste streams tegelijk op elke dag, de resolutie van het bestand en de verzonden resolutie, apps en personen) en per maand, naar aantal weergaven of duur, voor iedereen of voor de gekozen personen.
- **Bibliotheken, Gebruikers en titels in de eenvoudige modus**: tabellen van elke bibliotheek en persoon met wat er het laatst is afgespeeld, waarop en vanwaar (het adres alleen met *Netwerkdetails zien*), en een pagina voor elke bibliotheek, persoon en titel met de cijfers en een eigen geschiedenis.
- Voor scripts: `/api/stats/split` (weergaven per app, persoon, resolutie, dag of maand, uitgesplitst naar mediatype of naar hoe ze afspeelden); toplijsten naar aantal verschillende kijkers (`sort=users`); activiteit beperkt tot gekeken weergaven (`watched=1`); inzichten voor de gevraagde onderdelen (`parts=`); de piek uitgesplitst in transcoderingen, direct streamen en direct afspelen, ook per dag; wat elke bibliotheek bevat en het laatst afspeelde; onlangs toegevoegd per soort (`kind=`); de heatmap per mediatype; de laatste weergave van een persoon; of een stream van thuis afspeelt.
- **Een livekanaal**: het dashboard, de downloads van Onderweg en de statusbalk houden zich actueel over één WebSocket per tabblad (`/api/live`), zodat wat er speelt en wat er downloadt op het scherm verandert op het moment dat het verandert, en niet pas bij de volgende keer kijken. Achter een reverse proxy die WebSocket-upgrades niet doorgeeft, of op een verbinding die zonder melding is weggevallen, vragen de pagina’s zoals voorheen om de paar seconden. `/api/now-playing`, `/api/downloads` en `/api/summary` blijven zoals ze zijn, voor scripts.
- **Server → Capaciteit**: kan deze server het aan, op basis van wat FinStats heeft gezien. De drukste momenten ooit en van de afgelopen 90 dagen: streams tegelijk, en hoeveel daarvan hun beeld met hardware lieten transcoderen, zonder hardware, of zonder dat Jellyfin het zei (een geschiedenis die uit Tautulli is geïmporteerd, is die van Plex en telt niet mee). Elke schijf waar de bibliotheken op staan en hoe snel wat Jellyfin in zes maanden heeft toegevoegd hem vult (een schijf die pas onlangs is gevuld, wordt gemeten vanaf een maand na zijn eerste bestand, want die eerste maand is het vullen zelf), en, waar Jellyfin 10.11 of nieuwer de vrije ruimte meldt, hoe lang die in dat tempo meegaat. FinStats zegt nooit hoeveel streams de server aan zou kunnen.
- **.ics exporteren** onder Onderweg → Binnenkort: de lijst zoals je hem hebt gefilterd, gedownload als agendabestand om één keer te importeren. Het werkt zichzelf niet bij; een agendasleutel uit Instellingen → API-sleutels geeft nog steeds een agenda die dat wel doet.
- **Het bekijken waard**: Capaciteit wijst erop wanneer elke transcodering waarover Jellyfin in 90 dagen rapporteerde zonder hardwareversnelling liep terwijl transcoderen vaak voorkomt, wanneer de bibliotheekscan van Jellyfin een kwart van de tijd of meer loopt, en wanneer een schijf nog minder dan zes maanden ruimte heeft. Elk punt komt met zijn cijfers en kan opzij worden gelegd, net als de bevindingen van Bibliotheekgezondheid.
- **Waar je bibliotheek uit bestaat** is te beperken tot één bibliotheek of meer, onder Bibliotheken en op de pagina van elke bibliotheek (`library_id=a,b`).
- **Datum- en tijdnotaties** in Instellingen → Uiterlijk: een datum als 31.01.2026, 31/01/2026, 31-01-2026, 01/31/2026, 2026-01-31, 2026.01.31 of 2026/01/31, en een tijd in 24 of 12 uur, met of zonder voorloopnul, of elk zoals de taal het schrijft. Bewaard door deze browser.
- **Een welkom**: de eerste keer dat iemand zich aanmeldt, in welke browser ook, en aan het eind van de installatie, vraagt FinStats naar de taal, de datum- en tijdnotaties en de indeling, met van elk een afbeelding. Overslaan houdt Brits Engels, 31/01/2026, 13:05 en FinStats.

### Changed
- Het logo in een pagina is het kale beeldmerk naast de naam, gezet in Inter welk lettertype een preset ook kiest; waar het alleen staat (het favicon) houdt het zijn afgeronde tegel.
- Een weergave die in software is getranscodeerd, wordt als zodanig vastgelegd waar Jellyfin dat zegt; voorheen zag die er hetzelfde uit als een van een server die het niet zegt. Pagina’s noemen beide nog steeds Software.
- Een update voert nu, één keer en de oudste versie eerst, uit wat een versie moet doen aan een database die een oudere FinStats heeft geschreven, naast de migraties. Die van 2.3.0 leest bij de eerste start de gegevens van de server, zodat Capaciteit er meteen is.

### Performance
- **Een pagina die openstaat, vraagt niets**: het dashboard deed 30 verzoeken per minuut om actueel te blijven en de downloads van Onderweg 18; met het livekanaal doen ze er allebei geen. Hoeveel tabbladen er ook openstaan, FinStats vraagt Jellyfin, Sonarr en Radarr niet meer dan voorheen, en een verborgen tabblad laat de downloadwachtrij nog steeds vertragen.
- De pagina van een persoon leest alleen de weergaven van die persoon: totalen en laatste weergave werden over elke weergave op de server uitgerekend en daarna tot die persoon beperkt, ongeveer vier seconden bij een miljoen weergaven.
- De pagina van een bibliotheek leest alleen de titels en weergaven van die bibliotheek, waar hij die van elke bibliotheek telde en er één hield: een derde van de tijd bij een miljoen weergaven.
- Liep de klok van Jellyfin voor op die van FinStats, dan las *Na de bibliotheekscan van Jellyfin* elke minuut de hele bibliotheek opnieuw tot het verschil voorbij was; elke scan wordt nu één keer gevolgd.
- Een GeoIP-bestand dat niet kon worden geopend, werd elk kwartier opnieuw gekopieerd, honderd megabyte en meer, en elk adres werd opnieuw geplaatst; het wordt pas opnieuw geprobeerd als het verandert.
- Bibliotheekgezondheid opent meteen nadat de aanwijzer op de link heeft gerust: wat vooraf was opgehaald, werd bewaard waar de pagina nooit keek.

### Stability
- Een update die mislukte nadat een migratie was uitgevoerd en die Docker meteen herstartte, maakte bij elke start een nieuwe kopie van de half bijgewerkte database, en na drie starts was de enige kopie die de vorige versie kon openen weggesnoeid. De kopie van vóór de update blijft nu bewaard.
- Een titel die in Jellyfin werd verwijderd terwijl de bibliotheek werd gelezen (een bestand dat Radarr verving), kon ervoor zorgen dat het lezen een andere titel miste die er nog wel was en die als verwijderd markeerde, waarna de weergaven ervan op een andere kopie konden worden gezet. Wat het lezen niet zag, wordt nu eerst op id nagevraagd.
- Een back-up terugzetten die was gemaakt voordat een bestand werd hernoemd of geüpgraded, zette elke weergave die sindsdien naar het nieuwe bestand was verplaatst een tweede keer terug, voorgoed verdubbeld.
- Een aanvraag in het antwoord van Seerr die FinStats niet kon lezen (een veld dat in een update van Seerr was hernoemd), werd door de dagelijkse volledige leesronde als verwijderd gemarkeerd.
- Een taak die tegen een bug aanliep, bleef voorgoed op ‘bezig’ staan, en geen trigger startte hem opnieuw tot FinStats werd herstart; hij eindigt nu als mislukt.
- Bij een beschadigde zip van Tautulli stopte het uploaden voordat het bestand, dat de tokens van elke Plex-gebruiker bevat, werd verwijderd; het bleef liggen tot de opruimronde na zes uur.
- Een import of terugzetting die werd afgebroken terwijl het bestand binnenkwam, liet dat bestand voorgoed in de gegevensmap staan, met adressen en al; het wordt bij de volgende start verwijderd.
- Een afbeelding kon half geschreven worden geserveerd terwijl ze in de cache werd gezet, en de browser hield haar dan zo.
- Ingetrokken API-sleutels werden voorgoed bewaard, hoeveel er ook werden gemaakt en ingetrokken.
- Een lezing van iemands bekeken- en favorietmarkeringen die leeg of onleesbaar terugkwam, wiste ze allemaal tot de volgende lezing; die wordt nu geweigerd, zoals een lege lezing van een bibliotheek.
- Een aflevering met een eindnummer dat geen aflevering kan hebben (een datum die daar staat) kon Bibliotheekgezondheid na een lezing van de bibliotheek eindeloos bezighouden.

### Fixed
- Een notitie in het Wijzigingslog met code binnen vette tekst (die van 2.0.2 en 2.0.3) toonde de backticks in plaats van de code.
- Een grafiek van aantallen kon zijn as nummeren als 0, 3, 5, 8, 10, of zelfs 0, 0, 1, 1, 1: stappen van 2,5 afgerond geschreven. Aantallen stijgen nu in hele stappen.
- Een agendasleutel die was gemaakt door iemand die geen beheerder is, kwam met een link op het eigen adres van de browser in plaats van het openbare adres van FinStats, zodat hij alleen van binnenuit werkte; de pagina vroeg ook om instellingen die werden geweigerd.
- De laatste weergave van een persoon kon een paar seconden te kort zijn om ergens anders als weergave te tellen, en noemde dan een titel die niet in de Geschiedenis van die persoon staat.
- Als terugzetten uit Onlangs verwijderd mislukte, zei de pagina niets en bleef de knop draaien; hetzelfde gold voor een mislukt *Terughalen* in Bibliotheekgezondheid en een mislukt *Heropenen* of *Alles afhandelen* in Beveiliging.
- De agendafeed heet FinStats in een agenda-app, niet finstats.
- De downloadwachtrij noemde aan iedereen die hem mocht zien wie om elke download had gevraagd; voor de aanvraag van een ander is nu *Activiteit van iedereen zien* nodig, net als in Onderweg.
- Als *Meldingen ontvangen* werd ingetrokken, bleven de bestemmingen van die persoon berichten krijgen, en zelf kon diegene ze niet meer uitschakelen.
- Een weergave die binnen het samenvoegvenster werd gepauzeerd en hervat, werd één keer als gestopt gemeld, en het hervatten en het echte einde nooit.
- De weergaven van een nummer konden naar een nummer met dezelfde naam (‘Intro’) van een andere artiest worden verplaatst als het album de bibliotheek verliet.
- Een meldingsbestemming die als IPv4-adres binnen IPv6 was geschreven (NAT64, 6to4), ging door als openbaar terwijl ze het lokale netwerk bereikte.
- Een koppeling met Sonarr of Radarr die naar een andere server werd gericht, bleef een week lang de posters van de oude server tonen.
- De waarschuwingen van Beveiliging zeiden dat er niets te bekijken was als er twee personen of meer waren gekozen, en een pagina die leeg was geraakt doordat de laatste waarschuwing was afgehandeld, had geen weg terug.
- Elke API-sleutel die nog niet was verlopen, zei dat hij ‘zojuist’ was verlopen.
- Een dialoogvenster bleef na Terug boven de pagina staan, en de pagina scrolde niet tot het was gesloten.
- Het auditlogboek toonde acht soorten vermeldingen (een teruggehaalde weergave of back-up, een geleegde prullenbak, openbare profielen, een gewijzigd uiterlijk) met hun id’s, en niets onder Details.
- Een titel via het contextmenu op de kijklijst zetten mislukte zonder melding als de server weigerde.
- *Samen gekeken* op het dashboard behandelde meerdere tegelijk gekozen personen als één persoon.
- De Terugblik schreef uren en dagen onder de tien met de decimale komma van de browser, naast de eigen komma’s tussen duizendtallen.
- Capaciteit zei dat een schijf vol zou raken "in het tempo van de afgelopen 0 maanden".
- Op de pagina Beveiliging telden de adressen zonder plaats ieders afspelingen van altijd, welke persoon en periode de pagina ook toonde.
- Iemand met *FinStats beheren* maar zonder *Activiteit van iedereen zien* kon ieders afspeling naar de prullenbak verplaatsen.
- Een kaart uit de Terugblik of een openbaar profiel kon uit de cache van de browser komen in de taal die eerder was gekozen.
- De downloads die het livekanaal stuurde, waren in Brits Engels, in welke taal de pagina ook was.
- Een aanmelding die veranderde wat iemand mag zien, bereikte diens andere open tabbladen pas bij de volgende controle na een halve minuut.
- Een tabblad dat zijn liveverbinding aan een nieuwer tabblad afstond, nam er bij de volgende pagina een terug, zodat de tabbladen van één persoon elkaars verbinding bleven sluiten.
- Tabellen sorteerden groottes van 1.000 tot 1.023 eenheden als tekst in talen die cijfers met een spatie groeperen, en Turkse percentages evenzo.
- Een titel met een plaatshouder in de naam, zoals "Big {device} Bunny", werd ingevuld door de meldingen van de server.
- Tabellen sorteerden duur en aantallen van een totaal (5 van 26) als tekst in elke taal behalve Engels en de Noordse talen.
- Een tabblad dat nog in de ene taal openstond, hoorde de server in een andere zodra een ander tabblad die had gekozen.
- Een woord naast een getal kreeg de verkeerde vorm in het Pools, Russisch en Oekraïens ("5 wieczory", "2 потоков").
- Capaciteit telde de eerste vulling van een bibliotheek als groei en deelde wat elke bibliotheek toevoegde door zes maanden, over welke periode er ook gemeten was; een bestand dat de plaats van een ander innam (een upgrade uit Radarr of Sonarr, of een bestand dat Jellyfin een nieuwe id gaf) telde als helemaal nieuw, terwijl alleen het verschil nieuw was.
- Wie samen keek met SyncPlay was geen groep als de een minuten na de ander aanschoof; afspeelbeurten die tegelijk op dezelfde plek eindigen horen nu bij elkaar, en de geschiedenis wordt één keer opnieuw gegroepeerd.
- Iemand zonder *Activiteit van iedereen zien* kon aan *Nooit gekeken* van een bibliotheek zien dat een ander een titel had afgespeeld; nu telt alleen het eigen kijken.

## [2.2.0] - 2026-10-01 to 2026-10-07

Bibliotheekgezondheid en FinUI

### Added
- **Een eigen menu op een computer**: Instellingen → Uiterlijk biedt er acht: de zijbalk zoals hij was, gegroepeerd naar waar de pagina’s voor zijn, een rail van pictogrammen die onder de aanwijzer openklapt tot de zijbalk, een zijbalk die met zoeken begint, een dock, een opdrachtbalk, pagina’s die je bovenaan vastzet, en gekleurde tegels. De zijbalken kunnen rechts staan. Het dock schuift weg terwijl je omlaag scrolt en komt terug als je omhoog scrolt of naar de onderrand van het venster wijst; daarbij verhuist de statusbalk naar boven.
- **Een eigen menu op een telefoon**: een tabbalk met Meer (de nieuwe standaard), een paneel dat half opengaat, het hele scherm, de pagina’s in een waaier rond de duim, of een adresbalk die uitgroeit tot het menu. Elk menu doorzoekt zijn pagina’s en geeft andere woorden door aan de zoekfunctie.
- Beide worden bewaard door de browser waarin ze zijn gekozen, zodat een telefoon en een computer elk hun eigen keuze houden; beide zijn FinUI-componenten, `desktop-nav` en `mobile-nav`.
- **Een contextmenu**: klik met rechts op een titel, een persoon of een bibliotheek waar dan ook in FinStats (een poster op het dashboard, een rij in een lijst, een naam in een tabel, een film onder Binnenkort die nog niet in de bibliotheek staat), of houd er een vinger op, of druk op de menutoets, en er groeit een menu uit het punt waar je drukte: openen, openen in een nieuw tabblad, een film of serie op je kijklijst zetten, openen in Jellyfin of op TMDB en TVDB, de link kopiëren. Shift met een rechterklik blijft van de browser zelf. FinUI’s `context-menu`.
- **Zoeken zonder dialoogvenster**: zoeken groeit uit je menu (een zijbalk wordt breder, het dock rijst op, de naam van de opdrachtbalk verandert erin, het menu van een telefoon houdt het vast) of neemt, als je dat kiest in Instellingen → Uiterlijk, de plaats van de pagina in terwijl het menu blijft staan. Ctrl+Space opent het zoals voorheen; het venster boven alles is verdwenen. FinUI’s `search`.
- **Alles in een bibliotheek**: de kaart Onlangs toegevoegd van een bibliotheek biedt *Alles erin*, elke film, serie of elk album dat ze bevat als posters, op naam, jaar, datum van toevoegen of grootte, en verfijnd terwijl je typt. Er komen er meer bij terwijl je scrolt.
- **Server → Bibliotheekgezondheid**: FinStats vergelijkt elk bestand met zijn buren en somt op wat eruit springt, elk met de cijfers erachter. Het meldt alleen; er wordt in Jellyfin niets gewijzigd, verwijderd of opnieuw gescand.
- **Gaten**: afleveringen die ontbreken tussen twee bestanden van een seizoen (‘Seizoen 2: afleveringen 4 en 5 ontbreken tussen 3 en 6’). Een bestand met twee afleveringen telt als beide; specials en het einde van een seizoen blijven buiten beschouwing.
- **Seizoenen die afwijken** in resolutie, HDR of codec van de rest van hun serie, en **afwijkende afleveringen**: één bestand dat niet bij zijn seizoen past, zoals een SD-aflevering in een seizoen in 1080p. De resolutie gaat op breedte of hoogte, zodat een bijgesneden film van 1920×800 1080p is.
- **Kopieën en versies**: dezelfde film of aflevering twee keer, volgens de provider-id’s waar de kijklijst en Onderweg al op afgaan. Twee kopieën in één resolutie tonen de ruimte die dubbel wordt bezet; een 4K-kopie naast een 1080p-kopie staat vermeld als twee versies en telt nooit als verspilling.
- **Dunne bestanden**: een bitrate ver onder wat de resolutie en codec gewoonlijk nodig hebben (‘4K HEVC met 1,1 Mbps’); de dunne afleveringen van een seizoen zijn één bevinding.
- **Nasynchronisaties die stoppen**: een audiotaal die sommige seizoenen in elke aflevering hebben en andere in geen enkele.
- **Nooit geïdentificeerd**: films en series die Jellyfin heeft gekoppeld zonder TMDB-, TVDB- of IMDb-id.
- **Negeer** een bevinding die bewust zo is, met een notitie. Ze komt vanzelf terug als haar bestanden veranderen. Wie FinStats mag beheren en de server mag zien, kan negeren; genegeerde bevindingen staan in het auditlogboek en horen bij een back-up.
- De pagina van een bibliotheek zegt hoeveel er daar te bekijken is, en leidt ernaartoe.
- **FinUI**, de componenten waaruit FinStats is opgebouwd, is een eigen project (`github.com/finstats/finui`): elke knop, kaart en elk bedieningselement in elke toestand en beide thema’s op `finui.finstats.no`, en **FinUI create** om de kleuren, afronding, dichtheid en meer aan te passen terwijl je kijkt en het met één opdracht mee naar huis te nemen. **Instellingen → Systeem** leidt erheen.
- **Instellingen → Uiterlijk**: iedereen kiest zelf hoe FinStats eruitziet, met de code van een FinUI-preset: basiskleur, accent, grafiekkleuren, contrast, afronding, dichtheid, randen, kaarten, knoppen, velden, tabellen, markering, beweging, pictogrammen, het menu, de pagina achter alles, de focusring, hoe koppen worden gezet, en de lettertypen voor tekst, koppen en cijfers. Je keuze is alleen van jou; de pagina laat zien wat de code betekent en leidt naar FinUI create om haar te wijzigen.
- **Stijlen**: elf complete looks om mee te beginnen, in één keuze in FinUI create, waarvan Instellingen → Uiterlijk de code overneemt. Het zijn Washi (FinStats zoals het wordt geleverd), Studio, Noir, Matinee, Console, Pastille, Ledger, Gazette, Arcade, Canopy en Harbour. FinStats zelf levert één look en biedt geen lijst met stijlen.
- **Pictogrammen die bewegen**: elk pictogram in een link, een knop of een tabblad maakt één keer zijn eigen beweging als je ernaar wijst (vernieuwen draait, de pijl van downloaden valt in zijn bakje, de prullenbak tilt zijn deksel op), en een vernieuwknop die is ingedrukt blijft draaien tot het antwoord binnen is. Ze staan stil bij beperkte beweging en bij Beweging: Uit.
- **Beweging, met FinMotion**: de componenten van FinUI bewegen nu: de knop van een schakelaar wordt naar de overkant gegooid, de onderstreping van een tabblad kruipt naar het volgende tabblad, een geweigerd veld schudt van nee, een kopie wordt in één haal afgevinkt, meldingen liggen op elkaar als een hand kaarten en waaieren open als je ernaar wijst, het paneel van een telefoon geeft mee als een elastiek als je eraan trekt, de rijen van een gesorteerde tabel schuiven naar hun plaats, een getal dat verandert rolt als een kilometerteller, en grafieken tekenen zichzelf de eerste keer dat ze in beeld komen. Alles volgt de keuze voor Beweging in Instellingen → Uiterlijk en staat stil bij beperkte beweging. FinMotion is een eigen project (`github.com/finstats/finmotion`), dat bovenop FinUI ligt.
- **Meer pictogrammen in FinUI**: nu 239, eerst 71, elk met een eigen beweging (volume omhoog en omlaag, een headset en een gedempte, een potlood dat schrijft, een bel die rinkelt, een telefoon die zoemt), en elk ervan kan uit de galerie van FinUI worden gedownload als SVG, stilstaand of geanimeerd.
- **Lettertypen**: vierendertig families met een open licentie om uit te kiezen (Geist, Roboto, Manrope, IBM Plex Sans, Montserrat, Lora, Playfair Display, Fira Code en meer), meegeleverd zoals Inter, nooit ergens anders vandaan gehaald, en alleen geladen als ze zijn gekozen.
- **Meer basiskleuren en accenten**: Nord, Solarized, Gruvbox, Catppuccin, Rosé Pine, Tokyo Night, Dracula, Everforest en Ocean; zestien accenten meer, waarvan tien overdag de ene kleur hebben en ’s nachts een andere. Elke basiskleur heeft overdag nu een eigen papier: de grijzen leken in het lichte thema op elkaar.
- **Verwijderen kan 30 dagen ongedaan worden gemaakt**: een verwijderde weergave of back-up wacht in de prullenbak, en **Ongedaan maken** in de melding die volgt haalt hem terug. Activiteit en Instellingen → Back-ups tonen allebei wat onlangs is verwijderd en wanneer het definitief verdwijnt, als er iets is. Een weergave in de prullenbak telt niet mee in totalen, grafieken, terugblikken en profielen, en een import of terugzetten brengt hem niet als nieuw terug; wie hem samen met iemand keek, telt als alleen tot hij terugkomt.

### Changed
- Dark Reader en vergelijkbare extensies laten de pagina’s van FinStats met rust (`darkreader-lock`): FinStats tekent beide thema’s zelf, en een overgeschilderde pagina vocht daartegen.
- Op een telefoon schuift het menu niet meer van links in beeld: de tabbalk staat onderaan, tenzij er een ander is gekozen.
- FinStats leest na deze update je bibliotheek één keer, om te weten waar bestanden met meerdere afleveringen eindigen.
- Op de pagina van een titel is **Aan kijklijst toevoegen** even hoog als **Openen in Jellyfin** ernaast; de knop was een maat kleiner.
- De filters voor periode en personen blijven bovenaan het scherm terwijl een pagina scrolt, zodat 7d onderaan een lange pagina Alles wordt; net als de filters voor aanvragen en Binnenkort in Onderweg.
- De knop naar boven blijft uit de buurt van het menu dat de onderkant van het venster inneemt: laag in de hoek naast het dock, net boven de tabbalk, duimknop of adresbalk van een telefoon, en anders boven de statusbalk. Het is FinUI’s `to-top`.
- Een weergave of back-up verwijderen verplaatst hem naar de prullenbak in plaats van hem meteen te verwijderen (`PUT` met `{"deleted": true}`; `DELETE` is uit de API verdwenen). Back-ups boven het aantal dat wordt bewaard, worden nog steeds definitief verwijderd.
- Het **Wijzigingslog** is één blad in plaats van een stapel kaarten: elke notitie is een eigen regel met haar soort (Toegevoegd, Gewijzigd, Opgelost en de rest) en het pictogram van die soort in een kolom ervoor, onder een kop per reeks. De lijst met versies blijft rechts staan.

### Performance
- FinStats openen downloadt alleen wat is veranderd: een stylesheet, script of lettertype dat de browser al heeft, wordt in een paar bytes bevestigd in plaats van opnieuw verstuurd. Elk bezoek haalde alle stylesheets en modules van FinStats volledig op, hoewel elk een tag had om ze uit elkaar te houden.

### Stability
- Een back-up terugzetten leest de instellingen alleen uit de instellingenrij: van een bestand van een nieuwere FinStats die daar meer bewaart, vielen de instellingen zonder melding weg.
- Een open tabblad groeit niet meer met elke bezochte pagina: elke pagina die was verlaten, bleef in het geheugen achter de pagina op het scherm (ongeveer 2.600 elementen per bezoek), vastgehouden door de lijst van FinMotion met wat het beweegt en door de scroll-listener van de filterbalk, tot het tabblad werd gesloten.
- Een verkeerd getypte opdracht (`finstats bakcup`), of `restore` zonder bestand, maakt of upgradet niet meer eerst een database voordat hij zegt dat hij niet weet wat hij moet doen.

### Fixed
- Het kleine grafiekje van een tegel loopt niet meer buiten de tegel als het getal ernaast breed is (‘56d 23u’), of als vier tegels een smallere rij delen: de grafiek wijkt en het getal houdt zijn breedte.
- In het lichte thema werd een primaire knop violet onder de aanwijzer; hij houdt nu de kleur van het zegel.
- In Instellingen staat een schakelaar op gelijke hoogte met de naam en de uitleg van zijn instelling; hij hing bovenaan de rij.
- In een gerangschikte lijst zonder iets onder een naam, zoals Topgebruikers op het dashboard, stond de naam bovenaan zijn rij naast een gecentreerde afbeelding; hij is nu gecentreerd.
- Een rij statistiektegels op een breedte tussen telefoon en desktop (vier tegels, drie naast elkaar) liet de laatste alleen op een eigen rij staan; vier tegels staan nu met vier, twee of één naast elkaar, en drie met drie of één.
- De leesronde van de server telde een apparaat waarop niemand was aangemeld, terwijl zo’n apparaat niet wordt bewaard: de taak zei drie apparaten waar de pagina Server er twee toonde.
- Eén seconde, één minuut, één uur: waarschuwingen zeiden ‘1 seconden uit elkaar’ en ‘over 1 minuten’, en een taak van Jellyfin met een interval van één minuut liep ‘elke 1 minuten’.
- `finstats --help` noemt elke variabele die FinStats leest; `FINSTATS_PUBLIC_IP_URL`, `FINSTATS_ALLOW_LIBRARY_SHRINK` en `FINSTATS_SKIP_PREUPDATE_BACKUP` ontbraken.
- De installatiewizard zei over een adres dat geen adres was dat hij ‘geen verbinding kon maken’, in een zin die doorliep in de volgende; nu zegt hij dat het adres er geen is.
- In Instellingen → Back-ups verdween een vraag die werd gesteld terwijl er een back-up werd geschreven, zodra de back-up klaar was.
- Met het menu als rail die openklapt, toonde het sluiten van zoeken tegelijk de open zoekfunctie en de gesloten rail, en liet daarna de rail open tot de aanwijzer kwam en weer ging. Esc geeft de focus van het toetsenbord nu terug aan waar hij was toen zoeken openging, en laat die van een klik los.
- Op een telefoon hield het sluiten van zoeken de pagina gedimd tot de zoekfunctie weg was, en sprong ze dan in één frame naar helder; het dimmen vervaagt nu mee.
- De stappen van de Tautulli-import zeiden dat de back-up wordt gedownload na **Backup Database**. Tautulli bewaart hem op zijn eigen machine: de stappen zeggen nu waar (de map `backups`, `/config` met Docker), welk bestand je moet nemen, en dat je het moet overzetten.
- Zoeken dat werd gesloten voordat het helemaal open was, liet de rail die openklapt zonder animatie achter: vanaf dan ging hij in één keer open en dicht.
- De rail die openklapt bleef open nadat er een pagina uit was gekozen, tot er ergens anders werd geklikt, en tekende in sommige browsers, of ingezoomd, een schuifbalk onder de pictogrammen. Hij klapt nu alleen open voor de aanwijzer en het toetsenbord, en tekent geen schuifbalk als hij dicht is.
- Op de pagina Beveiliging telt een adres dat de database wel in een land plaatst maar niet op de kaart, mee voor dat land, in plaats van bij de adressen zonder bekende plaats.

## [2.1.2] - 2026-10-01

### Added
- **Importeren uit Tautulli**: neem je Plex-geschiedenis mee. Upload de databaseback-up van Tautulli (de `.db`, of de `.zip` waarin hij zit) onder **Instellingen → Importeren**; de weergaven ervan krijgen een eigen plek in het filter *Vastgelegd door* van Activiteit.
- **Een schakelbord om te zeggen wie wie is**: Plex-gebruikers aan de ene kant, Jellyfin-gebruikers aan de andere. Trek een draad van elke Plex-gebruiker naar wie die nu is, of klik op de een en dan op de ander, of gebruik het toetsenbord. Wie geen draad heeft, wordt niet geïmporteerd, en twee Plex-accounts kunnen in één persoon samengaan.
- Films en afleveringen worden op naam aan je bibliotheek gekoppeld (een film op titel en jaar, een aflevering op serie en nummer), en een titel die nog niet op je server staat, wordt bewaard en gekoppeld zodra hij binnenkomt. Een hervatte kijkbeurt is één weergave; muziek wordt niet geïmporteerd.
- De back-up bevat de toegangstokens van Plex: FinStats leest ze nooit, en verwijdert het bestand zodra je importeert of opnieuw begint.
- **Niet-gekoppelde media** onder **Instellingen → Gegevens**: elke titel uit Tautulli, Jellystat of Streamystats die bij niets in je bibliotheek past, met waar hij vandaan kwam en waar hij waarschijnlijk al staat. **Koppelen** verbindt hem via de beste gissingen of een zoekopdracht (een film kan als special van een serie worden gekoppeld), en FinStats onthoudt de keuze, zodat dezelfde geschiedenis zichzelf koppelt als ze opnieuw wordt geïmporteerd. Keuzes horen bij een back-up.

### Performance
- **De Terugblik opent drie tot vijf keer sneller**: de meest bekeken acteurs en regisseurs van het jaar werden uitgerekend door het jaar voor elke persoon erin opnieuw te doorlopen. Gemeten op een echte geschiedenis ging een jaar van 0,29 s naar 0,09 s en de afgelopen 12 maanden van 0,53 s naar 0,12 s.

### Fixed
- **Geïmporteerde geschiedenis vindt titels die anders heten**: een weergave past ook op de naam in de oorspronkelijke taal, zodat een serie die Plex of Tautulli kende als 오징어 게임 aan Squid Game wordt gekoppeld, en *Im Westen nichts Neues* aan *All Quiet on the Western Front*. De oorspronkelijke titels van je bibliotheek worden bij de volgende scan gelezen.
- Namen worden vergeleken zoals zoeken ze vergelijkt (hoofdletters, accenten, leestekens, een jaartal in de naam, een ‘The’ vooraan), en een jaar van uitbrengen mag er één naast zitten, zodat ‘Star Wars: Episode I - The Phantom Menace’ ‘… – The Phantom Menace’ vindt en een Kingsman uit 2014 die uit 2015. Een titel die onder een heel andere naam bekendstaat, wordt nog steeds nooit geraden.
- Een mislukte import of terugzetting van Streamystats werd in meldingen bij zijn interne naam genoemd.
- De knop *Het jaar delen als kaarten* van de Terugblik en de kijkersrang ernaast hadden geen ruimte tussen elkaar.

## [2.1.1] - 2026-10-01

### Changed
- **De kijklijst wordt aangeboden overal waar een titel staat**: op de kaarten van Binnenkort op het dashboard, net als in Onderweg → Binnenkort, en op de pagina van een aflevering of seizoen, die dan de serie op de lijst zet.
- Op een poster op het dashboard verschijnt de kijklijstknop als je naar de kaart wijst of hem met het toetsenbord bereikt, in plaats van de hoek van elke poster te bedekken; op een touchscreen is hij er altijd.

### Stability
- Een koppeling met Sonarr, Radarr of Seerr opslaan terwijl iemand **Instellingen → Uitgaande verbindingen** open had, kon beide voorgoed op elkaar laten wachten.
- **Instellingen die niet kunnen worden gelezen, houden FinStats bij het starten tegen**, met de naam van de instelling die fout is, in plaats van elke instelling stilletjes naar de standaardwaarde terug te zetten.
- **Twee wijzigingen in de instellingen tegelijk** (de instellingenpagina en de planning van een taak, bijvoorbeeld) maken elkaar niet meer ongedaan.
- Een pauze, sprong of spoorwissel die werd gezien terwijl de database bezet was, wordt bij de volgende ronde geschreven in plaats van te verdwijnen.
- Het lezen van een bibliotheek mislukt niet meer op een kijklijst met meerdere items die één titel blijken te zijn.

### Fixed
- **Een vervangen bestand is niet ‘nu op de server’**: als het bestand van een film op iemands kijklijst wordt geüpgraded, krijgt diegene niet meer te horen dat de film net is binnengekomen.
- Eén mislukte leesronde van de plug-ins, geplande taken of opslag van Jellyfin maakt die lijst op de pagina Server niet meer leeg tot de volgende leesronde.
- **Een weergave uit Streamystats stopt op elke pagina waar ze stopte**: de afhaakcurve van een titel en het ‘uitgekeken’ van een aflevering gingen in plaats daarvan uit van de lengte, en telden haar als geschat.

## [2.1.0] - 2026-09-29 to 2026-09-30

Kijklijst

### Added
- **Een eigen kijklijst**: zet er een film of serie op vanaf de pagina ervan, vanuit Binnenkort, vanuit Onlangs toegevoegd of vanuit zoeken (Ctrl+Enter), en vind hem onder **Kijklijst** op je eigen profiel.
- **Elk item zegt hoe het er nu voor staat**: op de server, ‘5 van 26 afleveringen’, aangevraagd, ‘Komt vrijdag’, niet op de server, uit de bibliotheek verdwenen, of gekeken. Het wordt elke keer uitgerekend als je kijkt, met dezelfde opvatting van ‘gezien’ als je voortgang in series. Gekeken titels blijven staan, in een eigen groep, tot je ze eraf haalt.
- **Een titel die nog niet op je server staat**, kan rechtstreeks op de lijst worden gezet vanuit wat Sonarr of Radarr nog verwacht. Hij wordt vanzelf aan de titel gekoppeld zodra die binnenkomt, en een hernoemd bestand neemt zijn item mee.
- **‘Op je kijklijst: nu beschikbaar’**: een melding voor je eigen bestemmingen als iets waar je op wachtte binnenkomt. Uit tot je hem aanvinkt.
- **Niemand anders ziet je kijklijst**, ook een beheerder niet, en de melding ervan gaat alleen naar jouw bestemmingen. Er wordt niets naar Jellyfin, Seerr, Sonarr of Radarr geschreven.
- Kijklijsten horen bij een back-up, en terugzetten voegt ze samen zonder iets te verdubbelen.

### Changed
- Items van `/api/upcoming` bevatten de TVDB- en TMDB-id’s van hun titel, en `/api/auth/me` zegt of de server kijklijsten bijhoudt.

## [2.0.4] - 2026-09-29

### Added
- **Taken plannen zoals Jellyfin het doet**: elke taak die FinStats uit zichzelf doet, heeft triggers (dagelijks op een tijdstip, wekelijks op een dag, met een interval, bij het starten, of na de bibliotheekscan van Jellyfin), elk met een optionele tijdslimiet. Klik op een taak om de planning te zien en te wijzigen; *Trigger toevoegen* opent hetzelfde dialoogvenster dat Jellyfin heeft.
- **Taken is een eigen onderdeel** in Instellingen, met de laatste uitvoering van elke taak in woorden (‘Laatst uitgevoerd 38 minuten geleden, duurde 6 minuten’) en een knop Uitvoeren; het stond eerst bovenaan Systeem.
- **Metadatawijzigingen**: elk uur leest FinStats wat Jellyfin sinds de vorige keer heeft opgeslagen (namen, beschrijvingen, genres, beoordelingen, cast en crew, bestandsgegevens en afbeeldingen), zodat een bewerking in Jellyfin verschijnt zonder op een bibliotheekscan te wachten.
- Back-up en de geolocatiedatabase kunnen ook vanuit Taken worden uitgevoerd; Esc gaat van de planning van een taak terug naar de lijst.

### Changed
- **Het thema is een schakelaar met drie standen**, Licht · Apparaat · Donker, op de plek van de knop die ze een voor een doorliep: klik op een derde ervan, sleep het schuifje of gebruik de pijltoetsen. Het schuifje draagt het pictogram van de keuze, dat in beeld beweegt als die verandert.
- **Afmelden staat onder Instellingen → Account**, dat zegt wie hier is aangemeld; de zijbalk houdt de themaschakelaar.
- **De oude planningsinstellingen zijn nu triggers**: *Bibliotheekscan van Jellyfin volgen* met het reserve-interval, *Elke N dagen een back-up maken* en de GeoIP-schakelaar *De database actueel houden* zijn de bijbehorende triggers op hun taken geworden, met de waarden die je had. Verzamelen, Back-ups en Beveiliging linken in plaats daarvan naar de planning.
- Een start voert niet meer elke taak opnieuw uit: wanneer elke taak het laatst klaar was, wordt bewaard.
- Een taak van Jellyfin die nog geen schatting heeft, toont ‘nog ongeveer …’, met puntjes die rondgaan waar de tijd komt te staan, in plaats van ‘ETA…’.

### Performance
- **De kaart van Beveiliging hapert niet meer**: slepen en zoomen verschuiven de kaart die al getekend is en tekenen haar één keer als het gebaar tot rust komt, en de puls van wat nu speelt laat niet meer de hele kaart 120 keer per seconde opnieuw tekenen terwijl niemand eraan zit. Slepen ging op een trage computer van 34 naar 60 beelden per seconde.

### Fixed
- **Een poster of portret dat in Jellyfin is vervangen, verschijnt in FinStats**: afbeeldingen werden een week op de server en nog een week in de browser in de cache bewaard, wat Jellyfin intussen ook had. Ze worden nu bewaard onder de afbeeldingstag van Jellyfin, zodat een nieuwe afbeelding een nieuw bestand is, en de browser controleert elke keer dat hij er een toont of zijn kopie actueel is. Een portret vervangen slaat alleen de persoon opnieuw op, dus Metadatawijzigingen leest de personen ook.
- **Server → Taken toont een schatting zodra de pagina opent**: de resterende tijd werd alleen gemeten aan metingen die de pagina zelf deed terwijl ze openstond, dus elk bezoek begon met ‘ETA…’ tot ze het percentage had zien bewegen. FinStats meet een lopende taak van Jellyfin nu zelf, elke 10 seconden zolang er een loopt, en de pagina krijgt de schatting aangeleverd. Een taak die Jellyfin de hele tijd op 0% meldt (de eerste fase van een bibliotheekscan), houdt haar puntjes tot ze beweegt: er wordt niets geraden.
- **Tooltips op de kaart verschijnen op de stip**: in Firefox kon de tooltip van de stip van wat nu speelt ver weg terechtkomen, en waar stippen overlappen (iemand die thuis kijkt) hing het van één pixel af welke tooltip verscheen. Eén tooltip noemt nu alles onder de aanwijzer.
- **De statusbalk heeft één vorm bij aanwijzen**: elke link (de streams, Repo, de versie) licht op als hetzelfde kleine vakje, en de puntjes tussen de items staan midden in de tussenruimtes. Aanwijzen van de versie liet ook de punt en de ruimte ervoor oplichten.
- **Een bestandspad neemt de hele breedte van zijn kaart in**: op de kaart Bestand van een titel, en bij de mappen van Jellyfin op de pagina Server, werd een pad in één smalle cel geperst en liep het over negen regels naast lege ruimte.
- Een dialoogvenster dat uitfadet, slikt geen klik op de pagina eronder meer in.

## [2.0.3] - 2026-09-28

### Security
- De pagina Server gaf de id van elk apparaat aan iedereen met *De server zien*; voor id’s is ook *Netwerkdetails zien* nodig, zoals overal elders.
- Twee rechtenschakelaars die werden omgezet voordat de eerste was opgeslagen, stuurden twee keer dezelfde beginlijst, en de tweede maakte de eerste ongedaan: een ingetrokken recht bleef toegekend, een uitgeschakeld onderdeel van een openbaar profiel bleef gepubliceerd, terwijl beide schakelaars toonden wat er was gekozen.

### Stability
- **Maar één import of terugzetting tegelijk**: twee uploads die samen werden verstuurd, kwamen allebei door de controle, en terugzetten keek nooit naar imports; de tweede wachtte de database uit en mislukte na minutenlang uploaden.
- **Een adres met een kapotte `%`-escape opent de pagina Niet gevonden**, in plaats van eindeloos een draaiend laadicoontje te tonen.
- **Een groot paginanummer in een lijst is een lege pagina**: het getal liep over en kwam terug als de eerste pagina.

### Fixed
- **Tabellen sorteren getallen goed in een Duitse of Noorse browser**: ‘1,204’ werd gelezen als 1,204 met een decimale komma, en alles boven een miljoen als tekst.
- **De tabelweergave van een grafiek sorteert datums, weekdagen en posities op tijd**, niet op hun letters.
- **De downloadgeschiedenis noemt waar imports vandaan kwamen**: Sonarr en Radarr noemen de indexer en het protocol alleen bij het ophalen, dus die uitsplitsingen waren altijd leeg. Een mislukt seizoenspakket is één mislukking, niet één per aflevering.
- **Een samen gekeken weergave verwijderen** zoekt de groepen van die titel opnieuw, in plaats van een ‘groep’ van één over te laten.
- **Tijd samen is voor één persoon nooit meer dan diegene bleef**: de pagina Samen schreef diegene de hele avond toe.
- **Een avond in gezelschap over middernacht op oudejaarsavond** (of de rand van ‘afgelopen N dagen’) hoort geheel bij de periode waarin hij begon, in plaats van twee avonden alleen.
- **Geïmporteerde weergaven van afleveringen worden bij de naam van de aflevering genoemd** waar Jellystat alleen die van de serie bewaarde.
- **Geolocatie vindt de stadsdatabase naast de Country- en ASN-databases** die `geoipupdate` schrijft, in plaats van zichzelf uit te schakelen.
- **De plek die het vaakst wordt teruggespoeld volgt de filters van de pagina**, net als de tellingen.
- **‘Je jaar is klaar’** gaat alleen naar mensen van wie de Terugblik iets bevat.
- **Een gedempte waarschuwing opnieuw afhandelen houdt haar paar gedempt**; alleen heropenen maakt dempen ongedaan.
- **Een verlopen API-sleutel telt niet meer mee voor de twintig.**
- **De sessielimiet laat de oudste sessie gaan**, ook als veel aanmeldingen in dezelfde seconde vallen.
- **Een thuisadres dat het opzoeken heeft gevonden, blijft staan** als hetzelfde adres van de getypte lijst wordt gehaald.
- **Afbeeldingen in het auditlogboek en de sleutellijst**, waar alleen initialen stonden.
- **De meldingenlijst van een beheerder zegt van wie een persoonlijke bestemming is**, in plaats van ‘Van jou’ bij iedereen; **Onlangs verstuurd** toont elke bestemming van de laatste gebeurtenis.
- **Esc op een titel of persoon verlaat FinStats nooit**, en sluit alleen het bovenste dialoogvenster.
- **Een pagina wordt getekend, ook als opslaan het vooraf ophalen annuleert waar ze op wachtte.**
- **Speelt nu toont een gepauzeerde kijker die door de tijdlijn schuift** waar diegene nu is, niet pas als er op afspelen wordt gedrukt.
- **Een koppeling verwijderen werkt bij welke pagina’s er zijn**, net als er een opslaan.
- **De bouwdatum van de geolocatiedatabase houdt haar jaar** in Amerikaans Engels.

## [2.0.2] - 2026-09-28

### Security
- Achter een proxy telde de aanmeldlimiet het eerste `X-Forwarded-For`-adres, dat de client zelf meestuurt: per poging een nieuw verzonnen adres, en de limiet gold nooit. Nu is het het laatste, dat de proxy schreef.
- *IP-adressen en plaatsen meesturen* stuurde adressen naar een persoonlijke bestemming van iemand zonder *Netwerkdetails zien*, die geen enkele pagina diegene zou tonen. Voor de mislukte aanmeldingen van een ander is nu ook *De server zien* nodig, zoals op de pagina Beveiliging.

### Stability
- **Een noodstop gaat nooit verloren**: als een leesronde van de bibliotheek werd geweigerd in de eerste ogenblikken na het starten, voordat de server erop wachtte, werd de weigering genoteerd en draaide FinStats door in plaats van te stoppen.
- **Een Jellyfin die antwoordt in een vorm die FinStats niet kan lezen, wordt geweigerd** en niet geloofd: elke gebruiker, of elke titel van een bibliotheek, die binnenkwam onder sleutels die het niet kent (`id` voor `Id`), kwam met een volledige telling door de controle en werd daarna als verwijderd gemarkeerd.
- **Het einde van een weergave valt nooit weg**: als de database langer bezet was dan de wachttijd (een import die hem vasthield), bleef een weergave die op dat moment eindigde als spelend gemarkeerd tot de volgende herstart. Ze wordt nu bij de volgende ronde geschreven.
- **Instellingen die een terugzetting niet kan lezen, blijven ongemoeid**: een back-up waarvan de instellingen niet meer bij deze versie pasten, zette bij het terugzetten elke instelling terug: samenvoegvenster, rechten, thuisadressen, planning.
- **Een stuurteken in een naam of titel breekt de kaarten niet meer**: de profielkaart, elke verhaalkaart die die titel toonde en de ZIP ervan antwoordden met een fout.

### Fixed
- **Meldingen stoppen voor iemand die in Jellyfin is uitgeschakeld of verwijderd**: diens persoonlijke bestemmingen kregen nog steeds alles wat de oude rechten toestonden.
- **Een melding in de wachtrij wordt vóór het versturen opnieuw nagevraagd**: een uitgeschakelde bestemming, of een eigenaar die een recht was kwijtgeraakt, kreeg nog steeds wat op een nieuwe poging wachtte.
- **Een bestemming die altijd `Retry-After` antwoordt, wordt opgegeven** na vijf pogingen, zoals elke andere, in plaats van dertig dagen lang opnieuw te worden geprobeerd.
- **Apparaat-id’s op de pagina van een persoon** worden alleen getoond met *Netwerkdetails zien*, zoals overal elders.
- **Een gepubliceerd jaar wacht voor alles een dag**: de uitgekeken en afgehaakte series volgden elke weergave, zodat op afspelen drukken bij een afgehaakte serie de openbare pagina binnen enkele seconden veranderde.
- **Een aangevraagde serie telt als gekeken** als een van haar afleveringen dat is: elke aangevraagde serie gold als nooit gekeken, in het hoofdstuk Aanvragen en op de kaart Gevraagd.
- **Het hoofdstuk Samen van de Terugblik laat Live-tv weg**, net als de rest van het jaar: een kanaal dat samen aanstond, kon de toptitel worden, en het aandeel kon boven 100% uitkomen.
- **De kaart Dagen van ‘de afgelopen 12 maanden’ eindigt op de nieuwste dag**: het raster liet de meest recente drie of vier weken weg.
- **De nieuwe afleveringen van een serie worden één keer per avond aangekondigd**: afleveringen aan weerszijden van middernacht UTC werden op een server ten oosten of westen daarvan twee keer aangekondigd.
- **Seerr koppelen kondigt oude aanvragen niet aan als klaar om te kijken**: een aanvraag die bij de eerste leesronde al beschikbaar was, zonder ander gegeven over wanneer, gold als op dat moment binnengekomen.
- **De titels van een bibliotheek die in Jellyfin is verwijderd, gaan mee**: ze bleven in zoeken, op de plank Onlangs toegevoegd en in de totalen staan.
- **Een remux blijft direct streamen** als een meting even haar transcoderingsgegevens kwijt is (een sprong), in plaats van voor de rest van de weergave een transcodering te worden.
- **‘Jellyfin is onbereikbaar’ betekent dat leesrondes achter elkaar mislukten**: losse time-outs met dagen ertussen, op een server die rustig via de socket luisterde, telden daar samen toe op.
- **Een import uit Streamystats toont de voortgang op een eigen kaart**, niet over het laatste resultaat van de Jellystat-import.
- **Een adres waarvan de host met ‘web’ begint** (`http://web:8096`) wordt als host gelezen, en niet afgekapt als het pad van de webclient van Jellyfin.
- **Een fout in de wachtrij verbergt de hele inlog** van een adres, ook als het wachtwoord een schuine streep bevat.

## [2.0.1] - 2026-09-28

### Changed
- **De talen van een titel staan niet meer in de weg**: de kop noemt de eerste drie audio- en ondertiteltalen, elk op één regel, en een nieuwe kaart **Talen** somt ze allemaal op: de volledige bij elkaar, de gedeeltelijke met hoeveel afleveringen ze hebben. De twee dozijn ondertiteltalen van een lange serie duwden de achtergrondafbeelding een scherm omlaag.

### Fixed
- **De poster van een titel blijft naast de naam staan**: als de tekst ernaast langer was dan de poster (een lange samenvatting, veel studio’s), zakte de poster naar de onderkant van de kop, onder lege achtergrond.
- **Een versie die FinStats niet kan lezen, wordt zo genoemd**: een `app_version` die met de hand in iets onleesbaars was veranderd, liet FinStats weigeren te starten als ‘de oudere FinStats 2.0.1’, wat het niet was. Het weigert nog steeds, en zegt nu welke waarde het niet kon lezen en hoe die moet worden geschreven.

## [2.0.0] - 2026-09-25 to 2026-09-28

Terugblik 2026, openbare profielen en samen kijken

### Added
- **Met wie je keek**, in de Terugblik: avonden in gezelschap, de tijd die je daarin doorbracht, de titel die mensen het vaakst samenbracht, en tot drie kijkgenoten, in de app bij naam genoemd en gelinkt aan hun profiel.
- **Series tot het eind gekeken**, in de Terugblik: de laatste aflevering van elke serie die dat jaar is uitgekeken, volgens dezelfde opvatting van ‘gezien’ als de voortgangsbalken van het profiel, en ook de series die zijn begonnen en blijven liggen, minder dan de helft gezien en twee maanden niets afgespeeld.
- **Wat je aanvroeg**, in de Terugblik: aanvragen in Seerr van dat jaar, hoeveel er binnenkwamen en hoeveel je daarna keek. Alleen waar er aanvragen zijn.
- **Het jaar tegenover het jaar ervoor**: uren, weergaven en dagen, en hoeveel meer of minder.
- **Het jaar als verhaal**: elk hoofdstuk als kaart van 1080×1920, het formaat van verhalen op de telefoon (het jaar, de cijfers, topseries, films en muziek, genres, het kijkprofiel, uren en dagen, elke dag van het jaar, records, gezelschap, uitgekeken series, aanvragen en het jaar ervoor). Bewaar er één, of allemaal als ZIP, via ‘Het jaar delen als kaarten’.
- **Een kaart noemt nooit iemand anders**: kaarten worden getekend uit een kopie van het jaar zonder veld voor een andere persoon, een rang of een app. In de app draagt een kaart helemaal geen naam.
- **Het jaar van de server**, voor Jellyfin-beheerders: elke titel en elk totaal en het kijkprofiel van het huis, zonder dat iemand wordt gerangschikt of genoemd. Een schakelaar op de pagina Terugblik, `scope=server` in de API.
- **‘Je jaaroverzicht is klaar’**: een melding in december, één keer aan iedere persoon die dat jaar heeft gekeken.
- **Openbare profielen**: iemand kan een deel van zijn profiel publiceren op een link die iedereen zonder account kan openen (totalen en toptitels, reeksen en wanneer er wordt gekeken, het hele jaar als verhaal in kaarten, de laatste tien weergaven), elk deel uit tot de eigenaar het inschakelt onder Instellingen → Openbaar profiel. Apparaten, adressen, apps, afspeelmethoden, bestandspaden en andere mensen worden nooit gepubliceerd, wat er ook aanstaat, en de getoonde naam is er een die de eigenaar intypt, nooit diens Jellyfin-inlog.
- **Niets op een gepubliceerde pagina is nieuwer dan een dag**, recente weergaven en elk totaal net zo goed, zodat een pagina die iemand in de gaten houdt nooit zegt wie er op dit moment kijkt.
- **Deelbare kaarten**: bij elk gepubliceerd profiel hoort een afbeelding van 1200×630, van het profiel en van het jaar, die Discord, Signal, Mastodon en de rest tonen als de link wordt geplakt, en die kan worden gedownload om te posten. Op de server getekend uit wat de pagina toont en verder niets.
- **Een link die je kunt intrekken**: 22 willekeurige tekens, met één klik opnieuw in te stellen; de oude link antwoordt dan precies zoals een link die nooit heeft bestaan.
- **Openbare profielen staan uit tot een beheerder ze toestaat**: één schakelaar voor de hele server, een lijst van wie wat publiceert, en een manier om een profiel offline te halen. Publiceren, wijzigingen, opnieuw instellen en offline halen staan in het auditlogboek, en een sleutel kan niets openbaar maken.
- **Samen**, een eigen pagina: wie met wie kijkt, hoeveel, en hoe dat is veranderd. Tijd samen, avonden en het aandeel van de kijktijd in gezelschap, elk tegenover de periode ervoor; uren in gezelschap tegenover uren alleen per dag of week; elk paar mensen met hun avonden, tijd samen, wanneer ze het laatst keken en wat ze het meest kijken; de titels die het vaakst samen zijn gekeken; de recente avonden; en, voor wie iedereen mag zien, de tijd alleen van iedere persoon tegenover de tijd in gezelschap.
- Elk paar telt: een avond met z’n drieën telt voor elk van de drie paren, waar de kaart eerst alleen precies dezelfde groep mensen telde. De tijd samen van een paar is de kortste van de twee kijkbeurten.
- De kaart **Samen gekeken** op het dashboard en op een profiel linkt naar de pagina, en de kaart van een profiel zegt welk aandeel van de kijktijd van die persoon in gezelschap was.
- `GET /api/stats/groups` krijgt `pairs`, `series`, `people`, `previous`, `bucket`, en `watch_s` en `share` in de totalen; de sleutels die het al had, zijn ongewijzigd.
- **Waar mensen stoppen**, op de pagina van elke film en aflevering: het aandeel weergaven dat bij elke minuut nog kijkt, van iedereen aan het begin tot wie het einde haalde, met het terugspoelen en het aanzetten van ondertitels in die weergaven op dezelfde as gemarkeerd. De kaart zegt hoeveel stoppunten zijn gemeten (weergaven die FinStats vastlegde) en hoeveel geschat (geïmporteerde weergaven, op basis van hoe lang ze liepen), en heeft drie weergaven nodig voordat ze iets tekent.
- **Wie blijft kijken**, op de pagina van een serie: hoeveel mensen elke aflevering begonnen, op volgorde, en per aflevering een kolom **Uitgekeken**. ‘Iedereen haakt af bij aflevering drie’ zijn mensen die nooit op afspelen drukken bij aflevering vier.
- **Bestanden die nooit afspelen**, onder Afspelen: een film of aflevering die drie keer of vaker is gestart en nooit voorbij dertig seconden kwam, met wie het probeerde en in welke apps. Het logboek van Jellyfin bevat geen afspeelfouten, dus de weergaven zijn de getuigen; de minimale weergaveduur geldt hier niet, want die halen precies deze weergaven nooit.
- **Vaakst teruggespoeld** en **Ondertitels aangezet**, onder Afspelen: titels gerangschikt naar sprongen terug per weergave, met de minuut waar die zich ophopen, en naar het aandeel weergaven waarin binnen de eerste tien minuten ondertitels werden aangezet. Voor alle drie de lijsten is *Activiteit van iedereen zien* nodig. Sprongen korter dan 20 seconden werden nooit vastgelegd, dus een kort herhaald zinnetje is bewust onzichtbaar.
- Een sprong onthoudt waar hij vandaan kwam, als getal naast het label (`from_s` in de tijdlijn van een weergave). Elke sprong die al was vastgelegd, krijgt zijn getal uit het label, en een teruggezette back-up van hiervoor ook.
- `GET /api/stats/files` voor de drie lijsten; `GET /api/items/{id}` krijgt `insights`, en de afleveringen van een serie krijgen `users` en `finished`.
- **API-sleutels**: iedereen die is aangemeld, kan onder Instellingen → API-sleutels sleutels voor zichzelf maken, voor een script, een dashboard of een telefoon. Een sleutel wordt verstuurd als `Authorization: Bearer fs_…`, wordt één keer getoond, gehasht opgeslagen, kan een vervaldatum hebben en wordt met een klik ingetrokken; een beheerder ziet die van iedereen en kan ze intrekken. Hij staat voor dezelfde persoon met dezelfde rechten als een aanmelding, bij elk verzoek live gelezen, zodat wie het recht om zich aan te melden verliest, meteen ook elke sleutel kwijt is. Een sleutel kan geen sleutels maken of intrekken.
- **Een agendafeed**: `GET /api/calendar.ics?key=…` is wat Sonarr en Radarr eraan zien komen, als iCalendar waarop een telefoon zich kan abonneren. Een sleutel met het bereik *agenda* opent de feed en verder niets; de feed leest de sleutel uit het adres omdat een geabonneerde agenda geen header kan meesturen, nooit uit de cookie, en noemt niemand.
- **Een auditlogboek** onder Server → Audit, voor Jellyfin-beheerders: elke aanmelding en mislukte poging met het adres, elke instelling of elk recht dat is gewijzigd en waarin, elke sleutel die is gemaakt, gebruikt of ingetrokken, elke koppeling, bestemming, back-up en import, elke verwijderde weergave en afgehandelde waarschuwing, met wie, vanwaar, met welke sleutel en of het lukte. Een jaar bewaard, onderdeel van back-ups. `GET /api/audit` bladert erdoorheen.
- **Importeren uit Streamystats**, een eigen onderdeel onder Instellingen naast Jellystat. Neem het bestand van **Settings → Backup & Import → Download Backup** en zet het erin: jaren geschiedenis komen in een paar seconden binnen, en niets wat je al hebt, wordt opnieuw geteld. `finstats import-streamystats <file>` doet het zonder interface, `POST /api/import/streamystats` via de API; er loopt één import tegelijk, van welke soort ook.
- **Allebei de trackers gebruikt? Importeer beide bestanden.** Een weergave wordt herkend als een die FinStats al heeft aan de eigen id van de tracker ervoor en, als dat niet lukt, aan dezelfde persoon die hetzelfde item kijkt met een van beide uiteinden van de weergave dicht bij een uit de geschiedenis, zodat een export van Jellystat en een export van Streamystats van dezelfde avonden allebei kunnen worden geïmporteerd, in willekeurige volgorde, en een avond die FinStats zelf heeft gevolgd ook niet nog eens wordt geïmporteerd. ‘Dichtbij’ is het samenvoegvenster onder Instellingen → Verzamelen.
- Geïmporteerde rijen zeggen wat ze niet kunnen zeggen. Een weergave die Streamystats zelf volgde, bewaart niets over het bestand (geen codec, resolutie, bitrate of container), dus die blijven leeg in plaats van met een gok te worden gevuld, en de kaarten die ze zouden tonen, verbergen zichzelf. Een weergave die het uit Jellystat had overgenomen, bevat de hele sessie, en die wordt in plaats daarvan gelezen: gedateerd op het moment dat ze eindigde, het enige moment dat een van beide trackers bewaarde, en alleen als transcodering beschreven als de sessie er echt een was.
- Rijen die Streamystats schreef omdat Jellyfin een item als gekeken meldde, voor een kijkbeurt die het nooit zag, worden geteld en overgeslagen: ze zijn zo lang als de hele film en zouden kijktijd zijn op een avond waarop niemand keek. Het resultaat zegt hoeveel het er waren.
- Hoe gegevens van Streamystats worden geïnterpreteerd, veld voor veld, geleerd uit een echte export in plaats van uit documentatie: `docs/streamystats-import.md`.
- **Van welke tracker elke weergave kwam**, als filter in Activiteit: wat FinStats zelf vastlegde, wat uit Jellystat kwam, wat uit Streamystats kwam. Alleen de bronnen die je geschiedenis echt bevat, worden aangeboden, en het filter is beperkt zoals elke andere lijst, zodat iemand die alleen de eigen weergaven mag zien ook alleen hoort waar de eigen geschiedenis vandaan kwam.
- **De filters in Activiteit zijn keuzelijsten om aan te vinken**, zodat je films *en* afleveringen zonder muziek kunt opvragen, twee afspeelmethoden, of twee personen tegelijk, in plaats van één van elk of allemaal. Mediatype kreeg **Overig**, nu dat kan worden gecombineerd.
- Meerdere personen tegelijk, overal waar het gebruikersfilter staat (het dashboard, afspelen, bibliotheken, beveiliging en de rest), en de kaart en elke statistiek volgen. Een gebruikersfilter beperkt nog steeds alleen wat iemand mag zien: zonder *Activiteit van iedereen zien* wordt een verzoek vastgezet op wie het doet, wie het adres ook noemt en hoeveel.
- **Een licht thema**: washipapier, met inkt als tekst en een vermiljoen accent. FinStats volgt de lichte of donkere instelling van het apparaat tot je er een kiest.
- **Een themaknop naast Afmelden** wisselt tussen Apparaat, Licht en Donker. De keuze wordt in deze browser bewaard en toegepast voordat de pagina wordt getekend, zodat nooit even het andere thema verschijnt.
- **Openen in Jellyfin**: elke pagina van een film, serie en aflevering heeft een knop die de pagina van die titel in Jellyfin opent, in een nieuw tabblad.
- **Jellyfin-adres voor mensen** onder Instellingen → Jellyfin: waar die knoppen naartoe wijzen, voor als het adres waarmee FinStats verbinding maakt er geen is dat een browser kan bereiken. Leeg houdt het adres waarmee FinStats verbinding maakt; alleen een Jellyfin-beheerder kan het instellen.

### Changed
- **FinStats heeft een nieuw thuis**: `github.com/finstats/finstats`, en het image is `ghcr.io/finstats/finstats`. Wijzig de imagenaam in je `docker run` of je compose-bestand om updates te blijven krijgen; de gegevensmap blijft zoals hij is.
- **Instellingen toont één onderdeel tegelijk.** De pagina was veertien kaarten in één kolom en zeshonderd woorden uitleg. Nu heeft elk onderdeel een eigen adres (`/settings/collection`), een lijst links schakelt ertussen, en op een telefoon loopt de lijst door als chips. Links naar de oude ankers komen nog steeds goed aan.
- **Een instelling is één rij**: wat het is, één regel die zegt wat er geldt, het bedieningselement rechts. Getallen in een onderdeel worden samen opgeslagen met één knop onderaan; schakelaars slaan zichzelf nog steeds op. De statuskaarten (Taken, Uitgaande verbindingen, Database) en de link naar de licenties staan onder **Systeem**, en de twee importfuncties hebben een eigen onderdeel.
- **Zoek een instelling** via het vak in de kop (`/` zet de focus erop) op elk woord uit de naam; het zegt in welk onderdeel de instelling staat, en Enter springt naar de rij.
- **Instellingen is er voor iedereen die is aangemeld**: de onderdelen waarmee de server wordt beheerd, blijven voor wie hem mag beheren, en iemand die alleen meldingen mocht ontvangen, kan eindelijk bij de eigen bestemmingen.
- **De pagina Server toont één onderdeel tegelijk**, zoals Instellingen: Overzicht (systeem en opslag), Taken, Apparaten, Plug-ins, Logboek en Audit, elk op een eigen adres. Links naar `/server#jobs` komen nog steeds goed aan.
- **Het serverlogboek is een onderdeel van de pagina Server** (`/server/log`) in plaats van een eigen pagina in het menu; `/events` stuurt daarheen door, met de filters.
- Een melding over een mislukte taak, een mislukte back-up of een koppeling die niet meer antwoordt, linkt naar het onderdeel van de instellingen waar het onder valt, in plaats van naar de bovenkant van de pagina.
- Wat je bibliotheek over een geïmporteerde weergave kan vertellen (wat voor iets het was, welke aflevering, de speelduur, bij welke bibliotheek het hoort) wordt na **elke** leesronde van de bibliotheek ingevuld, niet alleen op het moment van een import. Geschiedenis wordt meestal geïmporteerd voordat FinStats de bibliotheek ooit heeft gelezen, dus die weergaven bleven voorgoed geraden of onbekend.
- Een back-up van FinStats terugzetten voegt geen weergave meer toe die van een tracker kwam onder een andere id, en houdt nog steeds twee echte kijkbeurten van hetzelfde met minuten ertussen.
- De details van een weergave noemen de tracker die haar binnenbracht, in plaats van alles wat geïmporteerd is een weergave uit Jellystat te noemen.

### Performance
- **Een herstart is bijna meteen klaar, hoe groot de geschiedenis ook is.** Elke start rekende over de hele geschiedenis uit wie samen keek en besliste opnieuw welke weergaven lokaal waren, allebei zonder iets te veranderen: 24 s en 5,5 s bij tien miljoen weergaven. Een start doet nu alleen wat de vorige keer is blijven liggen, en alles pas opnieuw na een update of als de instelling erachter is veranderd.
- **Een menigte die tegelijk binnenkomt, wordt in seconden vastgelegd.** Elke nieuwe weergave, elke voortgangsopslag en elk apparaat werd los geschreven, en elke beëindigde weergave werd afgesloten met een eigen blik op de hele geschiedenis van de titel; een ronde schrijft nu elke soort in één keer. 100.000 weergaven die samen beginnen, kosten 3 s, waar 25.000 die samen eindigden meer dan vier minuten kostten.
- **Speelt nu blijft snel met duizenden kijkers.** Uitzoeken wie samen kijkt, vergeleek elke stream met elke andere en noemde elke kijkgenoot bij naam: veertig seconden bij 10.000 streams, en gigabytes aan geheugen daarbovenop. Streams worden nu binnen een titel vergeleken, en elke stream toont een paar namen en hoeveel anderen.
- Uitrekenen wie samen keek, herschreef elke gegroepeerde weergave uit de geschiedenis telkens als FinStats startte, importeerde of een weergave zag eindigen; nu schrijft het alleen wat is veranderd. Uitzoeken welke weergaven waren geëindigd, wordt niet meer kwadratisch trager met het aantal dat speelt.
- **Herkennen van een weergave die de geschiedenis al heeft, gaat via een index**, zodat een import van 150.000 weergaven seconden duurt, en opnieuw koppelen na een leesronde van de bibliotheek van titel naar titel stapt in plaats van elke weergave te lezen terwijl de database bezet wordt gehouden.
- **Afspeelinzichten en het overzicht zijn sneller over alle tijd**: de onafhankelijke delen worden naast elkaar uitgerekend, de grafiek van gelijktijdige streams zoekt de lokale dag één keer per kwartier op in plaats van één keer per weergave, en genres worden per titel geteld. Bij een miljoen weergaven: van 4,1 s naar 2,4 s, en van 1,6 s naar 1,1 s.

### Stability
- **Een bibliotheek met televisie wordt niet meer gelezen als een leeggehaalde bibliotheek.** Een bibliotheek lezen gebeurt in twee rondes (elk item, en daarna een kleinere voor de cast en crew van alleen films en series), en de controle die beslist of een leesronde te vertrouwen is om te verwijderen wat ze niet zag, keek naar de tweede. Bij een bibliotheek met televisie of muziek liggen de twee aantallen ver uit elkaar, dus de controle weigerde elke leesronde en stopte FinStats, waarna de installatie herstartte en hetzelfde opnieuw deed. Er werd nooit iets ten onrechte verwijderd, maar geen enkele leesronde van de bibliotheek kon afronden. Als je bibliotheek met series of muziek geen nieuwe titels meer oppikte, is dit de reden.
- Een import kon mislukken met ‘database is locked’ als een andere schrijfactie (een weergave die werd bijgewerkt) tussen de eerste lees- en de eerste schrijfactie viel. Beide importfuncties nemen de schrijfvergrendeling bij het begin, en het databasejournaal blijft niet meer zo groot als de grootste import ooit.
- **Een databasebestand dat beschadigd, afgekapt, van iemand anders of alleen-lezen is, wordt meteen geweigerd**, met de naam van het bestand en de mededeling dat er niets is gewijzigd, in plaats van 30 stille seconden; in de database van een ander programma wordt nooit geschreven.
- Een back-up of momentopname van vóór een update die halverwege wordt onderbroken, laat geen bestand meer achter, dat de plaats van een goede momentopname tussen de drie bewaarde kon innemen.
- **De locatiedatabase overschrijven terwijl FinStats draait, laat het niet meer crashen**: FinStats leest uit een eigen kopie van het bestand.
- Een Jellyfin-app die elke keer een nieuwe apparaat-id meldde, kon het geheugen van FinStats laten groeien zolang het draaide; apparaten die vijf minuten niet zijn gezien, worden nu vergeten.
- Eén persoon houdt hoogstens 30 aangemelde sessies; de oudste wordt bij de volgende aanmelding afgemeld.

### Fixed
- **Dezelfde avond wordt niet meer twee keer geteld als een item is hernoemd.** Geschiedenis opnieuw koppelen aan een titel die in Jellyfin opnieuw was toegevoegd, kon een weergave die uit een tracker was geïmporteerd tot een duplicaat maken van een die er al was, omdat de controle die het had moeten opmerken liep voordat de id verhuisde. Die loopt nu opnieuw overal waar id’s opnieuw worden gekoppeld. Alleen geïmporteerde weergaven worden ooit verwijderd, nooit een die FinStats zelf vastlegde, en hoeveel er zijn verdwenen, staat in het logboek. De eerste start van 2.0 loopt de hele geschiedenis één keer na op de gevallen die er al zijn.
- **Een sessie volgt de persoon nu zoals Jellyfin die nu kent.** Een beheerder die Jellyfin had gedegradeerd, hield voor de rest van de sessie van 30 dagen volledige toegang tot FinStats, en iemand die in Jellyfin was uitgeschakeld of verwijderd, bleef aangemeld. Ze verliezen die nu zodra FinStats de gebruikers van Jellyfin leest (elke 15 minuten, of meteen vanuit Instellingen), zoals API-sleutels altijd al deden.
- Aanmelden schrijft wat Jellyfin over de persoon zegt in de gebruikerslijst van FinStats, zodat een geïmporteerde back-up van een tracker niet meer iemand beheerder kan maken voordat de eerste leesronde van de gebruikers dat rechtzet.
- Iemand die in Jellyfin is verwijderd, wordt als verwijderd gemarkeerd, ook als twee leesrondes van de gebruikers in dezelfde seconde vallen.
- **Downloadfouten tonen nooit het wachtwoord van een downloadclient of de API-sleutel van een indexer**, die Sonarr en Radarr daarin kunnen citeren.
- Weergaven die tijdens een leesronde van de bibliotheek opnieuw aan een hernoemde titel werden gekoppeld, werden tot de volgende herstart niet op samen kijken gecontroleerd; nu meteen.

## [1.6.5] - 2026-09-24

### Added
- **Vóór elke upgrade wordt automatisch een volledige back-up gemaakt.** De eerste keer dat een nieuwere FinStats je database opent, schrijft het er een volledige kopie van (je hele geschiedenis, instellingen en al) naar `data/pre-update-backups/` *voordat* het iets wijzigt. Gaat er ooit iets stuk bij een upgrade, dan zijn je gegevens veilig: stop FinStats, zet de kopie terug in plaats van `finstats.db`, start de versie die je had, en meld de bug. Er gaat niets verloren. De nieuwste paar kopieën worden bewaard; zet `FINSTATS_SKIP_PREUPDATE_BACKUP=1` om het uit te schakelen als de schijfruimte krap is.

## [1.6.4] - 2026-09-24

### Added
- **Vijf plekken meer om meldingen naartoe te sturen**: Telegram, Slack, Pushover, Pushbullet en e-mail, naast de bestemmingen webhook, Discord, ntfy en Gotify die er al waren. Elk vraagt alleen wat het nodig heeft (een chat-id, een gebruikerssleutel, een mailbox), en de drie die bij één dienst wonen (Telegram, Pushover, Pushbullet) kunnen alleen daar worden bereikt, zodat een token nooit naar een host kan worden gestuurd die erop lijkt. Mail gaat via TLS vanaf de eerste byte of via `STARTTLS` de deur uit, en er is geen derde mogelijkheid.
- **Licenties**, onder **Instellingen**: de eigen GPL van FinStats en elke licentie van derden waarop het is gebouwd (253 Rust-crates, de twee meegeleverde lettertypen, de kaartcontouren en de geolocatiedatabase), elke tekst volledig, zoals het eigen project hem schreef. Kies een onderdeel en de licentie opent in het venster naast de lijst; een crate die meerdere bestanden meelevert, wisselt ter plekke tussen die bestanden. De lijst wordt gegenereerd uit de crates waaruit FinStats werkelijk wordt gebouwd, zodat er geen afhankelijkheid kan worden toegevoegd zonder dat de licentie wordt vastgelegd.

## [1.6.3] - 2026-09-24

### Fixed
- Het vangnet grijpt nu ook in als Jellyfin *geen* bibliotheken of *geen* gebruikers meldt (een kapotte leesronde, nooit een gewone dag), wat de afstelling van 1.6.2 had doorgelaten. Een paar bibliotheken kwijtraken geldt nog steeds als gewoon; ze allemaal tegelijk kwijtraken niet.

## [1.6.2] - 2026-09-24

### Fixed
- Het vangnet voor de gegevensintegriteit uit 1.6.1 was te gretig: een bibliotheek die echt het grootste deel van haar items verloor (een handvol clips waarvan de bestanden weg waren, bijvoorbeeld), werd aangezien voor een kapotte leesronde, zodat FinStats weigerde haar bij te werken en stopte. Het grijpt nu alleen in bij een *duidelijk* kapotte leesronde: een bibliotheek die groot genoeg is om ertoe te doen en helemaal leeg terugkomt, of een grote die tot bijna niets is leeggehaald. Alledaagse wijzigingen worden toegepast zoals voorheen, en het geval ‘Jellyfin gaf niets terug’ wordt nog steeds opgevangen.

## [1.6.1] - 2026-09-24

### Added
- **FinStats vertrouwt geen leesronde van Jellyfin meer die je geschiedenis zou wissen.** Als een bibliotheek waarvoor FinStats duizenden items bewaart ineens leeg of bijna leeg terugkomt (een upgrade van Jellyfin die de API veranderde, een fout die zich voordoet als een leeg resultaat), houdt FinStats de gegevens die het heeft, markeert het de synchronisatie als mislukt (en stuurt het een melding), en stopt het netjes met een duidelijke boodschap in plaats van alles als verwijderd te markeren. Start opnieuw zodra Jellyfin weer zichzelf is, of zet `FINSTATS_ALLOW_LIBRARY_SHRINK=1` als je echt een bibliotheek hebt leeggemaakt. Hetzelfde vangnet geldt voor de lijst met bibliotheken en de lijst met gebruikers.

### Security
- Een `X-Forwarded-Host`-header die een client meestuurde, kon instaan voor een vreemde `Origin` en zo een schrijfactie vanaf een andere site langs de same-origin-controle krijgen. Die header wordt nu alleen vertrouwd achter een vertrouwde proxy (`FINSTATS_TRUST_PROXY`), dezelfde regel die het opzoeken van het clientadres al gebruikt. De sessiecookie met `SameSite=Lax` was en blijft de eerste verdedigingslinie.
- Een persoonlijke meldingsbestemming kon worden gericht op `0.0.0.0` of `::`, die deze machine bereiken. De controle die persoonlijke bestemmingen op openbare adressen gericht houdt, telt die nu als lokaal, naast loopback en de privébereiken.

### Fixed
- De pagina **Afspelen** laadt nu ook voor een bibliotheek met een weergave zonder vastgelegde methode (een rij die is teruggezet uit een back-up die ouder is dan de kolom); zo’n weergave telt gewoon als direct afspelen.
- Een heel lang woord in een zoekopdracht of filter geeft geen serverfout meer: SQLite weigert een `LIKE`-patroon van meer dan 50.000 tekens, dus filterwoorden worden vóór de query tot een redelijke lengte ingekort.
- De pagina **Instellingen** scrolt op een telefoon niet meer zijwaarts als een kaart een hele zin terugmeldt, zoals het resultaat van een terugzetting.

## [1.6.0] - 2026-09-24

Meldingen

### Added
- Kaart **Meldingen** onder Instellingen: voeg een bestemming toe, vink aan waarover ze moet horen, druk op Testen. Geen bestemming betekent dat er nergens een verzoek naartoe gaat.
- Vier soorten bestemmingen: een gewone **webhook** (JSON, beschreven in `docs/api.md`, optionele `Authorization`-header), **Discord** (een embed via de webhook-URL van een kanaal), **ntfy** (ntfy.sh of je eigen server, naar een topic) en **Gotify** (je eigen server, met een applicatietoken). Titels reizen mee in het bericht en niet in een header.
- Elf soorten gebeurtenissen, elk met een eigen vinkje: onmogelijke reis, een account dat in een nieuw land wordt gezien, een reeks mislukte aanmeldingen, een taak die mislukte, een back-up die niet kon worden geschreven, iets wat niet meer antwoordt (Jellyfin, Sonarr, Radarr of Seerr) en wanneer het weer antwoordt, nieuwe titels in de bibliotheek (samengevoegd per serie en dag, zoals de plank op het dashboard ze samenvoegt), een aanvraag die te kijken is geworden, en een weergave die begint of eindigt. De laatste twee staan standaard uit.
- Een aanvraag die te kijken is geworden, noemt de titel, wie erom vroeg en hoe lang diegene heeft gewacht.
- Persoonlijke bestemmingen: geef iemand **Meldingen ontvangen** en die kan er een eigen toevoegen, die alleen krijgt wat diegene al mag zien: de eigen aanvragen en waarschuwingen altijd, die van anderen met *Activiteit van iedereen zien*, adressen met *Netwerkdetails zien*. Een persoonlijke bestemming moet naar een openbaar adres wijzen; alleen een beheerder mag er een binnen het netwerk op richten.
- **IP-adressen en plaatsen meesturen**, per bestemming en standaard uit: een bericht over een onmogelijke reis noemt anders alleen plaatsen (‘Oslo, Noorwegen en Londen, Verenigd Koninkrijk’).
- **Onlangs verstuurd**, op dezelfde kaart: wat er is gezegd, naar welke bestemming, en wat de andere kant antwoordde als het niet aankwam. Een mislukt bericht wordt binnen ongeveer een uur vier keer opnieuw geprobeerd, en op een bestemming die verzoeken afremt, wordt gewacht.
- **Het adres van FinStats**, een instelling voor beheerders, is waaruit de link in een melding wordt opgebouwd. Leeg betekent dat berichten geen link bevatten.
- De eigen geplande taken van Jellyfin op de pagina Server: wat elke taak doet in woorden in plaats van met de naam van de code, en of het een van de zware is. Een lopende taak toont een voortgangsbalk, een percentage en hoeveel er nog over is; de rest toont wanneer ze de volgende keer lopen, wanneer ze het laatst liepen en of dat goed ging. Taken die Jellyfin op het eigen dashboard verbergt, staan er ook bij, en niets in FinStats kan een taak starten of stoppen.
- Hoeveel er van een taak over is, wordt gemeten aan het tempo waarin het percentage werkelijk is gestegen, tegenover de recentste meting die ver genoeg terug ligt om iets te betekenen, en niet aan het gemiddelde van de hele run, zodat een taak die versnelt of vertraagt wordt beschreven naar het tempo dat ze nu heeft.
- De takenlijsten die FinStats al om andere redenen leest (de controle op een bibliotheekscan elke 5 minuten, de servergegevens elke 15), voeden dezelfde meting, zodat een taak die al een uur loopt meestal een schatting heeft op het moment dat de pagina opent. Er wordt Jellyfin daarvoor niets extra’s gevraagd.
- Een lopende taak waarvan nog niets is gemeten, toont **ETA** met puntjes die rondgaan in plaats van een getal, en een taak waarvan het percentage een halve minuut niet is veranderd, toont ‘68% al 4m’ ernaast. Niets neemt de plaats in van een schatting die niet is gemeten.
- Een lijst van elke versie naast het Wijzigingslog: één klik gaat naar elke versie, de versie die je leest is gemarkeerd, en op een smal scherm wordt het een rij reeksen boven de notities.

### Changed
- Het Wijzigingslog klapt niet meer in. Elke reeks staat open en de lijst met versies is hoe je je verplaatst, in plaats van één groep tegelijk open en een klik om bij een andere te komen.
- **Instellingen → Uitgaande verbindingen** toont elke meldingsbestemming naast Jellyfin en de koppelingen met diensten: wat het is, de host, of het aanstaat, en wanneer het het laatst een bericht aannam.
- Niets aan het verzamelen, de geschiedenis of welk cijfer op het scherm ook verandert. FinStats *leest* nog steeds alleen uit Jellyfin, Sonarr, Radarr en Seerr.

## [1.5.2] - 2026-09-22

### Changed
- Een leesronde van de sessielijst van Jellyfin en het opnieuw starten van de klok die erom vroeg zijn één aanroep, zodat ze niet meer uit elkaar kunnen raken. Dat was de vorm van de fout met op hol geslagen verzoeken die werd gevonden toen 1.5.0 werd gebouwd.
- Dat bewijzen kost een fractie van een seconde in plaats van zeven minuten tegen een echte server, dus het wordt gecontroleerd telkens als er iets verandert. Dezelfde verzoeken, dezelfde geschiedenis, dezelfde cijfers op het scherm.

## [1.5.1] - 2026-09-22

### Fixed
- Een transcodering wordt één keer vastgelegd, niet één keer per seconde. Als een weergave begint met transcoderen en terugvalt naar direct afspelen, laat Jellyfin de transcoderingsgegevens op de sessie staan; elke meting werd vergeleken met een waarde die FinStats net had overschreven, dus elke meting leek een verandering en de pagina Activiteit liep vol met regels ‘Transcoderen’ met daaronder ‘Direct afspelen: …’.
- Regels die al waren geschreven, worden bij het starten opgeruimd: waar een reeks ervan hetzelfde zegt, blijft de eerste staan. Een weergave die moest worden getranscodeerd, telt nog steeds als transcodering.

## [1.5.0] - 2026-09-22

Sessies volgen via WebSocket

### Changed
- FinStats houdt vanaf het moment dat het start één verbinding met Jellyfin open, en zo hoort het van een weergave. 1.4.0 leverde het als instelling, standaard uit, om zich tegen echte servers te bewijzen; nu staat het altijd aan. Dezelfde weergaven, pauzes en sprongen, uitgerekend door dezelfde code.
- Er speelt niets: FinStats vraagt Jellyfin helemaal niets. Een avond waarop niemand kijkt, kost geen enkel verzoek.
- Er speelt iets: de sessielijst wordt één keer per seconde gelezen. Daar krijgt een pauze, een sprong of een spoorwissel zijn precisie, en zo wordt een weergave afgesloten waarvan de client zonder afscheid verdween.
- Alles wat geladen is, staat drie metingen achter elkaar op pauze: het vragen stopt en de verbinding neemt het over, waar een gepauzeerde film eerst elke seconde dezelfde vraag kostte, ongeveer 16 MB per uur. Op hervatten, of iemand anders die iets start, wordt binnen ongeveer een seconde gereageerd, en gepauzeerde tijd telt nog steeds nooit als kijktijd.
- Elke uitgaande leesronde vraagt om gecomprimeerde antwoorden en pakt ze uit (Jellyfin, Sonarr, Radarr en Seerr comprimeren allemaal als erom wordt gevraagd), wat meer dan de helft afhaalt van de twee grootste leesrondes: de sessielijst tijdens een weergave, en het lezen van de bibliotheek. De geolocatiedatabase wordt nog steeds gedownload zoals hij is opgeslagen.
- De kaart Jellyfin in Instellingen zegt welke van de twee helften draait, en de live-stip volgt of de verbinding iets doorgeeft in plaats van in welke helft je zit.
- Er draait maar één van de twee tegelijk: terwijl FinStats vraagt, zegt het Jellyfin te stoppen met versturen, en vraagt het om weer te beginnen op het moment dat de laatste weergave eindigt. De verbinding blijft de hele tijd open.
- Een Jellyfin die het abonnement met niets beantwoordt, wordt geloofd zolang hij keep-alives beantwoordt: FinStats zegt dat één keer, leest intussen zelf de sessielijst, en blijft geabonneerd tot de eerste push de zaak beslist. ‘Kan niet pushen’ en ‘was aan het herstarten toen we het vroegen’ lijken op elkaar, dus stilte is nooit een oordeel.
- De terugvaloptie is onaangeroerd: een verbinding die sluit, stil wordt of dit protocol niet blijkt te spreken, valt bij de volgende ronde terug op vragen met een timer en blijft opnieuw verbinden, met de intervallen **Terwijl iemand kijkt, controleer elke** en **Terwijl er niets speelt, controleer elke**.

### Added
- Wat de collector doet, op `GET /api/status`: met welke van de vier dingen hij bezig is (`idle_socket`, `playing_poll`, `paused_socket`, `fallback`), of de verbinding open en geabonneerd is, hoe vaak hij vraagt, hoeveel sessieleesrondes er de afgelopen minuut uitgingen, en wanneer dat het laatst veranderde. Beantwoord uit het geheugen (geen verzoek, geen query), en zonder namen, titels of aantallen.
- Die velden worden bij elke ronde met elkaar vergeleken, en een tegenspraak die aanhoudt, wordt in het logboek geschreven.
- Eén begrenzer onder elke sessieleesronde in het proces: hoogstens 2 per seconde en 70 per minuut, met alles daarboven vertraagd, geteld en hoogstens één keer per minuut gelogd. Normaal kijken blijft daar ruim onder.

### Fixed
- Live-updates vervangen het pollen echt. Ingeschakeld vroeg 1.4.0 nog steeds elke vijf seconden om de sessielijst en opende het de verbinding elke twintig tot veertig seconden opnieuw, omdat het de verbinding beoordeelde op hoe recent er een lijst was binnengekomen, en Jellyfin stuurt er een als er iets *verandert* en daartussen niets. Of de verbinding leeft, is nu of Jellyfin überhaupt antwoordt, gecontroleerd met een keep-alive ongeveer twee keer per minuut.
- Een sessielijst die binnenkwam terwijl FinStats tussen twee rondes zat, werd weggegooid, in de veronderstelling dat er anderhalve seconde later weer een kwam. Hij wordt nu bewaard en gebruikt.
- **Laatst gecontroleerd** onder Instellingen, en de kaart Uitgaande verbindingen, raken niet meer verouderd terwijl de verbinding stil is: het antwoord van Jellyfin op een keep-alive is wat ze tonen.

### Removed
- De instelling **Laat Jellyfin doorgeven wat er speelt** onder Instellingen → Verzamelen, en `live_socket` uit de instellingen-API.
- `socket_enabled` uit de status van de collector: ‘uitgeschakeld’ bestaat niet meer.
- `last_reconcile_at` uit de status van de collector, vervangen door `socket_live`.

## [1.4.1] - 2026-09-22

### Changed
- Seerr wordt alleen volledig gelezen als er iets is veranderd: elke vijf minuten vraagt FinStats om één rij, de laatst gewijzigde aanvraag, en de ronde eindigt daar als het die al kent. Dat is ongeveer een kilobyte tegenover ongeveer zestig voor een pagina van vijftig. Aanvragen die nog onderweg zijn, worden elk kwartier bekeken, alles wordt één keer per dag opgesomd, en een nieuwe aanvraag verschijnt nog steeds binnen vijf minuten.
- De wachtrijen van Sonarr en Radarr worden elke vijf seconden gelezen zolang een pagina ze toont, elke minuut zolang er iets in de wachtrij staat en elke vijf minuten als die leeg is, in plaats van de klok rond elke minuut. De pagina openen, een dienst koppelen of een nieuwe aanvraag in Seerr ververst ze meteen.

## [1.4.0] - 2026-09-21

Optionele WebSocket-sessies, en overal waar FinStats naartoe kan

### Added
- **Laat Jellyfin doorgeven wat er speelt** onder Instellingen → Verzamelen, uit tot je het aanzet: één verbinding draagt de sessielijst, en ongeveer 17.000 verzoeken per dag worden er bijna geen. De weergaven, pauzes, sprongen en groepen worden uitgerekend door precies de code die ze eerst uitrekende. Zolang het live is, doet FinStats nog één gewoon verzoek per minuut *terwijl er iets speelt*, en geen als er niets speelt. Een verbinding die sluit, vijftien seconden stil wordt of het protocol niet blijkt te spreken, valt bij de volgende ronde terug op de timer en blijft opnieuw verbinden.
- **Uitgaande verbindingen**, een kaart in Instellingen voor Jellyfin-beheerders: elke bestemming die FinStats kan bereiken (je Jellyfin, de dienst ‘wat is mijn IP’, de database van DB-IP, en elke Sonarr, Radarr of Seerr), met waar ze voor is, of ze aanstaat en wanneer ze het laatst antwoordde. Opgebouwd uit wat FinStats al weet, en ze toont alleen adressen: nooit een sleutel, nooit een basispad.
- **Nu opzoeken** onder Instellingen → Thuisnetwerk, voor de dag dat je openbare adres verandert.

### Changed
- Het opzoeken ‘wat is mijn IP’ loopt één keer, de eerste keer dat FinStats een adres nodig heeft, in plaats van elk kwartier.
- De kaart Jellyfin in Instellingen zegt hoe de collector op de hoogte wordt gehouden, en, als de verbinding aanstaat maar niets doorgeeft, waarom niet.

## [1.3.0] - 2026-09-20

Sonarr, Radarr en Seerr, op één pagina

### Added
- **Koppelingen** onder Instellingen, voor Jellyfin-beheerders: **Sonarr**, **Radarr** en **Seerr** (of Jellyseerr/Overseerr), meerdere van een soort, elk getest voordat het wordt opgeslagen. Je downloadclient hoeft niet apart te worden ingesteld, want Sonarr en Radarr praten er al mee en FinStats leest wat zij weten.
- **Aanvragen**: wie wat aanvroeg, hoe lang het duurde voor het binnenkwam, en of diegene het ooit heeft gekeken. Tegels voor wachtende titels en de gebruikelijke wachttijd, een grafiek van hoe dat per maand veranderde, en lijsten van wat weken geleden binnenkwam en nooit is afgespeeld, en wie het meest aanvraagt.
- **Binnenkort**: nieuwe afleveringen en filmreleases uit Sonarr en Radarr als agenda per dag, gemarkeerd met of *jij* die serie kijkt en, voor wie iedereen mag zien, wie nog meer. Een serie telt als gekeken als iemand er in de afgelopen vier maanden een aflevering van heeft afgespeeld. Een rij Binnenkort op het dashboard, één per profiel voor de series die die persoon kijkt, en wat er voor een titel aankomt op de eigen pagina ervan.
- **Downloads**: de live-wachtrij van elke Sonarr en Radarr (wat het is, hoe ver het is, hoe snel, wat er bij het importeren misging), met de persoon die erom vroeg ernaast. Een seizoenspakket is één regel, hoeveel afleveringen het ook bevat, en wat aandacht nodig heeft, staat bovenaan. Daaronder wat er in een week, een maand of een jaar binnenkwam per indexer, kwaliteit en downloadclient, en welke downloads mislukten. Het dashboard toont dezelfde lijst, ingekort.
- **Zien wat er downloadt**, een nieuw recht. Zonder dit zien mensen nog steeds hoe ver hun *eigen* aanvraag is en hoe lang het nog duurt, en verder niets: geen releasenamen, geen snelheden, geen andere downloads. Voor de aanvragen van anderen is *Activiteit van iedereen zien* nodig.

### Changed
- FinStats stuurt deze diensten `GET` en verder niets: er zit geen code in die een aanvraag kan goedkeuren, een zoekopdracht kan starten, of een download kan toevoegen, pauzeren of verwijderen.
- Hun API-sleutels worden in de eigen database van FinStats bewaard, nooit teruggestuurd naar de browser, nooit in een logboek geschreven en nooit onderdeel van een back-up. Er wordt geen enkele omleiding gevolgd, zodat een sleutel niet terecht kan komen op een adres dat je niet zelf hebt ingevuld, en certificaten worden gecontroleerd tenzij je dat voor één koppeling uitschakelt.

## [1.2.2] - 2026-09-20

### Added
- Audio- en ondertiteltalen bij elke film en aflevering (‘Audio: Japans · Engels’).
- Hoe ver een nasynchronisatie gaat: een serie of seizoen zegt hoeveel van de afleveringen elke taal hebben (‘Engels: 13 van 26 afleveringen’), en de afleveringenlijst heeft een kolom **Audio**.
- **Audiotalen** en **Ondertiteltalen** op bibliotheekpagina’s: hoeveel bestanden in elke taal kunnen worden afgespeeld.
- FinStats leest de bibliotheek na deze update één keer opnieuw om de talen op te pikken. Het zegt nooit ‘nagesynchroniseerd’, omdat Jellyfin de oorspronkelijke taal van een titel niet geeft, en een spoor zonder taaltag wordt vermeld als ‘Onbekend’.

## [1.2.1] - 2026-09-20

### Changed
- De afbeeldingen in de README zijn bij deze versie opnieuw gemaakt, met een nieuwe van de pagina **Beveiliging**. In de app verandert niets.

## [1.2.0] - 2026-09-20

Vanwaar mensen kijken, op een kaart

### Added
- **Pagina Beveiliging**, voor mensen die netwerkdetails en de activiteit van iedereen mogen zien: een wereldkaart met een stip voor elke plek waarvandaan je gebruikers kijken, groter naarmate er meer gebeurt. Thuis is groen, plekken van buitenaf zijn paars, live streams pulseren en mislukte aanmeldingen van buitenaf zijn rood. Sleep om te verschuiven, zoom met de knoppen, de toetsen of Ctrl + scrollwiel, en overlappende stippen smelten samen tot je inzoomt. Daaronder elke plek met de mensen, weergaven, kijktijd en aanmeldingen, en de landen naar aantal weergaven.
- **Onmogelijke reis**: een waarschuwing als één account op twee plekken wordt gezien die geen vlucht met elkaar verbindt, of op twee verre plekken tegelijk, met beide waarnemingen, de afstand, de tijd ertussen en de snelheid die daarvoor nodig was geweest. **Op kaart tonen** tekent de reis.
- **Nieuw land**: de eerste keer dat iemand afspeelt of zich aanmeldt vanuit een land waar diegene nog niet is gezien.
- Handel een waarschuwing af met een notitie, of demp een paar plekken voor die persoon, voor een VPN of een telefoon waarvan de provider in de hoofdstad zit. Waarschuwingen die in oude geschiedenis worden gevonden, na een import of op een nieuwe database, worden als afgehandeld opgeborgen in plaats van de lijst te overspoelen. Waarschuwingen en wat je erover hebt besloten, horen bij back-ups.
- Plekken komen uit een stadsdatabase die lokaal wordt gelezen: er wordt nooit een adres ergens heen gestuurd, en de kaart wordt getekend uit contouren die met FinStats worden meegeleverd in plaats van uit een kaartdienst. Download de gratis database van DB-IP vanaf de pagina Beveiliging of **Instellingen → Beveiliging**, laat FinStats hem maandelijks verversen (uit tot je het aanzet), of zet je eigen `.mmdb` (DB-IP, MaxMind GeoLite2-City) in `data/geoip/`. `FINSTATS_GEOIP_DB` wijst een bestand elders aan.
- **Instellingen → Beveiliging**: de database die in gebruik is, de maandelijkse update, en hoe snel (900 km/u) en hoe ver uit elkaar (500 km) twee waarnemingen moeten zijn om als onmogelijke reis te tellen. Stadsdatabases zitten er vaak een paar honderd kilometer naast, en daar is de afstand voor.

## [1.1.3] - 2026-09-20

### Added
- Ctrl+Space doorzoekt ook de cast en crew van alles in je bibliotheek, onder **Cast en crew**, met hun foto, of ze acteren of regisseren, en in hoeveel van je titels ze zitten; een keuze opent hun pagina. Net zo vergevingsgezind als het zoeken naar titels (woorden in elke volgorde, accenten genegeerd, een tikfout toegestaan), en als er meerdere even goed passen, komt wie in meer van je titels zit eerst.

## [1.1.2] - 2026-09-20

### Added
- **Onlangs toegevoegd** op het dashboard, boven de grafiek Activiteit: de 30 nieuwste aanwinsten als een rij posters die je zijwaarts kunt scrollen, met pijlen op een computer. Elk zegt wanneer het binnenkwam en wat het is (een film, een album, of ‘Seizoen 4 · 3 afleveringen’), en een enkele nieuwe aflevering zegt welke, met de eigen poster van het seizoen als het die heeft.
- **Shift + scrollwiel** en de pijltoetsen verschuiven één poster, Page Up en Page Down een scherm vol, Home en End gaan naar het ene of het andere eind, en vegen werkt. Het gewone scrollwiel scrolt nog steeds de pagina.
- Nieuwe afleveringen van dezelfde serie die op dezelfde dag binnenkomen, delen één item, en een hele serie die in één keer is toegevoegd, is één item (‘3 seizoenen · 60 afleveringen’), zodat één grote import niet al het andere van de lijst duwt.
- De rij negeert de periode en persoon van het dashboard, omdat ze over de bibliotheek gaat en niet over weergaven, en verbergt zichzelf zolang de bibliotheek leeg is.

## [1.1.1] - 2026-09-20

### Added
- Pagina’s openen meteen: even nadat FinStats is geladen, haalt het stilletjes op wat je waarschijnlijk als volgende opent (het dashboard, je profiel en tijdlijn, Activiteit, Gebruikers, Bibliotheken en elke bibliotheek, Afspelen, de serverpagina’s, de Terugblik en de actiefste mensen), zodat een pagina die je opent er meteen staat en op de achtergrond toch wordt ververst.
- Films, series en castleden worden alleen opgehaald als je er duidelijk naartoe gaat, omdat het er duizenden zijn: de aanwijzer op een link laten rusten, hem aanraken of hem met het toetsenbord bereiken haalt die ene pagina op.
- Het wacht tot de pagina die je hebt geopend klaar is met laden, werkt één pagina tegelijk terwijl de browser niets te doen heeft, pauzeert in een tabblad op de achtergrond, doet niets op een databesparende of heel trage verbinding, en wordt vergeten als je je afmeldt.
- Een kleine link **Repo** met het GitHub-logo in de statusbalk, naast de versie.

### Fixed
- De **Tijdlijn** vergat welke bibliotheken waren uitgeschakeld als de pagina opnieuw werd geladen of vanuit een bladwijzer werd geopend, hoewel de keuze in het adres stond.

## [1.1.0] - 2026-09-20

Een tijdlijn van alles wat je hebt gekeken

### Added
- **Tijdlijn**, naast **Overzicht** op elk profiel: je kijkgedrag als één spoor van vandaag terug tot de eerste weergave die FinStats kent, met een avond afleveringen samengevouwen tot één halte. Elke halte is een poster met wat er werd gekeken en wanneer (‘Seizoen 2 · Afleveringen 3-6’, een film en of die meer dan één zit kostte, of een album en hoeveel nummers), en opent de titel.
- Het spoor volgt de breedte van het venster: drie haltes per rij op een breed scherm, twee op een smaller, elke tweede rij in omgekeerde richting met een bocht die haar met de volgende verbindt, tot één rechte lijn op een telefoon. De eerste halte van elke maand draagt de maand.
- Oudere haltes laden vanzelf terwijl je scrolt, helemaal terug tot waar de geschiedenis begint.
- Schakelaars voor **Bibliotheken** boven het spoor laten weg wat je niet wilt zien, zoals muziek, en de keuze hoort bij het adres.
- Iets wat nog speelt, zegt **Speelt nu**.
- Je ziet je eigen tijdlijn; voor die van anderen is het recht om iedereen te zien nodig.

## [1.0.5] - 2026-09-20

### Added
- Bescherming tegen downgraden: de database onthoudt de nieuwste versie van FinStats die haar heeft geopend, en een oudere versie weigert erop te starten in plaats van stilletjes te werken met gegevens die ze niet helemaal begrijpt. De foutmelding zegt wat je moet doen. Versies tot en met 1.0.4 zijn uitgebracht voordat deze controle bestond.

## [1.0.4] - 2026-09-20

### Fixed
- In de details van een weergave was de kopieerknop naast een lange **Apparaat-id** op een eigen regel beland.
- Op de pagina Afspelen werd de laatste kolom van **Welke clients transcoderen** afgekapt (‘Transc…’); koppen van twee woorden lopen nu door op een volgende regel.

### Changed
- De schermafbeeldingen in de README zijn weer actueel; ze toonden nog versie 0.5.0.

## [1.0.3] - 2026-09-20

### Added
- De pagina van een aflevering toont de eigen afbeelding van die aflevering, die Jellyfin in de afleveringenlijst toont, in plaats van de poster van de serie. Een aflevering waarvoor Jellyfin geen afbeelding heeft, houdt de poster.

## [1.0.2] - 2026-09-20

### Fixed
- In het **Wijzigingslog** hadden de groepen v0.9 en v0.10 geen kop naast de versie. De kop van een groep is nu altijd de eerste zin van waar de eerste versie over ging, zodat een lange inleiding niet midden in een zin wordt afgebroken.

## [1.0.1] - 2026-09-20

### Fixed
- Het gepubliceerde image startte niet op een nieuwe machine: ‘unable to open database file: /data/finstats.db’, na een halve minuut te zijn blijven hangen. Docker maakt een ontbrekende map `data` aan als root, en FinStats draait als gewone gebruiker. De container maakt de map nu van zichzelf en schakelt dan over naar die gebruiker, zodat `docker run` de eerste keer werkt. Voor bestanden van een andere gebruiker dan 1000 stel je `PUID` en `PGID` in; `--user` werkt zoals voorheen.
- Een gegevensmap waarin echt niet kan worden geschreven, wordt meteen gemeld, met de opdracht die het oplost, in plaats van 30 seconden later als databasefout.

## [1.0.0] - 2026-09-20

Klaar voor iedereen

### Added
- Een kant-en-klaar image, nu op `ghcr.io/finstats/finstats`, voor 64-bits Intel/AMD- en ARM-machines (een Raspberry Pi 4 of 5 werkt): `:latest` is de nieuwste versie, een versietag blijft waar hij is, en `:edge` is de ontwikkelversie.
- Elke versie verschijnt op de pagina met releases van het project op GitHub, met deze notities.

### Changed
- Versienummers betekenen nu wat ze zeggen: updates binnen 1.x maken je gegevens, back-ups en instellingen niet stuk.
- Het Docker Compose-bestand en alle instructies gebruiken het gepubliceerde image. Bij een zelfgebouwde `finstats:latest` hoeft alleen de imagenaam te worden gewijzigd; de map `data` gaat ongewijzigd mee.

## [0.10.2] - 2026-09-20

### Fixed
- Op een telefoon konden Instellingen en de pagina van een serie zijwaarts scrollen: een label dat alleen voor schermlezers bedoeld was, ontsnapte uit zijn tabel, en de afleveringenlijsten en de castrij deden hetzelfde. Ze scrollen nu binnen zichzelf.
- De Terugblik scrolt niet meer zijwaarts als het achtergrondwoord van een hoofdstuk breder is dan de pagina.
- Een lange regel ‘kijkt op…’ onder **Speelt nu** eindigt op een beletselteken in plaats van midden in een letter te worden afgekapt.
- De poster voor elke serie op een profiel was voor schermlezers een tweede, naamloze link; hij is nu voor hen verborgen en de titel ernaast is de link.

## [0.10.1] - 2026-09-20

### Fixed
- Een back-up downloaden mislukte in de browser (‘the source file could not be read’ in Firefox) zonder dat de pagina zei waarom: FinStats comprimeerde een back-up, die al gecomprimeerd is, een tweede keer, en de dubbel ingepakte overdracht brak voor het einde af. Back-ups worden verstuurd zoals ze zijn, met hun grootte vooraf, zodat de browser echte voortgang kan tonen.

## [0.10.0] - 2026-09-20

Back-ups, en verhuizen naar een nieuwe installatie

### Added
- Elke week een back-up, de nieuwste vijf bewaard: elke weergave met de tijdlijn van pauzes en sprongen, gezien-markeringen, rechten, thuisadressen, het serverlogboek en je instellingen, in één klein bestand, een paar honderd kB voor duizenden weergaven. **Instellingen → Back-ups** somt ze op met elk een knop **Downloaden**, maakt er een op verzoek, en stelt in hoe vaak en hoeveel. Ze staan in de map `backups` van je gegevensmap. Een back-up bevat geen Jellyfin-API-sleutel en geen aanmeldsessie, maar wel een volledige kijkgeschiedenis met IP-adressen, dus bewaar hem op een privéplek.
- Terugzetten voegt samen in plaats van te overschrijven: weergaven die er al zijn, worden overgeslagen, dus het kan veilig twee keer, of in een installatie die al een tijdje draait. Vink ‘Ook instellingen en rechten terugzetten’ uit voor alleen de geschiedenis. Back-ups werken tussen versies in beide richtingen. Vanuit een terminal: `finstats backup` en `finstats restore <file>`.

### Changed
- Het verzamelen controleert elke seconde terwijl iemand kijkt en elke 5 seconden terwijl niemand kijkt, waar het eerst de hele tijd elke 5 seconden was, zodat pauzes, sprongen en spoorwissels tot op de seconde worden vastgelegd. Beide staan onder **Instellingen → Verzamelen**; een eerder gewijzigd interval moet opnieuw worden ingesteld.

### Fixed
- Een IP-adres in de details van een weergave brak midden in het adres over twee regels. De markering Lokaal/Op afstand schuift eronder als er geen ruimte is.

## [0.9.1] - 2026-09-20

### Fixed
- Mensen thuis werden als op afstand getoond. Een apparaat op je eigen netwerk dat Jellyfin via de openbare naam bereikt, zoals een reverse proxy of een domein, komt binnen met het openbare IP-adres van het huishouden, en alleen privéadressen telden als lokaal. FinStats leert nu het openbare adres en telt die weergaven over de hele geschiedenis als lokaal, en het onthoudt eerdere adressen.

### Added
- **Instellingen → Thuisnetwerk**: de adressen die FinStats als thuis beschouwt, je eigen toevoegen (een eerder adres, een tweede huis, een VPN-uitgang), of het opzoeken uitschakelen. Het opzoeken vraagt elk kwartier een eenvoudige dienst ‘wat is mijn IP’, en probeert er meerdere (Amazon, Cloudflare en andere) omdat DNS die advertenties blokkeert, zoals Pi-hole, ze vaak tegenhoudt, en een ervan heeft helemaal geen DNS nodig. Het verzoek bevat niets over jou of je server, en met de schakelaar uit praat FinStats met niets anders dan Jellyfin. `FINSTATS_PUBLIC_IP_URL` stelt een eigen dienst in.

## [0.9.0] - 2026-09-20

Elke tabel, in de volgorde die jij wilt

### Added
- Elke tabel sorteert: klik op een kop om erop te sorteren, nog een keer om de volgorde om te draaien, een derde keer voor de oorspronkelijke volgorde. De betekenis van een cel wordt begrepen, dus ‘3d 2u’ sorteert als tijd en ‘1,4 GB’ als grootte, en lege cellen komen achteraan. Het geldt voor de gebruikerslijst, kijkers, afleveringen, apparaten, plug-ins, geplande taken, de tabelweergave van elke grafiek, de rechtentabel, en de staaflijsten (codecs, resoluties, clients en de rest), die een eigen smalle kop krijgen.
- Activiteit en het Serverlogboek sorteren over al hun pagina’s in plaats van alleen de rijen op het scherm, en de volgorde staat in het adres, zodat ze een herlaadbeurt of een gedeelde link overleeft.
- Tabellen van tien rijen of meer krijgen een klein filterveld erboven.

## [0.8.2] - 2026-09-20

### Changed
- De jaarkiezer van de Terugblik is dezelfde schakelaar als elke andere periodekiezer in FinStats (7d · 30d · 90d …).

## [0.8.1] - 2026-09-19

### Added
- Pagina’s voor acteurs en regisseurs, vanuit ‘Meest gekeken personen’ in de Terugblik of de nieuwe rij **Cast en crew** bij elke film, serie of aflevering: alles waarin ze op je server te zien zijn, hoeveel daarvan is gekeken en door wie, en wat er nog wacht. De pagina volgt de periode die je kiest, wie alleen de eigen statistieken mag zien, ziet daar alleen het eigen kijkgedrag, en Esc brengt je terug.

## [0.8.0] - 2026-09-19

De Terugblik heeft een nieuw uiterlijk, en meer te vertellen

### Added
- **Meest gekeken personen**: de acteurs met wie je de meeste tijd doorbracht en de regisseurs achter wat je keek, als een rij portretten. Cast en crew worden samen met de bibliotheek gelezen en verschijnen na de eerste leesronde van de bibliotheek na deze update.
- **Jouw jaar in dagen**: elke dag van het jaar als één vakje, helderder naarmate je meer keek, met je langste reeks en drukste dag erboven. Wijs een dag aan, of loop met de pijltoetsen door de kalender, om te zien wat erin zat.
- **In cijfers**: weergaven, kijktijd, verschillende titels, dagen gekeken, langste reeks en hoeveel er **herbekeken** werd, plus hoe het jaar zich verdeelt over afleveringen, films en muziek.
- Het hoofdstuk Genres zegt hoeveel genres je aanraakte en hoe groot het aandeel van het grootste was.

### Changed
- Een nieuw ontwerp: het opent op je jaar ‘opnieuw afgespeeld’, de kop naast de posters die het vulden, boven een golfvorm van het jaar met één balk per week en de luidste verlicht, en je kiest het jaar er direct boven. Elk hoofdstuk heeft een kop met het kernwoord in violet, een zin die de cijfers draagt, en het woord van het hoofdstuk groot en vaag erachter. Het eindigt zoals films eindigen, met de aftiteling: met in de hoofdrol, geregisseerd door, vertoond op, speelduur.
- Uren, weekdagen en maanden delen één hoofdstuk **Kijkpatronen** met een schakelaar ertussen, en de kop volgt waar je naar kijkt (‘Vrijdag kreeg de kroon’).

### Fixed
- **Speelt nu** op het dashboard bleef leeg als niemand keek, zodat de kop bij de filters eronder leek te horen. Het vak ‘Er speelt nu niets’ is terug.

## [0.7.10] - 2026-09-19

### Added
- Bibliotheekafbeeldingen: elke bibliotheekkaart, en de eigen pagina van de bibliotheek, toont de afbeelding die Jellyfin ervoor gebruikt, via de afbeeldingsproxy van FinStats zoals de posters. Een bibliotheek zonder afbeelding houdt haar pictogram.

## [0.7.9] - 2026-09-19

### Changed
- De kaart Genres opent weer als **lijst**, met de radar als tweede optie op de schakelaar. Een bestaande keuze blijft behouden.

## [0.7.8] - 2026-09-19

### Added
- Een radarweergave voor de kaart Genres, op het dashboard en op profielen: één spaak per genre, te wisselen tussen **Radar** en **Lijst** in de hoek van de kaart, en de keuze wordt onthouden. Een punt aanwijzen, of de pijltoetsen gebruiken, toont de kijktijd en het aandeel van dat genre.

## [0.7.7] - 2026-09-19

### Fixed
- Films die bij een collectie horen, werden getoond als ‘Niet meer in bibliotheek’. Met *group movies into collections* ingeschakeld geeft Jellyfin de collectie in plaats van de films erin, dus FinStats zag ze nooit; op één server verborg dat 556 van de 1.238 items in de bibliotheek Films. FinStats vraagt nu om de films zelf, en de leesronde die deze update start, brengt ze terug en koppelt hun weergaven opnieuw.

## [0.7.6] - 2026-09-19

### Changed
- Op profielen is de kaart **Series** omlaag geschoven tot net boven Genres, zodat een lange lijst series de kijktijd, de activiteitsgrafiek en de toptitels niet meer uit beeld duwt. De dagreeksen blijven bovenaan, en een serie die je hebt opengeklapt, blijft open als je de periode wijzigt.

## [0.7.5] - 2026-09-19

### Changed
- Zoeken vergelijkt woord voor woord, in elke volgorde, met alles ertussen, in plaats van de exacte zin nodig te hebben, dus ‘alya hides’ vindt *Alya Sometimes Hides Her Feelings in Russian*. Hoofdletters, accenten en leestekens doen er niet toe, een ‘The’ vooraan wordt genegeerd, en een tikfout wordt vergeven (‘mentalsit’ vindt *The Mentalist*); woorden van drie letters of minder moeten nog steeds goed worden getypt. Betere overeenkomsten komen eerst, en muziek wordt ook op artiest gevonden.
- De zoekvakken in Activiteit en het Serverlogboek zoeken woord voor woord in al hun kolommen: ‘alya opera’ vindt weergaven van die serie in Opera.

## [0.7.4] - 2026-09-19

### Changed
- Het Wijzigingslog is gegroepeerd per reeks: elke reeks `0.x` is één uitklapgroep met de versies erin, met waar de reeks over ging, hoeveel het er zijn en wanneer. Alleen de nieuwste staat open, en een dichte groep laat nog steeds zien welke versie je draait.

## [0.7.3] - 2026-09-19

### Fixed
- De klok van Speelt nu haperde om de paar seconden: de verversing elke 5 seconden tekende hem ook opnieuw, dus als de server een fractie voorliep, sprong het getal te vroeg en stond het daarna een seconde stil. Alleen de tik van eens per seconde laat hem nu bewegen, met precies één seconde (gemeten 999 tot 1001 ms uit elkaar), en de verversing corrigeert nog steeds na een pauze, een sprong of echte afwijking, en landt dan op de tik.

## [0.7.2] - 2026-09-19

### Added
- Samen kijken, live: elke kaart van Speelt nu zegt ‘Met maria’ terwijl mensen samen hetzelfde kijken, waar een groep eerst pas werd herkend als de weergaven waren afgelopen. Live tellen mensen als samen als ze op dezelfde titel zitten en ofwel binnen het groepsvenster begonnen, ofwel op bijna dezelfde positie zijn, wat ook werkt als FinStats midden in een stream is herstart.

## [0.7.1] - 2026-09-19

### Changed
- Speelt nu telt elke seconde: de klok tikt en de balk glijdt ertussen, in plaats van te springen telkens als de server wordt bevraagd. De server wordt nog steeds elke 5 seconden bevraagd en blijft de bron van de waarheid: de weergave synchroniseert opnieuw bij een pauze, een sprong of als de server voorloopt, maar niet voor een positie die maar een paar seconden oud is, omdat clients maar ongeveer elke tien seconden iets melden. Gepauzeerde streams staan stil.

## [0.7.0] - 2026-09-19

Wie kijkt er samen

### Added
- Samen kijken: als verschillende mensen dezelfde titel op hetzelfde moment starten, telt FinStats dat als samen kijken. Een kaart **Samen gekeken** op het dashboard toont de groepen, de titels die ze delen en hun uren samen, elk profiel toont met wie die persoon het meest kijkt, gedeelde weergaven dragen een klein personenteken in Activiteit, en de details van een weergave zeggen met wie hij werd gekeken. Het werkt op je hele geschiedenis, geïmporteerde weergaven inbegrepen.
- Een instelling voor hoe dicht de starts bij elkaar moeten liggen, standaard 60 seconden: ongeveer een derde van de echte groepssessies begint 6 tot 60 seconden na elkaar. Mensen moeten ook een paar minuten naast elkaar blijven kijken, zodat twee mensen die toevallig dezelfde aflevering openen geen groep zijn.

### Changed
- Zoeken opent met **Ctrl + Space** in plaats van Ctrl/⌘ + K.

## [0.6.1] - 2026-09-19

### Added
- Beheerders kunnen de Terugblik van een ander openen, bovenaan de pagina Terugblik of met **Terugblik openen** op diens profiel. De tekst noemt die persoon dan bij naam in plaats van ‘je’.

### Changed
- Ieder ander ziet nog steeds alleen de eigen Terugblik, welke rechten diegene ook heeft, en er is nog steeds geen Terugblik van de hele server.

## [0.6.0] - 2026-09-19

Je profiel, en waar je bent in elke serie

### Added
- Voortgang in series op profielen: elke serie die je hebt aangeraakt als een balk met één segment per aflevering (gezien, begonnen of nog niet), per seizoen open te klappen, gesorteerd op hoe dicht je bij het einde bent, met aparte weergaven voor uitgekeken series en alles.
- **Als gezien markeren** op je eigen profiel, per aflevering, seizoen of serie, voor wat je keek terwijl er niets werd vastgelegd. De eigen gekeken-markeringen van Jellyfin worden ook gebruikt. Markeringen blijven in FinStats en veranderen nooit iets in Jellyfin; een vastgelegde weergave kan niet worden ontmarkeerd, je eigen markeringen wel.
- Dagreeksen op profielen: je langste reeks dagen achter elkaar met een weergave, je huidige reeks, en op hoeveel dagen je iets hebt gekeken.
- **Mijn profiel** in de zijbalk.
- Activiteit toont waar het afspelen stopte (bijvoorbeeld `44:15 / 45:00`) onder de voortgangsbalk, en de datum en tijd onder ‘3 u geleden’. Weergaven die uit Jellystat zijn geïmporteerd, hebben geen stoppositie, omdat Jellystat die nooit vastlegde.

### Changed
- Afleveringen die ontbreken of nog niet zijn uitgezonden, worden niet meer als deel van je bibliotheek behandeld. Jellyfin vermeldt ze als plaatshouders zonder bestand, en ze telden mee in de aantallen afleveringen.

## [0.5.1] - 2026-09-19

### Added
- Een logo: een afspeelknop in drie grafiekbalken gesneden, in de zijbalk en op de aanmeldschermen, als scherpe SVG met een `favicon.ico` in meerdere formaten voor browsers en bladwijzerbalken die er een willen, plus een pictogram voor het beginscherm van telefoons en tablets.

## [0.5.0] - 2026-09-19

Jij bepaalt wie wat ziet

### Added
- Rechten onder Instellingen → Toegang, voor iedereen tegelijk of per persoon: **Activiteit van iedereen zien** (statistieken en geschiedenis van anderen, de pagina Gebruikers, elke live stream), **Netwerkdetails zien** (IP-adressen, apparaat-id’s, lokaal of op afstand), **De server zien** (de pagina Server, het serverlogboek, mislukte aanmeldingen, bestandspaden) en **FinStats beheren** (instellingen, taken, importeren, weergaven verwijderen).
- Aanmelden per persoon, om een paar mensen toe te laten zonder FinStats open te zetten voor elke Jellyfin-gebruiker.
- Wijzigingen gelden meteen, ook het intrekken van toegang, en alles wordt door de server afgedwongen in plaats van door knoppen te verbergen.

### Changed
- Jellyfin-beheerders hebben nog steeds alles en zijn de enigen die rechten kunnen wijzigen; iemand die FinStats mag beheren, kan niets toekennen, niet aan zichzelf en niet aan een ander.
- De jaarlijkse Terugblik blijft persoonlijk, wat er ook wordt toegekend.
- Voor bestaande installaties verandert niets tot je iets toekent.

## [0.4.5] - 2026-09-19

### Changed
- FinStats valt onder de **GNU General Public License v3.0**. Er is nooit een eerdere versie gepubliceerd, dus er is niets onder andere voorwaarden uitgebracht. De licenties van de meegeleverde lettertypen worden ernaast meegeleverd.

## [0.4.4] - 2026-09-19

### Added
- Esc gaat een stap terug: op een geopende bibliotheek, gebruiker of titel brengt het je terug naar waar je vandaan kwam, met de lijst precies zoals je hem achterliet. Wat er openstaat, sluit eerst (details van een weergave, zoeken, een keuzelijst, het mobiele menu), en Esc wordt met rust gelaten terwijl je typt of een grafiek met het toetsenbord bedient.

## [0.4.3] - 2026-09-19

### Changed
- FinStats onthoudt wat elke pagina het laatst toonde: teruggaan naar een profiel, een bibliotheek of een periode die je al hebt bekeken, toont die meteen, profielfoto en al, terwijl de nieuwe cijfers erachter laden, en de pagina wordt alleen opnieuw getekend als er iets is veranderd. Wat wordt onthouden, leeft in het browsertabblad en wordt weggegooid als je je afmeldt of iets wijzigt.
- Laadplaatshouders verschijnen alleen als het laden echt traag is. Eerst toonde elke pagina ze minstens een derde seconde.

### Fixed
- Bibliotheken die in Jellyfin zijn verwijderd en nooit een weergave hadden, worden niet meer vermeld. Een verwijderde bibliotheek met geschiedenis wordt nog steeds getoond, gemarkeerd als verwijderd.

## [0.4.2] - 2026-09-19

### Fixed
- Op de pagina Gebruikers stond het label **Beheerder** op een eigen regel en maakte het die rij hoger dan de andere. Labels staan nu naast de naam.

## [0.4.1] - 2026-09-19

### Fixed
- Het woord ‘null’ verschijnt buiten december en januari niet meer onder de titel van het dashboard, als er geen banner van de Terugblik te tonen is.

## [0.4.0] - 2026-09-19

Zie wat er veranderd is zonder de app te verlaten

### Added
- Tabblad **Wijzigingslog** met elke versie, en de versie die je draait gemarkeerd.
- De versie in de statusbalk linkt ernaartoe, en het tabblad toont na een update een stip tot je hebt gekeken.

### Changed
- Geschiedenis opnieuw koppelen aan hernoemde items gaat veel sneller: van 18 seconden naar een derde seconde bij 666 verweesde titels, zodat het starten direct blijft terwijl de geschiedenis groeit.

## [0.3.3] - 2026-09-19

### Changed
- De bibliotheek wordt opnieuw gelezen direct nadat de taak **Scan Media Library** van Jellyfin klaar is, nooit met een aparte timer en nooit terwijl er een scan loopt. FinStats heeft Jellyfin altijd alleen gelezen, en het start daar nooit een scan.
- Een wekelijkse leesronde als vangnet dekt servers die op realtime bewaking vertrouwen, waar de scantaak misschien nooit loopt.
- Het oude interval is nu alleen een terugvaloptie, voor als volgen is uitgeschakeld of de server geen scantaak meldt.

### Added
- Schakelaar **Bibliotheekscan van Jellyfin volgen** onder Instellingen → Verzamelen, standaard aan.

## [0.3.2] - 2026-09-19

### Fixed
- Geschiedenis overleeft hernoemen. Jellyfin geeft een hernoemd bestand een nieuwe id, waardoor de eerdere weergaven ervan bleven hangen onder de kale mapnaam (`Title (2010) [tmdbid-…] [imdbid-…]`) zonder poster. Die weergaven worden opnieuw gekoppeld aan het huidige item in de bibliotheek: op TMDB-, IMDb- of TVDB-id als de oude naam er een bevat, anders op opgeschoonde titel en jaar, en afleveringen op serie plus seizoens- en afleveringsnummer. Een overeenkomst moet eenduidig zijn, dus titels die echt zijn verwijderd, blijven zoals ze zijn.
- Live-tv-kanalen die uit Jellystat waren geïmporteerd, werden als films geteld, omdat Jellystat geen itemtype vastlegt. Ze worden herkend als Live-tv, en nieuwe weergaven van Live-tv worden vastgelegd met het eigen type van Jellyfin.

### Changed
- Live-tv blijft helemaal buiten de Terugblik: toplijsten, totalen, kijkprofiel, records en rang.

### Added
- `finstats relink` om geschiedenis op verzoek opnieuw te koppelen. Het loopt ook bij het starten en na elke leesronde van de bibliotheek.

## [0.3.1] - 2026-09-19

### Changed
- De Terugblik is strikt persoonlijk: iedereen, beheerders inbegrepen, ziet alleen de eigen, en de weergave ‘Iedereen’, de gebruikerswisselaar en het serverhoofdstuk zijn verdwenen.
- De Terugblik opent op het jaar dat ‘klaar’ is: het huidige jaar in december, anders het vorige. Andere jaren en ‘Afgelopen 12 maanden’ blijven één klik verwijderd.

## [0.3.0] - 2026-09-19

Je jaaroverzicht

### Added
- Tabblad **Terugblik**, een scrollend verhaal over een jaar: gekeken uren, topseries, films, nummers en genres, en een kijkprofiel (nachtuil, vroege vogel, weekendkijker, bingewatcher, filmliefhebber, muziekliefhebber, gewoontedier) met grafieken per uur van de dag en per weekdag.
- Het jaar maand voor maand, met de titel die elke maand bepaalde.
- Records: drukste dag, grootste binge, langste dagelijkse reeks, langste enkele weergave, vaakst herbekeken titel, eerste weergave van het jaar en de oudste gekeken titel.
- Ontdekkingen: nieuwe series begonnen, films en afleveringen uitgekeken, en de series die precies één aflevering kregen.
- Een banner van één regel op het dashboard in december en januari als er een Terugblik klaarstaat.

## [0.2.0] - 2026-09-19

Meer dan Jellystat ooit vastlegde

### Added
- Een tijdlijn voor elke weergave: starten, pauzeren, hervatten, springen, wisselen van audio en ondertitels, direct afspelen dat overgaat in transcoderen, en stoppen. Plus het aantal pauzes en sprongen, waarvandaan het afspelen werd hervat, en of de kijker op het lokale netwerk zat.
- Tabblad **Server**: versie van Jellyfin, wachtende updates en herstarts, schijfgebruik per bibliotheek (Jellyfin 10.11+), plug-ins, resultaten van geplande taken en elk geregistreerd apparaat.
- Inzichten in **Afspelen**: gelijktijdige streams door de tijd, welke clients transcoderen afdwingen, hoe ver mensen komen voor ze stoppen, lokale tegenover externe weergaven en kijkgedrag.
- Waar je bibliotheek uit bestaat: resoluties, codecs, dynamisch bereik, containers, titels per decennium, toevoegingen per maand, de grootste titels, en de titels die nog nooit iemand heeft gekeken, gecontroleerd tegen zowel de geschiedenis van FinStats als de eigen gekeken-markeringen van Jellyfin.
- Dashboard: piek van gelijktijdige streams, geschatte gestreamde data, aandeel externe weergaven, genres naar kijktijd en mislukte aanmeldingen.
- Itempagina’s: links naar IMDb en TMDB, studio’s, bitdiepte en framerate, en wie de titel in Jellyfin als afgespeeld heeft gemarkeerd.
- Gebruikerspagina’s: genres, en de films, afleveringen en favorieten die Jellyfin heeft vastgelegd.

### Fixed
- Het tijdlabel op een kaart van Speelt nu loopt niet meer buiten de kaart.
- Lange labels op de pagina Afspelen werden te vroeg afgekapt.
- Een serie die de bibliotheek heeft verlaten, toont nog steeds de lijst met seizoenen en afleveringen.

## [0.1.0] - 2026-09-19

Eerste versie

### Added
- Live verzamelen uit de sessies van Jellyfin: gebruiker, titel, client, apparaat, IP-adres, afspeelmethode, redenen voor transcoderen en hardwareversnelling, gegevens over video, audio en ondertitels, en kijktijd tegenover gepauzeerde tijd.
- Aanmelden met je Jellyfin-account. Een installatie in twee stappen maakt de eigen API-sleutel van FinStats aan, en wachtwoorden worden nooit bewaard. Gebruikers die geen beheerder zijn, kunnen zich desgewenst aanmelden om alleen hun eigen statistieken te zien.
- Pagina’s Dashboard, Activiteit, Gebruikers, Bibliotheken, Afspelen en Serverlogboek, een opdrachtpalet (Ctrl/⌘ K) en een live statusbalk.
- Import uit Jellystat: grote back-ups worden naar schijf gestreamd en in seconden geïmporteerd, en hetzelfde bestand twee keer importeren is veilig.
- Eén klein programma met een ingebouwde webinterface en één SQLite-bestand, plus een Docker-image.
