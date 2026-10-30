# Ändringslogg

Allt som har ändrats i FinStats, nyast först. Den här filen byggs in i programmet och är det som
fliken **Ändringslogg** i appen visar.

Format: `## [version] - date`, eller `## [version] - first day to release day` för en version som togs fram
under flera dagar, därefter `### Added`, `### Changed`, `### Security`, `### Performance`, `### Stability`, `### Fixed`
eller `### Removed` (Tillagt, Ändrat, Säkerhet, Prestanda, Stabilitet, Åtgärdat, Borttaget) med en punkt per ändring.
Säkerhet är något som någon kunde ha missbrukat, Prestanda något som har blivit snabbare eller lättare, Stabilitet något
som inte längre kan krascha, stanna, läcka eller förlora data, och Åtgärdat något som gav fel svar. En `x.y.0`-version
har en kort titelrad under rubriken: den titeln använder appen som rubrik för hela serien.

## [2.3.1] - 2026-10-26 to 2026-10-28

### Security
- **Installationen ber om en engångskod**: tills FinStats är installerat skriver det en kod i sin logg när det startar (`docker logs finstats`), och guiden ber om den innan den testar en adress eller ställer in något. Annars kunde den som nådde porten först peka FinStats mot en egen Jellyfin och bli dess administratör, eller använda testet för att se sig omkring i nätverket.
- **Åtkomst till bibliotek följer Jellyfin**: var och en ser bara de bibliotek som Jellyfin öppnar för dem, och i dem bara de titlar som Jellyfin visar dem inom deras åldersgräns, taggar och regler för oklassade titlar, i bibliotekssidor, sökning, rollista, bilder, Nyligen tillagt, Min lista, Kommer snart, kalenderflödet och aviseringar om nya titlar. Alla inloggade kunde bläddra i alla bibliotek.
- En länk kunde skicka någon från välkomsten till en annan webbplats direkt efter inloggningen. Nu följs bara FinStats egna sidor.
- Serverloggen och de misslyckade inloggningarna i insikterna gav allas inloggningsadresser och aktivitet till alla med *Se servern*. Någon annans rader kräver *Se allas aktivitet* och en adress *Se nätverksdetaljer*, precis som överallt annars.
- Ett personligt mål för aviseringar kunde nå det lokala nätverket genom ett namn som svarade annorlunda andra gången det slogs upp. Anslutningen görs till adressen som kontrollerades, fler intervall som aldrig når internet räknas som lokala, hushållets egen publika adress avvisas också, och ett namn som inte går att slå upp avvisas med samma ord som ett lokalt.
- Bakom en proxy som lägger till en egen `X-Forwarded-For`-rad, eller från ett IPv6-nätverk, kunde varje inloggningsförsök komma från en ny adress och gränsen gällde aldrig. Ett IPv6-/48 får nu 50 försök på fem minuter, en full gräns avvisar en ny adress i stället för att glömma en gammal, hela servern stoppar vid 60 misslyckanden i minuten, och en misslyckad inloggning granskningsloggas en gång per adress och tidsfönster.
- `FINSTATS_TRUST_PROXY` anger de proxyer (adresser eller intervall) vars `X-Forwarded-For`, `X-Forwarded-Host` och `X-Forwarded-Proto` man litar på, och `1` eller `true` litar nu bara på en motpart på loopback eller ett privat nätverk. Påslaget kunde vilken motpart som helst välja sin egen adress; avslaget bakom en proxy låste tio felaktiga gissningar ute alla användare.
- När användarna har lästs besvaras en inloggning för ett namn som är okänt, inaktiverat, borttaget eller inte insläppt i FinStats som ett felaktigt lösenord utan att Jellyfin tillfrågas, och inte tidigare än Jellyfin skulle ha svarat. Sådana namn kunde låsa Jellyfin-konton, ta sig runt Jellyfins regel om fjärråtkomst eller gömma sig bakom FinStats adress.
- Den som Jellyfin håller till hemnätverket (*Allow remote connections* avslaget) loggar bara in i FinStats därifrån, eftersom Jellyfin bedömde inställningen utifrån FinStats egen adress, och en läsning av användarna som inte säger om någon är inaktiverad håller dem ute.
- En sessionscookie märks Secure så snart inloggningssidan ligger på https, även bakom en proxy som inte säger det.
- En lyckad inloggning förlåter inte längre gissningarna före den. På https är sessionscookien en `__Host-`-cookie som en webbplats på ett närliggande namn inte kan ersätta och som inte tas emot under något annat namn, och en utloggning avslutar varje session som webbläsaren skickade.
- Bara en kalendernyckel öppnar kalenderflödet från dess adress; alla andra nycklar skickas i headern.
- En API-nyckel kan inte längre ge behörigheter, ändra vem som släpps in eller vad som är publikt, återställa en säkerhetskopia eller lägga till, ändra eller testa en tjänst eller ett mål för aviseringar: var och en av dem överlever nyckeln och kräver därför en inloggad webbläsare.
- Varje svar har `Cross-Origin-Opener-Policy: same-origin` och en `Permissions-Policy`, API-svar har `Cross-Origin-Resource-Policy: same-origin` och är `no-store` om de inte säger annat, och CSP:n tillåter inte längre `blob:`-bilder. En sida som öppnade FinStats kunde senare göra dess flik till en egen inloggningssida.
- Där FinStats vet vilket schema det nås med (en betrodd proxys `X-Forwarded-Proto`, eller den publika adressen) avvisas en skrivning från ett annat schema som cross-site.
- `/auth/me`, en titels sida och Bibliotekshälsa länkar till Jellyfin på dess publika adress; adressen som FinStats själv läser Jellyfin på ges bara till administratörer.
- En bild hämtas från Jellyfin bara för något som FinStats har, även en publicerad profils affischer och kort, så ett annat biblioteks bilder kan inte begäras med id.
- En privat trackers nycklar rensas bort även ur nedladdningsköns fel, utifrån deras form såväl som deras namn, och den som bara har *Se vad som laddas ner* får veta vilken tjänst som misslyckades och hur, aldrig dess interna adress eller vart den omdirigerar.
- Sidan Server namnger en enhets senaste användare bara med *Se allas aktivitet*, och Olänkad media kräver det också, utöver *Hantera FinStats*. Utan den räknar avsnittet Uppgifter bara ens egna uppspelningar och strömmar, ett borttaget bibliotek listas utifrån ens egna uppspelningar, och en väntande Tautulli-tavla kan inte avbrytas.
- Inställningarna visar tjänsten för den publika adressen bara med sin värd och Jellyfins adress och datamappen bara för administratörer, och en titel som biblioteket har förlorat beskrivs bara utifrån uppspelningar som läsaren får se.
- En Tautulli-säkerhetskopia vars tabeller inte var vanliga tabeller kunde låta alla andra skrivningar vänta tills FinStats startades om. En sådan fil avvisas, och SQLite läser säkerhetskopian defensivt.
- En uppladdning över 4 GB avvisas, och en zippad Tautulli-säkerhetskopia packas inte upp längre än den storlek den uppger; båda kunde fylla disken som FinStats egen databas ligger på.
- Ett svar från Jellyfin, Sonarr, Radarr, Seerr, uppslaget av den publika adressen eller GeoIP-nedladdningen läses inte längre än det högst får vara, räknat efter uppackning.
- Vilken Jellyfin-användare som helst kunde få FinStats att sluta registrera uppspelningar för alla genom att rapportera tusentals korta uppspelningar av en titel. Att räkna ut vilka som tittade tillsammans görs nu i ett enda svep och håller inget lås medan det pågår.
- Uppspelningar som sparats direkt efter varandra, eller en titel som spelas i tur och ordning från enhet efter enhet, slår inte längre ihop främlingar till en grupp, och den som bara får se sig själv får bara veta om de personer som ens egen uppspelning gick sida vid sida med, och bara under den tid den gjorde det.
- En länk kunde fortfarande skicka någon från välkomsten till en annan webbplats efter inloggningen, via en sökväg som `/.//`.
- Den som har *Hantera FinStats* kan inte längre ändra hur många säkerhetskopior som sparas eller hur ofta de skrivs, sammanslagningsfönstret (nu högst en timme) eller när användare, aktivitetsloggen och säkerhetskopior läses, och säkerhetskopior som en sådan person tar trycker aldrig ut en administratörs; hemadresser och resereglerna kräver det som larmen kräver, och att importera historik kräver även *Se allas aktivitet*.
- En sparad nyckel, token eller lösenord går bara till den server den sparades för, och ett mål för aviseringar skapas eller ändras bara när man är inloggad, aldrig med en API-nyckel. En administratör får stänga av eller ta bort någon annans personliga mål, men inte flytta det, stänga av dess certifikatkontroll eller läsa dess ntfy-ämne eller Pushover-användarnyckel.
- Webhooks för Discord och Slack måste vara `https`, eftersom deras adress är deras token.
- En flod av misslyckade inloggningar kan inte längre dölja en riktig eller begrava larmen: namn som inte är något konto ger ett meddelande per kvart som inte namnger någon, larm skickas först, och Slack och Discord visar vad ett meddelande säger som text och pingar ingen.
- Placeringen i Återblick, Kapacitet, en avisering och en publicerad profil berättar inte mer än sidorna gör: ingen placering eller andel av allas tid till den som bara får se sig själv, belastningstoppar bara som dagar med tre strömmar eller fler utan *Se allas aktivitet*, inget misslyckat jobb eller anslutning till den som inte får hantera FinStats, och en offentlig profil ändras en gång om dagen.
- En Tautulli-säkerhetskopia med en genererad kolumn avvisas, och en bild hämtas för en importerad uppspelning först när biblioteket har titeln.
- En återställning öppnar aldrig FinStats mer än det är: inställningar kommer tillbaka bara när man ber om det, inloggning, behörigheter och publika profiler blir det som både den här installationen och säkerhetskopian tillåter, och de publika adresserna förblir den här installationens. `finstats restore <file>` tar bara tillbaka historiken, precis som sidan, och även inställningarna med `--with-settings`.
- En återställning av enbart historiken tar inte längre tillbaka hemadresser, tystade larm, granskningsposter som utger sig för att vara den här installationens egna eller lokaliserade titlar som biblioteket saknar, och ingen återställning tar tillbaka uppspelningar, loggrader eller enheter daterade före 1990 eller efter i morgon. En återställd uppspelning räknas som importerad oavsett vilken källa den uppger, så en omstart fortsätter den aldrig.
- Datamappen tillhör bara FinStats egen användare (0700, dess filer 0600, `umask 077` i avbildningen). FinStats vägrar köra som root över en datamapp som en annan användare äger och anger vilket `docker exec -u` som ska användas, och entrypointen ändrar ägaren bara för mappen och FinStats egna poster.
- Ett måls namn får inte innehålla kontrolltecken, och namnet på en enhet, en person eller en titel kan inte längre förfalska en rad i loggen.
- En länks ruttparameter kan inte längre nå en annan API-sökväg än sidans egen, och FinUIs elementbyggare sätter en adress bara när den är http(s) eller relativ, aldrig `srcdoc`.
- Sökning, publicerade profiler och arbete som startas för hand kan inte längre hålla FinStats upptaget: en sökning slutar vid 200 tecken, en profil sparas per språk, kort väntar i en kort kö, och ett jobb, adressuppslaget, nedladdningen av platsdatabasen och ett testmeddelande startar bara så ofta, hur de än efterfrågas, med testmeddelanden räknade per avsändare. Att hämta platsdatabasen kräver också det som läsning av sidan Säkerhet kräver.
- Innehållet i en begäran måste fortsätta komma när de första 15 sekunderna har gått, ett svar som ingen läser ges upp efter 30 sekunder, och en adress håller högst 64 anslutningar, en betrodd proxy undantagen. Några hundra begäranden som aldrig skickade sitt innehåll kunde hindra FinStats från att svara någon.
- En e-postserver läses inte längre än något svar kan vara (1 MB för ett helt brev), och ett testmeddelande stoppas efter lika lång tid som ett meddelande i kön får ta. En server som aldrig avslutade en rad kunde få FinStats att hålla mer och mer minne tills den stoppades.
- En läsning av ett bibliotek bedöms mot de titlar läsningarna har sparat, inte dem en import tog med, och en Jellystat-titel i ett bibliotek som redan är läst kommer in som borta. En import av påhittade titlar kunde få varje läsning av ett bibliotek att stoppa FinStats vid varje start.
- Gränsen på 60 misslyckade inloggningar i minuten för hela servern räknar bara försök som skickas till Jellyfin, även de som fortfarande väntar på svar. Namn som FinStats själv avvisade lät vem som helst med några få adresser hindra alla från att logga in, och många gissningar som skickades samtidigt nådde alla Jellyfin.
- Den som Jellyfin håller till hemnätverket hålls där vid varje begäran, sessionen och nycklarna också, inte bara vid inloggning; hemma är ett privat nät eller hushållets uppslagna offentliga adress, aldrig en adress inskriven som hemma, och bakom en proxy som `FINSTATS_TRUST_PROXY` inte nämner hålls de ute. En kaka eller nyckel som tagits ut ur huset öppnade allt varifrån som helst, och en förvaltare kunde öppna huset för en adress de själva valde.
- En API-nyckel kan inte längre stänga av det som skulle märka att den missbrukas, eller säkerhetskopiorna som skulle göra om det den gjorde: hur många säkerhetskopior som sparas och hur ofta, sammanslagningsfönstret, hemadresser och reseregler, scheman för användare, aktivitetsloggen och säkerhetskopior, att tysta eller lösa alla larm, en säkerhetskopia i papperskorgen eller att ta bort ett aviseringsmål. Var och en varade längre än nyckeln när den återkallades.
- Affischerna från Sonarr och Radarr och önskelistans nästa avsnitt följer också biblioteksåtkomsten: en serie som Jellyfin döljer för någon har varken affisch eller datum för dem. Att räkna tjänsternas id:n avslöjade vilka dolda serier som var på väg, och en lista nämnde nästa avsnitt av en dold serie.
- Tillsammans-kapitlet i årsöversikten nämner bara dem som tittade samtidigt som du, som Tillsammans-sidan gör. En grupp du kom med i genom någon annan nämnde folk du aldrig tittade med.
- En import från Jellystat eller Streamystats läser hela filen innan den tar databasens skrivlås och skriver sedan bara de rader den behåller. Läst under låset kunde en fil med några få riktiga rader och gigabyte av vad som helst annat hålla all annan skrivning väntande, uppspelningar och inloggningar också.
- En återställning håller en behörighet per person, för inte fler personer än ett hushåll många gånger om, och kontrollerar en granskningspost för dubbletter utan att läsa varje post den just har skrivit. En konstruerad säkerhetskopia kunde hålla minne och databasens skrivlås längre och längre under återställningen.
- En återställning med inställningarna slår på uppslaget av den offentliga adressen och hämtningen av platsdatabasen bara där den här installationen också har dem på, och sparar aldrig färre säkerhetskopior eller skriver dem mer sällan än den här installationen gör. En säkerhetskopia som sparade en kunde få nästa körning att ta bort alla andra.
- En filstorlek som ingen disk kunde rymma läses som ingen storlek, och storlekar läggs ihop så att ingen summa kan svämma över. En importerad storlek fick bibliotekssidorna att svara med fel för gott.
- När FinStats vet att den nås via https (den offentliga adressen är https, eller en betrodd proxy säger det) tas en session bara från dess `__Host-`-kaka, och med en offentlig https-adress är varje ny session en https-session. En inloggning som skickades utan `Origin` genom en proxy som inte sa något gav en vanlig session som en grannwebbplats kunde plantera i någons webbläsare.

### Performance
- En publicerad profil räknas fram högst en gång i minuten, inte på nytt för varje bild och kort på den.
- En persons historik genomsöks efter säkerhetslarm en gång i taget, hur många uppspelningar som än börjar.
- Att logga in eller ut, återkalla en nyckel eller ändra en persons behörigheter ber bara de direktanslutningar det gäller att titta igen.
- Tautulli-kopplingstavlan läser historiken en gång, inte en gång per Plex-användare.
- En bild som saknas i cachen hämtas av en enda begäran medan de andra väntar, och en som källan inte kunde ge efterfrågas inte igen på 30 sekunder.
- Webbgränssnittets filer gzippas en gång och sparas, och typsnitt komprimeras inte igen.

### Stability
- En säkerhetskopia i Jellystats äldre format läses en rad i taget i stället för hel, och en rad som är längre än någon säkerhetskopias avvisas.
- Två Tautulli-uppladdningar samtidigt skriver inte längre i samma fil.
- Ett sidnummer långt bortom slutet på ett biblioteks titlar eller på Bibliotekshälsa är en tom sida, inte ett fel.
- En import eller återställning vars uppladdning avbröts avvisar inte längre alla andra tills en omstart, och varken den eller en import som övergavs innan den började lämnar en fil efter sig.
- En rad i en säkerhetskopia läses inte längre än 8 MB, och en Tautulli-säkerhetskopia tas emot i taget.
- En uppspelning behåller högst 2 000 händelser, och en import hoppar över en uppspelning som inte både börjar och slutar mellan 1990 och i morgon.
- Ett enda avsnittsnummer långt under noll får inte längre varje läsning av biblioteket att hänga sig: Bibliotekshälsa (även dess sammanfattning), Kapacitet, omlänkning och lokalisering adderar och jämför tal utan att slå runt, och en release-build kontrollerar spill, så att allt som missas stoppar en uppgift i stället för att gå i en slinga.
- Ett säsongs-, avsnitts- eller slutnummer behålls bara där Jellyfin kunde rymma det (0 till 2 147 483 647), från Jellyfin, Jellystat och Tautulli lika.
- En Tautulli-säkerhetskopia vars id:n upprepas avvisas, eftersom några MB rader under ett id fick processen att ta slut på minne, och en säkerhetskopia öppnas oföränderlig och lämnar ingen `-wal` eller `-shm` efter sig.
- En anslutning måste skicka sin begäran inom 15 sekunder och högst 512 betjänas samtidigt: några hundra halvöppna socketar tog FinStats offline. En ram på direktkanalen begränsas till 4 KB.
- Några kilobyte från Sonarr, Radarr eller Seerr kan inte längre bli gigabyte av minne: en lista behåller inte mer än FinStats läser av den.
- Ett mål för aviseringar som aldrig svarar håller bara upp sig självt: mål skickas till sida vid sida, och ett som hela tiden får timeout lämnas ifred i tio minuter.
- En kös återstående tid och totala hastighet, en återställd uppspelnings siffror och dagen då en uppspelning lämnar papperskorgen kan inte längre spilla över.
- En återställning läser hela filen innan den tar skrivlåset, avvisar en som packas upp till långt mer än sin storlek och skriver i omgångar, så insamlaren fortsätter registrera under tiden.
- Säkerhetskopior namnges i UTC och skrivs aldrig över en befintlig fil, och bara FinStats egna kopior från före en uppdatering rensas.
- En rad i serverloggen daterad i framtiden stoppar inte längre läsningen av aktivitetsloggen: den läses från dess nyaste rad som inte ligger i framtiden.
- Begäranden som anländer samtidigt till en publicerad länk räknar fram dess svar en gång, och dess kort ritas i ett eget spår, så den som har länken kan inte längre få ägarens Återblick-kort att svara 503.
- Uppspelningarna i ett bibliotek som saknas i Jellyfins lista över bibliotek väntar två dagar innan de flyttas till ett annat biblioteks kopia av samma titel. En lista som utelämnade ett bibliotek flyttade dem genast, och de förblev flyttade när biblioteket kom tillbaka.

### Fixed
- Misslyckade inloggningar på ett konto är en varning per tidsfönster som växer till ett larm vid tjugo, och ett namn räknas mot ett konto bara när det är skrivet helt ("x alice" räknas inte längre mot alice).
- Meddelanden till Discord och Slack escapas först och kortas sedan: ett enhetsnamn fullt av Markdown fick Discord att avvisa varje larm som nämnde det.
- En publik IPv6-adress godtas för ett personligt mål.
- Bakom en proxy som själv loggar in folk (nginx `auth_basic`) avvisas inte längre FinStats cookie.
- Ett visningsnamn som innehåller `{{url}}` visas som det skrevs i förhandsvisningen av en publicerad sida.
- En sammanfattning för år 9999 visar standardåret i stället för ett fel.
- En livesida vars behörighet togs bort och gavs tillbaka medan den var öppen hålls aktuell igen, utan omladdning.

### Removed
- **Synnmørsk** är inte längre ett av FinStats språk: en webbläsare som valde det läser nynorska, skriftspråket det byggde på.

## [2.3.0] - 2026-10-07 to 2026-10-26

Kapacitet, en direktkanal, Enkelt läge och språk

### Added
- **Språk**: FinStats talar 23 språk. Brittisk och amerikansk engelska samt norskt bokmål och nynorska skrivs och granskas av underhållaren; danska, tyska, spanska, finska, franska, hindi, isländska, italienska, japanska, koreanska, nederländska, polska, portugisiska (Brasilien), ryska, svenska, turkiska, ukrainska och kinesiska (förenklad och traditionell) är översättningar som ingen här har granskat, och säger på engelska det de ännu inte har. Alla sidor i båda layouterna, den offentliga profilen, Ändringslogg och det servern svarar med ord (ett felmeddelande, korten i Återblick, vad ett jobb i Jellyfin gör) visas på det språk som har valts under Inställningar → Utseende → Språk, per webbläsare; tills ett språk har valts avgör webbläsarens eget språk. Tal, datum, enheter och listor skrivs så som det språket skriver dem. Ett skript kan be om ett språk med `Accept-Language`.
- **Enkelt läge**: en andra layout för dig som kommer från Plex, uppbyggd som en klassisk statistikserver för Plex. Ett toppfält i stället för menyn, och bara statistiksidorna: Hem, Bibliotek, Användare, Historik, Diagram och Inställningar, med ett kugghjul för din profil, Inställningar, Ändringslogg, temat och utloggning. Väljs per webbläsare under Inställningar → Utseende → Layout. Det laddar bara sin egen halva av FinStats och startar därför snabbare; Återblick, Tillsammans, På väg, Säkerhet och serversidorna finns kvar i FinStats-layouten. Det använder FinStats egna färger, ljust och mörkt, och ingenting i det rör sig.
- **Hem i Enkelt läge**: det som spelas, med ett kort per ström (hur den spelas upp, hemifrån eller utifrån, när den tar slut), tittarstatistiken som rankade kort efter antal uppspelningar eller speltid de senaste N dagarna, vad varje bibliotek innehåller och vad som har kommit på senare tid.
- **Diagram i Enkelt läge**: uppspelningar efter mediatyp (dag för dag, efter veckodag och timme, de tio vanligaste apparna och personerna), efter hur de spelades upp (dag för dag, flest strömmar samtidigt varje dag, filens och den skickade upplösningen, appar och personer) och månad för månad, efter antal uppspelningar eller speltid, för alla eller för de valda personerna.
- **Bibliotek, Användare och titlar i Enkelt läge**: tabeller över alla bibliotek och personer med vad som spelades senast, på vad och varifrån (adressen bara med *Se nätverksdetaljer*), och en sida för varje bibliotek, person och titel med dess siffror och dess egen historik.
- För skript: `/api/stats/split` (uppspelningar efter app, person, upplösning, dag eller månad, uppdelade efter mediatyp eller efter hur de spelades upp); topplistor efter antal olika tittare (`sort=users`); aktivitet begränsad till uppspelningar som räknas som sedda (`watched=1`); insikter bara för de delar som efterfrågas (`parts=`); toppbelastningen uppdelad i omkodningar, direktströmningar och direktuppspelningar, även per dag; vad varje bibliotek innehåller och vad som spelades där senast; nyligen tillagt efter typ (`kind=`); värmekartan efter mediatyp; en persons senaste uppspelning; om en ström spelas upp hemifrån.
- **En direktkanal**: Översikt, nedladdningarna i På väg och statusfältet håller sig uppdaterade över en WebSocket per flik (`/api/live`), så det som spelas och laddas ner ändras på skärmen i samma stund som det sker i stället för vid nästa koll. Bakom en omvänd proxy som inte släpper igenom WebSocket-uppgraderingar, eller på en anslutning som föll bort utan ett ord, frågar sidorna med några sekunders mellanrum som förut. `/api/now-playing`, `/api/downloads` och `/api/summary` är kvar som de är, för skript.
- **Server → Kapacitet**: klarar den här servern sig, utifrån det FinStats har sett? De mest belastade ögonblicken som finns registrerade och de senaste 90 dagarnas: strömmar samtidigt, och hur många som fick bilden omkodad med hårdvara, utan, eller utan att något angavs (en historik som importerats från Tautulli kommer från Plex och räknas inte med). Varje disk som biblioteken ligger på och hur snabbt det Jellyfin har lagt till under sex månader fyller den (en disk som har fyllts på senare mäts från en månad efter dess första fil, eftersom den första månaden är själva påfyllningen), och, där Jellyfin 10.11 eller senare anger ledigt utrymme, hur länge det räcker i den takten. FinStats säger aldrig hur många strömmar servern skulle klara.
- **Exportera .ics** under På väg → Kommande: listan som du har filtrerat den, nedladdad som en kalenderfil att importera en gång. Den uppdaterar sig inte själv; en kalendernyckel från Inställningar → API-nycklar ger fortfarande en kalender som gör det.
- **Värt en titt**: Kapacitet säger till när varje omkodning som Jellyfin har rapporterat om under 90 dagar har körts utan hårdvaruacceleration trots att omkodning är vanligt, när Jellyfins biblioteksgenomsökning pågår en fjärdedel av tiden eller mer, och när en disk har mindre än sex månaders utrymme kvar. Varje punkt kommer med sina siffror och kan läggas åt sidan, precis som fynden i Bibliotekshälsa.
- **Vad ditt bibliotek består av** kan avgränsas till ett bibliotek eller flera, under Bibliotek och på varje biblioteks egen sida (`library_id=a,b`).
- **Datum- och tidsformat** under Inställningar → Utseende: ett datum som 31.01.2026, 31/01/2026, 31-01-2026, 01/31/2026, 2026-01-31, 2026.01.31 eller 2026/01/31, och en tid med 24 eller 12 timmar, med eller utan inledande nolla, eller båda så som språket skriver dem. Sparas i den här webbläsaren.
- **Ett välkomnande**: första gången någon loggar in, oavsett webbläsare, och i slutet av installationen, frågar FinStats efter språk, datum- och tidsformat och layout, med en bild av varje. Hoppa över behåller brittisk engelska, 31/01/2026, 13:05 och FinStats.

### Changed
- Logotypen inne på en sida är det rena märket bredvid namnet, satt i Inter oavsett vilket typsnitt en förinställning väljer; där den står ensam (favikonen) behåller den sin rundade ruta.
- En uppspelning som har omkodats i mjukvara registreras som det där Jellyfin anger det; tidigare såg den likadan ut som en från en server som inte anger det. Sidorna kallar fortfarande båda för Mjukvara.
- En uppdatering kör nu, en gång och äldsta versionen först, det som en version behöver göra med en databas som en äldre FinStats har skrivit, vid sidan av migreringarna. I 2.3.0 läses serverns uppgifter vid första starten, så Kapacitet finns där direkt.

### Performance
- **En sida som står öppen frågar inte efter något**: Översikt gjorde 30 förfrågningar i minuten för att hålla sig uppdaterad och nedladdningarna i På väg 18; med direktkanalen gör båda inga alls. Hur många flikar som än är öppna frågar FinStats inte Jellyfin, Sonarr och Radarr om något mer än förut, och en dold flik låter fortfarande nedladdningskön läsas mer sällan.
- En persons sida läser bara den personens uppspelningar: totalerna och den senaste uppspelningen räknades ut över alla uppspelningar på servern och begränsades sedan till personen, ungefär fyra sekunder på en miljon uppspelningar.
- Ett biblioteks sida läser bara det bibliotekets titlar och uppspelningar, där den förut räknade alla bibliotekens och behöll ett: en tredjedel av tiden på en miljon uppspelningar.
- När Jellyfins klocka gick före FinStats läste *Efter Jellyfins biblioteksgenomsökning* hela biblioteket på nytt varje minut tills skillnaden hade passerat; nu följs varje genomsökning en gång.
- En GeoIP-fil som inte gick att öppna kopierades på nytt, hundra megabyte och mer, och varje adress placerades på nytt, varje kvart; nu provas den igen först när den ändras.
- Bibliotekshälsa öppnas direkt efter att pekaren har vilat på länken: det som hade hämtats i förväg lades där sidan aldrig letade.

### Stability
- En uppdatering som misslyckades efter att en migrering hade körts, och som Docker genast startade om, tog en ny kopia av den halvt uppdaterade databasen vid varje start, och tre starter rensade bort den enda kopia som den föregående versionen kunde öppna. Kopian från före uppdateringen behålls nu.
- En titel som togs bort i Jellyfin medan biblioteket lästes (en fil som Radarr bytte ut) kunde få läsningen att missa en annan titel som fortfarande fanns kvar och markera den som borttagen, och dess uppspelningar kunde sedan flyttas till en annan kopia av den. Det som en läsning inte såg efterfrågas nu först via id.
- Att återställa en säkerhetskopia som togs innan en fil bytte namn eller uppgraderades återställde en gång till varje uppspelning som sedan hade flyttats till den nya filen, så att den fanns dubbelt för gott.
- Ett önskemål i Seerrs svar som FinStats inte kunde läsa (ett fält som bytte namn i en uppdatering av Seerr) markerades som borttaget av den dagliga fullständiga läsningen.
- Ett jobb som stötte på en bugg stod kvar som ”körs” för gott, och ingen utlösare startade det igen förrän FinStats startades om; nu avslutas det som misslyckat.
- En skadad Tautulli-zip stoppade uppladdningen innan filen togs bort, och den låg kvar, med alla Plex-användares åtkomsttoken, tills rensningen efter sex timmar.
- En import eller återställning som avbröts medan filen kom in lämnade filen kvar i datamappen för gott, adresser och allt; nu tas den bort vid nästa start.
- En bild kunde serveras halvskriven medan den lades i cachen, och webbläsaren behöll den sedan så.
- Återkallade API-nycklar sparades för gott, hur många som än skapades och återkallades.
- En läsning av någons sedd- och favoritmarkeringar som kom tillbaka tom eller oläslig rensade alla till nästa läsning; den avvisas nu, som en tom läsning av ett bibliotek.
- Ett avsnitt med ett slutnummer som inget avsnitt kan ha (ett datum skrivet där) kunde hålla Bibliotekshälsa sysselsatt utan slut efter en läsning av biblioteket.

### Fixed
- En punkt i ändringsloggen med kod inuti fetstil (i 2.0.2 och 2.0.3) visade sina backticks i stället för att visa koden.
- Ett diagram över antal kunde numrera sin axel 0, 3, 5, 8, 10 eller till och med 0, 0, 1, 1, 1: steg på 2,5 skrivna avrundade. Antal stiger nu i hela steg.
- En kalendernyckel som skapades av någon som inte får hantera FinStats fick en länk till webbläsarens egen adress i stället för FinStats offentliga, så den fungerade bara inifrån; sidan bad också om inställningar som den nekades.
- En persons senaste uppspelning kunde vara några sekunder för kort för att räknas som en uppspelning någon annanstans, och nämnde då en titel som personens Historik inte visar.
- En återställning från Nyligen borttagna som misslyckades sa ingenting och lät knappen fortsätta snurra; detsamma gjorde ett misslyckat *Ta tillbaka* i Bibliotekshälsa och ett misslyckat *Öppna igen* eller *Lös alla* i Säkerhet.
- Kalenderflödet heter FinStats i en kalenderapp, inte finstats.
- Nedladdningskön visade vem som hade bett om varje nedladdning för alla som fick se den; någon annans önskemål kräver nu *Se allas aktivitet*, som i På väg.
- Att ta bort *Få aviseringar* stoppade inte det som skickades till personens mottagare, och hen kunde inte längre stänga av dem.
- En uppspelning som pausades och återupptogs inom sammanslagningsfönstret aviserades som stoppad en gång, medan återupptagningen och det verkliga slutet aldrig aviserades.
- Ett spårs uppspelningar kunde flyttas till en annan artists spår med samma namn (”Intro”) när dess album försvann ur biblioteket.
- En aviseringsmottagare skriven som en IPv4-adress inuti IPv6 (NAT64, 6to4) godtogs som offentlig trots att den nådde det lokala nätverket.
- En anslutning till Sonarr eller Radarr som riktades mot en annan server fortsatte att visa den gamla serverns affischer i en vecka.
- Larmen i Säkerhet sa att inget behövde ses över när två eller fler personer var valda, och en sida som blev tom när dess sista larm löstes hade ingen väg tillbaka.
- Varje API-nyckel som inte hade gått ut sa att den gick ut ”nyss”.
- En dialogruta låg kvar över sidan efter Tillbaka, och sidan gick inte att rulla förrän den stängdes.
- Granskningsloggen visade åtta typer av poster (en uppspelning eller säkerhetskopia som tagits tillbaka, papperskorgen tömd, offentliga profiler, ett ändrat utseende) med sina id:n och ingenting under Detalj.
- Att lägga en titel i Min lista från dess snabbmeny misslyckades utan ett ord när servern sa nej.
- *Sett tillsammans* på Översikt behandlade flera personer som valts samtidigt som en enda person.
- Återblicken skrev timmar och dagar under tio med webbläsarens decimalkomma, samtidigt som den själv använde komma som tusentalsavgränsare.
- Kapacitet sa att en disk skulle bli full "i takten från de senaste 0 månaderna".
- På sidan Säkerhet räknade adresserna utan plats allas uppspelningar genom alla tider, oavsett vilken person och period sidan var avgränsad till.
- Någon med *Hantera FinStats* men utan *Se allas aktivitet* kunde lägga vems uppspelning som helst i papperskorgen.
- Ett kort från Återblick eller en offentlig profil kunde komma från webbläsarens cache på det språk som valts förut.
- Nedladdningarna som livekanalen skickade var på brittisk engelska, oavsett vilket språk sidan var på.
- En inloggning som ändrade vad någon får se nådde deras andra öppna flikar först vid nästa kontroll efter en halv minut.
- En flik som lämnade ifrån sig sin liveanslutning till en nyare tog tillbaka en vid nästa sida, så att en persons flikar hela tiden stängde varandras.
- Tabeller sorterade storlekar från 1 000 till 1 023 enheter som text på språk som grupperar siffror med mellanslag, och turkiska procent likaså.
- En titel med en platshållare i namnet, som "Big {device} Bunny", fylldes i av serverns meddelanden.
- Tabeller sorterade längder och antal av en helhet (5 av 26) som text på alla språk utom engelska och de nordiska.
- En flik som fortfarande var öppen på ett språk fick serverns svar på ett annat så snart en annan flik hade valt det.
- Ett ord bredvid ett tal fick fel form på polska, ryska och ukrainska ("5 wieczory", "2 потоков").
- Kapacitet räknade ett biblioteks första fyllning som tillväxt och delade det varje bibliotek lade till med sex månader, oavsett hur länge det mättes; en fil som tog en annans plats (en uppgradering från Radarr eller Sonarr, eller en fil som Jellyfin gav ett nytt id) räknades som helt ny, när bara skillnaden var ny.
- De som tittade tillsammans med SyncPlay blev ingen grupp när en anslöt några minuter efter den andra; uppspelningar som slutar samtidigt på samma ställe hör ihop nu, och historiken grupperas om en gång.
- Någon utan *Se allas aktivitet* kunde se av ett biblioteks *Aldrig sett* att någon annan hade spelat en titel; nu räknas bara den egna tittningen.

## [2.2.0] - 2026-10-01 to 2026-10-07

Bibliotekshälsa och FinUI

### Added
- **En egen meny på datorn**: Inställningar → Utseende erbjuder åtta: sidopanelen som den var, en grupperad efter vad sidorna är till för, en list med ikoner som öppnas till sidopanelen under pekaren, en sidopanel som börjar med sökning, en docka, ett kommandofält, sidor som du fäster överst, och färgade rutor. Sidopanelerna kan stå till höger. Dockan glider undan medan du rullar nedåt och kommer tillbaka när du rullar uppåt eller pekar mot fönstrets nederkant; med den flyttar statusfältet upp till toppen.
- **En egen meny på telefonen**: ett flikfält med Mer (den nya standarden), ett ark som öppnas halvvägs, hela skärmen, sidorna utfällda som en solfjäder runt tummen, eller ett adressfält som växer till menyn. Var och en söker bland sina sidor och skickar alla andra ord vidare till sökningen.
- Båda sparas av den webbläsare där de valdes, så en telefon och en dator behåller var sitt val; båda är FinUI-komponenter, `desktop-nav` och `mobile-nav`.
- **En snabbmeny**: högerklicka på en titel, en person eller ett bibliotek var som helst i FinStats (en affisch på Översikt, en rad i en lista, ett namn i en tabell, en film i Kommer snart som ännu inte finns i biblioteket), håll fingret på den eller tryck på menytangenten, så växer en meny fram ur punkten där du tryckte: öppna, öppna i ny flik, lägg en film eller serie i Min lista, öppna i Jellyfin eller på TMDB och TVDB, kopiera länken. Skift med högerklick ger fortfarande webbläsarens egen meny. FinUIs `context-menu`.
- **Sökning utan dialogruta**: sökningen växer fram ur din meny (en sidopanel vidgas till den, dockan reser sig till den, kommandofältets namn förvandlas till den, telefonens meny rymmer den) eller, om du väljer det i Inställningar → Utseende, tar sidans plats medan menyn står kvar. Ctrl+Space öppnar den som förut; fönstret ovanpå allt är borta. FinUIs `search`.
- **Allt i ett bibliotek**: kortet Nyligen tillagt i ett bibliotek erbjuder *Allt i det*: alla filmer, serier och album det innehåller, som affischer, efter namn, år, datum då de lades till eller storlek, och filtreras medan du skriver. Fler kommer till medan du rullar.
- **Server → Bibliotekshälsa**: FinStats jämför varje fil med sina grannar och listar det som sticker ut, vart och ett med siffrorna bakom. Den rapporterar bara; ingenting ändras, tas bort eller genomsöks på nytt i Jellyfin.
- **Luckor**: avsnitt som saknas mellan två filer i en säsong (”Säsong 2: avsnitt 4 och 5 saknas mellan 3 och 6”). En fil som innehåller två avsnitt räknas som båda; specialavsnitt och slutet av en säsong lämnas i fred.
- **Säsonger som skiljer sig** från resten av serien i upplösning, HDR eller kodek, och **udda avsnitt**: en fil som inte liknar resten av sin säsong, till exempel ett SD-avsnitt i en 1080p-säsong. Upplösningen går efter bredd eller höjd, så en beskuren film i 1920×800 är 1080p.
- **Kopior och versioner**: samma film eller avsnitt två gånger, enligt de leverantörs-id:n som Min lista och På väg redan går efter. Två kopior i samma upplösning visar utrymmet som tas upp två gånger; en 4K-kopia bredvid en 1080p-kopia listas som två versioner och räknas aldrig som slöseri.
- **Tunna filer**: en bithastighet långt under vad upplösningen och kodeken brukar kräva (”4K HEVC på 1,1 Mbps”); en säsongs tunna avsnitt är ett enda fynd.
- **Dubbningar som tar slut**: ett ljudspråk som vissa säsonger har på varje avsnitt och andra på inget.
- **Aldrig identifierade**: filmer och serier som Jellyfin har matchat utan id från TMDB, TVDB eller IMDb.
- **Avfärda** ett fynd som är så med avsikt, med en anteckning. Det kommer tillbaka av sig självt när dess filer ändras. Den som får hantera FinStats och se servern kan avfärda; avfärdanden står i granskningsloggen och ingår i en säkerhetskopia.
- Ett biblioteks sida säger hur många saker det finns att se över där, och leder till dem.
- **FinUI**, komponenterna som FinStats är byggt av, är ett eget projekt (`github.com/finstats/finui`): varje knapp, kort och kontroll i alla tillstånd och båda teman på `finui.finstats.no`, och **FinUI create** för att ändra färger, hörnradie, täthet och mer medan du tittar, och ta med det hem med ett enda kommando. **Inställningar → System** leder dit.
- **Inställningar → Utseende**: var och en väljer hur FinStats ser ut för just dem, med koden till en FinUI-förinställning: grundfärg, accentfärg, diagramfärger, kontrast, hörnradie, täthet, kanter, kort, knappar, fält, tabeller, markering, rörelse, ikoner, menyn, bakgrunden bakom alltihop, fokusringen, hur rubriker sätts, och typsnitten för text, rubriker och siffror. Ditt val gäller bara dig; sidan visar vad koden betyder och leder till FinUI create för att ändra den.
- **Stilar**: elva kompletta utseenden att utgå från med ett enda val i FinUI create, vars kod Inställningar → Utseende tar emot. De är Washi (FinStats som det levereras), Studio, Noir, Matinee, Console, Pastille, Ledger, Gazette, Arcade, Canopy och Harbour. FinStats självt levereras med ett utseende och erbjuder ingen lista över stilar.
- **Ikoner som rör sig**: varje ikon i en länk, en knapp eller en flik gör sin egen rörelse en gång när du pekar på den (uppdateringsikonen snurrar, nedladdningens pil faller ner i sitt fack, papperskorgen lyfter på locket), och en uppdateringsknapp som du har tryckt på fortsätter att snurra tills svaret har kommit. Stilla med reducerad rörelse, och med Rörelse: Av.
- **Rörelse, med FinMotion**: FinUIs komponenter rör sig nu: knoppen på en brytare kastas över, strecket under en flik kryper vidare till nästa flik, ett fält som nekas skakar på huvudet, en kopiering bockas av i ett enda drag, meddelanden läggs på varandra som kort på handen och fälls ut som en solfjäder när du pekar på dem, ett ark på telefonen ger efter som ett gummiband när du drar i det, raderna i en sorterad tabell flyttar till sina platser, en siffra som ändras rullar som en vägmätare och diagram ritar sig själva första gången de syns. Allt följer valet för Rörelse i Inställningar → Utseende och står stilla med reducerad rörelse. FinMotion är ett eget projekt (`github.com/finstats/finmotion`), lagt ovanpå FinUI.
- **Fler ikoner i FinUI**: 239 nu, upp från 71, var och en med en egen rörelse (volym upp och ned, ett headset och ett avstängt headset, en penna som skriver, en klocka som ringer, en telefon som vibrerar), och var och en kan laddas ner från FinUIs galleri som SVG, stilla eller animerad.
- **Typsnitt**: trettiofyra fritt licensierade familjer att välja bland (Geist, Roboto, Manrope, IBM Plex Sans, Montserrat, Lora, Playfair Display, Fira Code med flera), medföljande precis som Inter, aldrig hämtade någon annanstans ifrån, och laddade först när de väljs.
- **Fler grundfärger och accenter**: Nord, Solarized, Gruvbox, Catppuccin, Rosé Pine, Tokyo Night, Dracula, Everforest och Ocean; sexton nya accenter, tio av dem med en färg på dagen och en annan på natten. Varje grundfärg har nu ett eget papper på dagen: de grå såg likadana ut i det ljusa temat.
- **Borttagning kan ångras i 30 dagar**: en borttagen uppspelning eller säkerhetskopia väntar i papperskorgen, och **Ångra** i meddelandet som följer tar tillbaka den. Aktivitet och Inställningar → Säkerhetskopior listar var för sig det som nyligen har tagits bort, med när det försvinner för gott, så länge det finns något. En uppspelning i papperskorgen räknas inte med i några totaler, diagram, återblickar eller profiler, och en import eller återställning tar inte tillbaka den som ny; den som såg den tillsammans med någon räknas som ensam tills den kommer tillbaka.

### Changed
- Dark Reader och liknande tillägg låter FinStats sidor vara i fred (`darkreader-lock`): FinStats ritar båda temana själv, och en sida som målades om kämpade emot dem.
- På telefonen glider menyn inte längre in från vänster: flikfältet sitter längst ner om ingen annan meny har valts.
- FinStats läser ditt bibliotek en gång efter den här uppdateringen, för att ta reda på var filer som innehåller flera avsnitt slutar.
- På en titels sida är **Lägg till i Min lista** lika hög som **Öppna i Jellyfin** bredvid; den var en storlek mindre.
- Filtren för period och personer stannar överst på skärmen medan sidan rullar, så du kan byta från 7 d till Alla även längst ner på en lång sida; detsamma gäller filtren för önskemål och kommande i På väg.
- Knappen för att gå till toppen håller sig undan den meny som har fönstrets nederkant: lågt i hörnet bredvid dockan, strax ovanför telefonens flikfält, tumknapp eller adressfält, och annars ovanför statusfältet. Det är FinUIs `to-top`.
- Att ta bort en uppspelning eller säkerhetskopia flyttar den till papperskorgen i stället för att ta bort den direkt (`PUT` med `{"deleted": true}`; `DELETE` är borta från API:t). Säkerhetskopior utöver det antal som sparas tas fortfarande bort för gott.
- **Ändringslogg** är ett enda ark i stället för en hög med kort: varje punkt är en egen rad med sin typ (Tillagt, Ändrat, Åtgärdat och de andra) och typens ikon i en kolumn framför, under en rubrik för varje serie. Listan över versioner står kvar till höger.

### Performance
- När FinStats öppnas hämtas bara det som har ändrats: en stilmall, ett skript eller ett typsnitt som webbläsaren redan har bekräftas med några få byte i stället för att skickas på nytt. Varje besök brukade hämta alla FinStats stilmallar och moduler i sin helhet, fast var och en hade en tagg som skilde dem åt.

### Stability
- Att återställa en säkerhetskopia läser inställningarna bara från inställningsraden: en fil från en nyare FinStats som sparar mer där tappade sina inställningar utan ett ord.
- En öppen flik växer inte längre för varje sida som besöks: varje sida som lämnades blev kvar i minnet bakom den på skärmen (ungefär 2 600 element per besök), fasthållen av FinMotions lista över det den sätter i rörelse och av filterfältets rullningslyssnare, tills fliken stängdes.
- Ett felskrivet kommando (`finstats bakcup`), eller `restore` utan sin fil, skapar eller uppgraderar inte längre en databas innan det säger att det inte vet vad det ska göra.

### Fixed
- En rutas lilla diagram går inte längre utanför rutan när talet bredvid är brett (”56 d 23 h”), eller när fyra rutor delar en smalare rad: diagrammet ger vika och talet behåller sin bredd.
- I det ljusa temat blev en primärknapp violett under pekaren; nu behåller den sigillets färg.
- I Inställningar står en brytare i jämnhöjd med inställningens namn och hjälptext; den hängde längst upp i raden.
- I en rankad lista med ingenting under ett namn, som Toppanvändare på Översikt, stod namnet längst upp i sin rad bredvid en centrerad bild; nu är det centrerat.
- En rad med statistikrutor i en bredd mellan telefon och dator (fyra rutor, tre i bredd) lämnade den sista ensam på en egen rad; fyra rutor ligger nu fyra, två eller en i bredd, och tre ligger tre eller en.
- Läsningen av servern räknade med en enhet som ingen hade loggat in på, och som den inte sparar: uppgiften sa tre enheter där sidan Server listade två.
- En sekund, en minut, en timme: larm sa ”1 sekunder emellan” och ”om 1 minuter”, och en uppgift i Jellyfin med ett intervall på en minut kördes ”var 1 minuter”.
- `finstats --help` nämner alla variabler som FinStats läser; `FINSTATS_PUBLIC_IP_URL`, `FINSTATS_ALLOW_LIBRARY_SHRINK` och `FINSTATS_SKIP_PREUPDATE_BACKUP` saknades.
- Installationsguiden sa om en adress som inte var någon adress att den ”inte kunde ansluta”, i en mening som flöt ihop med nästa; nu säger den att adressen inte är någon adress.
- I Inställningar → Säkerhetskopior försvann en fråga, som ställts medan en säkerhetskopia skrevs, så snart säkerhetskopian var klar.
- När menyn var en list som öppnas och sökningen stängdes visades den öppna sökningen och den stängda listen samtidigt, och sedan stod listen öppen tills pekaren kom och gick. Esc ger nu tillbaka tangentbordsfokus dit det var när sökningen öppnades, och släpper fokus som ett klick gav.
- När sökningen stängdes på telefonen förblev sidan nedtonad tills sökningen var borta och slog sedan om till full ljusstyrka på en enda bildruta; nu tonar nedtoningen bort tillsammans med sökningen.
- Stegen för Tautulli-importen sa att säkerhetskopian laddas ner efter **Backup Database**. Tautulli sparar den på sin egen dator: stegen säger nu var (mappen `backups`, `/config` med Docker), vilken fil du ska ta och att du ska kopiera över den.
- En sökning som stängdes innan den hade öppnats färdigt lämnade listen som öppnas utan sin animering: därefter öppnades och stängdes den tvärt.
- Listen som öppnas stod kvar öppen efter att en sida valts i den tills man klickade någon annanstans, och i vissa webbläsare, eller med inzoomning, ritade den en rullningslist under ikonerna. Nu öppnas den bara för pekaren och tangentbordet, och ritar ingen rullningslist när den är stängd.
- På sidan Säkerhet räknas en adress som databasen placerar i ett land men inte på kartan till det landet, i stället för bland adresserna utan känd plats.

## [2.1.2] - 2026-10-01

### Added
- **Import från Tautulli**: ta med din historik från Plex. Ladda upp Tautullis databassäkerhetskopia (`.db`-filen, eller `.zip`-filen som innehåller den) under **Inställningar → Import**; dess uppspelningar får en egen plats i filtret *Registrerad av* i Aktivitet.
- **Ett kopplingsbord för att säga vem som är vem**: Plex-användare på ena sidan, Jellyfin-användare på den andra. Dra en sladd från varje Plex-användare till den de är nu, eller klicka på den ena och sedan den andra, eller använd tangentbordet. Den som lämnas utan sladd importeras inte, och två Plex-konton kan föras över till en och samma person.
- Filmer och avsnitt matchas mot ditt bibliotek efter namn (en film efter titel och år, ett avsnitt efter serie och nummer), och ett som ännu inte finns på din server sparas och kopplas till när det kommer. En återupptagen visning är en enda uppspelning; musik importeras inte.
- Säkerhetskopian innehåller Plex åtkomsttoken: FinStats läser dem aldrig och tar bort filen så snart du har importerat eller börjar om.
- **Olänkad media** under **Inställningar → Data**: varje titel från Tautulli, Jellystat eller Streamystats som inte matchar något i ditt bibliotek, med var den kom ifrån och var den med störst sannolikhet redan finns. **Hitta** länkar den utifrån de bästa gissningarna eller en sökning (en film kan hittas som ett specialavsnitt i en serie), och FinStats kommer ihåg valet, så samma historik länkar sig själv när den importeras igen. Valen ingår i en säkerhetskopia.

### Performance
- **Återblicken öppnas tre till fem gånger snabbare**: årets mest sedda skådespelare och regissörer räknades ut genom att gå igenom året på nytt en gång för varje person i det. Mätt på en verklig historik gick ett år från 0,29 s till 0,09 s och de senaste 12 månaderna från 0,53 s till 0,12 s.

### Fixed
- **Importerad historik hittar titlar som går under ett annat namn**: en uppspelning matchar också en titels namn på originalspråket, så en serie som Plex eller Tautulli kände som 오징어 게임 kopplas till Squid Game, och *Im Westen nichts Neues* till *All Quiet on the Western Front*. Ditt biblioteks originaltitlar läses vid nästa genomsökning.
- Namn jämförs så som sökningen jämför dem (versaler, accenter, skiljetecken, ett årtal inskrivet i namnet, ett inledande ”The”), och ett utgivningsår får skilja ett år, så ”Star Wars: Episode I - The Phantom Menace” hittar ”… – The Phantom Menace” och en Kingsman från 2014 hittar den från 2015. En titel som är känd under ett helt annat namn gissas fortfarande aldrig.
- En misslyckad import eller återställning från Streamystats aviserades under sitt interna namn.
- Knappen *Dela året som kort* i återblicken satt utan mellanrum mot tittarplaceringen bredvid.

## [2.1.1] - 2026-10-01

### Changed
- **Min lista erbjuds överallt där en titel finns**: på korten i Kommer snart på Översikt, som i På väg → Kommande, och på ett avsnitts eller en säsongs sida, vilket lägger dess serie i listan.
- På en affisch på Översikt syns knappen för Min lista medan du pekar på kortet eller når det med tangentbordet, i stället för att täcka hörnet på varje affisch; på en pekskärm finns den alltid där.

### Stability
- Att spara en anslutning till Sonarr, Radarr eller Seerr medan någon hade **Inställningar → Utgående** öppen kunde få båda att vänta på varandra för gott.
- **Inställningar som inte går att läsa stoppar FinStats vid start**, med namnet på den inställning som är fel, i stället för att tyst återställa alla inställningar till standard.
- **Två ändringar av inställningarna samtidigt** (inställningssidan och en uppgifts schema, till exempel) tar inte längre ut varandra.
- En paus, ett hopp eller ett spårbyte som sågs medan databasen var upptagen skrivs vid nästa varv i stället för att gå förlorat.
- En läsning av biblioteket misslyckas inte längre när någons Min lista har flera poster som visar sig vara samma titel.

### Fixed
- **En ersatt fil är inte ”nu på servern”**: att uppgradera filen till en film i någons Min lista säger inte längre till dem att den precis har kommit.
- En enda misslyckad läsning av Jellyfins tillägg, schemalagda uppgifter eller lagring tömmer inte längre den listan på sidan Server fram till nästa läsning.
- **En uppspelning från Streamystats slutar där den slutade på varje sida**: en titels avhoppskurva och ett avsnitts ”färdigsett” gick i stället efter dess längd och räknade den som uppskattad.

## [2.1.0] - 2026-09-29 to 2026-09-30

Min lista

### Added
- **Min lista, bara din**: lägg en film eller en serie i den från dess sida, från Kommande, från Nyligen tillagt eller från sökningen (Ctrl+Enter), och hitta den under **Min lista** på din egen profil.
- **Varje post säger var den står nu**: på servern, ”5 av 26 avsnitt”, önskad, ”Kommer på fredag”, inte på servern, har lämnat biblioteket, eller sedd. Det räknas ut varje gång du tittar, med samma tolkning av ”sett” som din seriestatus. Sedda titlar ligger kvar, i en egen grupp, tills du tar bort dem.
- **En titel som ännu inte finns på din server** kan läggas i listan direkt från det som Sonarr eller Radarr väntar på. Den kopplas till titeln av sig själv när den kommer, och en fil som byter namn tar med sig sin post.
- **”I Min lista: nu tillgänglig”**: en avisering till dina egna mottagare när något du väntade på kommer. Avstängd tills du kryssar i den.
- **Ingen annan ser din lista**, inte ens en administratör, och dess avisering går bara till dina mottagare. Ingenting skrivs till Jellyfin, Seerr, Sonarr eller Radarr.
- Listorna ingår i en säkerhetskopia, och en återställning slår ihop dem utan att dubblera något.

### Changed
- Posterna i `/api/upcoming` har sin titels id från TVDB och TMDB, och `/api/auth/me` säger om servern har Min lista.

## [2.0.4] - 2026-09-29

### Added
- **Schemaläggning av uppgifter, så som Jellyfin gör det**: varje jobb som FinStats gör på egen hand har utlösare (dagligen vid en tid, varje vecka på en dag, med intervall, vid start, eller efter Jellyfins biblioteksgenomsökning), var och en med en valfri tidsgräns. Klicka på en uppgift för att se och ändra dess schema; *Lägg till utlösare* öppnar samma dialogruta som Jellyfin har.
- **Uppgifter är ett eget avsnitt** i Inställningar, med varje jobbs senaste körning i ord (”Kördes senast för 38 minuter sedan och tog 6 minuter”) och en Kör-knapp; tidigare låg det överst i System.
- **Metadataändringar**: varje timme läser FinStats det som Jellyfin har sparat sedan förra gången (namn, beskrivningar, genrer, betyg, medverkande, filinformation och bilder), så en ändring i Jellyfin syns utan att vänta på en biblioteksgenomsökning.
- Säkerhetskopiering och geolokaliseringsdatabasen kan också köras från Uppgifter; Esc går tillbaka från en uppgifts schema till listan.

### Changed
- **Temat är en brytare med tre lägen**, Ljust · Enhet · Mörkt, där knappen som växlade mellan dem satt: klicka på en tredjedel av den, dra knoppen eller använd piltangenterna. Knoppen bär valets ikon, som glider in när den ändras.
- **Utloggning finns under Inställningar → Konto**, som säger vem som är inloggad här; sidopanelen behåller temabrytaren.
- **De gamla schemainställningarna är nu utlösare**: *Följ Jellyfins biblioteksgenomsökning* med sitt reservintervall, *Säkerhetskopiera var N:e dag* och GeoIP-brytaren *Håll databasen uppdaterad* blev motsvarande utlösare på sina uppgifter, med de värden du hade. Insamling, Säkerhetskopior och Säkerhet länkar till schemat i stället.
- En start kör inte längre varje jobb igen: FinStats sparar när vart och ett senast blev klart.
- Ett jobb i Jellyfin som ännu saknar uppskattning visar ”ungefär … kvar”, med prickar som växlar där tiden kommer att stå, i stället för ”ETA…”.

### Performance
- **Kartan i Säkerhet laggar inte längre**: att dra och zooma flyttar den karta som redan är ritad och ritar den en gång när gesten är klar, och pulsen för det som spelas nu får inte längre hela kartan att ritas om 120 gånger i sekunden medan ingen rör den. En dragning gick från 34 till 60 bildrutor i sekunden på en långsam dator.

### Fixed
- **En affisch eller ett porträtt som byts ut i Jellyfin syns i FinStats**: bilder cachades en vecka på servern och ytterligare en vecka i webbläsaren, vad Jellyfin än hade fått sedan dess. Nu cachas de under Jellyfins bildtagg, så en ny bild är en ny fil, och webbläsaren kontrollerar att dess kopia är aktuell varje gång den visar en. Att byta ett porträtt sparar bara om personen, så Metadataändringar läser också personerna.
- **Server → Jobb visar en uppskattning så snart sidan öppnas**: återstående tid mättes bara från avläsningar som sidan själv gjorde medan den var öppen, så varje besök började på ”ETA…” tills den hade sett procenttalet röra sig. Nu tar FinStats själv tid på ett jobb som körs i Jellyfin, var tionde sekund medan ett körs, och sidan får uppskattningen serverad. Ett jobb som Jellyfin rapporterar på 0 % hela tiden (första fasen i en biblioteksgenomsökning) behåller sina prickar tills det rör sig: ingenting gissas.
- **Kartans verktygstips visas vid pricken**: i Firefox kunde verktygstipset för pricken för det som spelas nu hamna långt bort, och där prickar överlappar (någon som spelar hemma) berodde det på en enda pixel vilket verktygstips som visades. Nu nämner ett enda verktygstips allt under pekaren.
- **Statusfältet har en enda form vid hovring**: varje länk (strömmarna, Repo, versionen) lyser upp som samma lilla ruta, och prickarna mellan posterna sitter centrerade i mellanrummen. Att hovra över versionen brukade också lysa upp pricken och mellanrummet före den.
- **En sökväg tar hela kortets bredd**: på en titels Fil-kort, och för Jellyfins mappar på sidan Server, trycktes en sökväg ihop i en smal cell och rann ner över nio rader bredvid tomt utrymme.
- En dialogruta som tonar ut sväljer inte längre ett klick på sidan under den.

## [2.0.3] - 2026-09-28

### Security
- Sidan Server gav varje enhets id till alla med *Se servern*; id:n kräver *Se nätverksdetaljer* också, precis som överallt annars.
- Två behörighetsbrytare som slogs om innan den första hade sparats skickade samma utgångslista två gånger, och den andra upphävde den första: en behörighet som togs bort förblev beviljad, en del av en offentlig profil som stängdes av förblev publicerad, medan båda brytarna visade det som hade valts.

### Stability
- **Bara en import eller återställning åt gången**: två uppladdningar som skickades samtidigt klarade båda kontrollen, och en återställning tittade aldrig på importer; den andra väntade ut databasen och misslyckades efter flera minuters uppladdning.
- **En adress med en trasig `%`-kodning öppnar sidan Hittades inte**, i stället för att lämna en snurrande indikator för gott.
- **Ett stort sidnummer i en lista är en tom sida**: det flödade över och kom tillbaka som första sidan igen.

### Fixed
- **Tabeller sorterar tal rätt i en tysk eller norsk webbläsare**: ”1,204” lästes som ett decimaltal, och allt över en miljon som text.
- **Ett diagrams tabellvy sorterar datum, veckodagar och positioner efter tid**, inte efter bokstäverna.
- **Nedladdningshistoriken anger var importer kom ifrån**: Sonarr och Radarr anger indexerare och protokoll bara när något hämtas, så de uppdelningarna var alltid tomma. Ett misslyckat säsongspaket är ett enda misslyckande, inte ett per avsnitt.
- **Att ta bort en uppspelning som setts tillsammans** hittar den titelns grupper igen, i stället för att lämna en ”grupp” med en enda person.
- **Tid tillsammans för en person är aldrig mer än hen stannade**: sidan Tillsammans tillgodoräknade personen hela kvällens tid.
- **En kväll i sällskap över midnatt på nyårsafton** (eller kanten av ”senaste N dagarna”) hör i sin helhet till det fönster där den började, i stället för att bli två kvällar ensam.
- **Importerade uppspelningar av avsnitt kallas vid avsnittets namn** där Jellystat bara hade sparat seriens.
- **Geolokaliseringen hittar stadsdatabasen bredvid Country- och ASN-databaserna** som `geoipupdate` skriver, i stället för att stänga av sig själv.
- **Den mest tillbakaspolade punkten följer sidans filter**, precis som dess antal.
- **”Ditt år är klart”** går bara till personer vars återblick innehåller något.
- **Att lösa ett tystat larm igen behåller paret tystat**; bara att öppna det igen häver en tystning.
- **En API-nyckel som har gått ut räknas inte längre in bland de tjugo.**
- **Sessionsgränsen släpper den äldsta sessionen** även när många inloggningar landar i samma sekund.
- **En hemadress som uppslaget har hittat blir kvar** när samma adress tas bort från den inskrivna listan.
- **Bilder i granskningsloggen och nyckellistan**, där det bara fanns initialer.
- **En administratörs aviseringslista säger vems en personlig mottagare är**, i stället för ”Din” på allas; **Nyligen skickat** visar alla mottagare för den senaste händelsen.
- **Esc på en titel eller en person lämnar aldrig FinStats**, och stänger bara den översta dialogrutan.
- **En sida ritas även när en sparning avbryter den förhämtning den väntade på.**
- **Spelas nu visar en pausad tittare som spolar** där hen är nu, inte först när hen trycker på play.
- **Att ta bort en anslutning uppdaterar vilka sidor som finns**, precis som att spara en gör.
- **Geolokaliseringsdatabasens byggdatum behåller sitt årtal** på amerikansk engelska.

## [2.0.2] - 2026-09-28

### Security
- Bakom en proxy räknade inloggningsgränsen den första `X-Forwarded-For`-adressen, som är den klienten skickar: en ny påhittad adress för varje försök, och gränsen slog aldrig till. Nu är det den sista, den som proxyn skrev.
- *Ta med IP-adresser och platser* skickade adresser till en personlig mottagare vars ägare saknar *Se nätverksdetaljer*, adresser som ingen sida skulle visa för ägaren. Någon annans misslyckade inloggningar kräver nu *Se servern* också, som på sidan Säkerhet.

### Stability
- **Ett stopp går aldrig förlorat**: när en läsning av biblioteket vägrades under de första ögonblicken efter start, innan servern lyssnade efter det, noterades vägran och FinStats körde vidare i stället för att stanna.
- **En Jellyfin som svarar i en form som FinStats inte kan läsa avvisas**, i stället för att bli trodd: varje användare, eller varje titel i ett bibliotek, som kom under nycklar den inte kände till (`id` för `Id`) klarade spärren med ett fullt antal och markerades sedan som borttagen.
- **En uppspelnings slut tappas aldrig**: när databasen var upptagen längre än sin väntetid (en import som höll den) stod en uppspelning som tog slut i det ögonblicket kvar som pågående till nästa omstart. Nu skrivs den vid nästa varv.
- **Inställningar som en återställning inte kan läsa lämnas i fred**: en säkerhetskopia vars inställningar inte längre passade den här versionen nollställde varje inställning vid återställningen: sammanslagningsfönster, behörigheter, hemadresser, schema.
- **Ett styrtecken i ett namn eller en titel förstör inte längre korten**: profilkortet, varje berättelsekort som visade den titeln och ZIP-filen med dem svarade med ett fel.

### Fixed
- **Aviseringarna upphör för den som har inaktiverats eller tagits bort i Jellyfin**: deras personliga mottagare fortsatte att få allt som deras gamla behörigheter tillät.
- **En köad avisering prövas igen innan den skickas**: en mottagare som hade stängts av, eller en ägare som hade förlorat en behörighet, fick ändå det som väntade på ett nytt försök.
- **En mottagare som alltid svarar `Retry-After` överges** efter fem försök som alla andra, i stället för att få nya försök i trettio dagar.
- **Enhets-id:n på en persons sida** visas bara med *Se nätverksdetaljer*, som överallt annars.
- **Ett publicerat år väntar ett dygn med allt**: dess avslutade och övergivna serier följde varje uppspelning, så att trycka på play på en övergiven serie ändrade den offentliga sidan inom några sekunder.
- **En önskad serie räknas som sedd** när ett av dess avsnitt är det: varje önskad serie lästes som aldrig sedd, i kapitlet Önskemål och på kortet Önskat.
- **Kapitlet Tillsammans i återblicken lämnar Live TV utanför**, som resten av året: en kanal som stod på medan man tittade tillsammans kunde bli topptiteln, och andelen kunde passera 100 %.
- **Kortet Dagar för ”de senaste 12 månaderna” slutar på den senaste dagen**: rutnätet tappade de tre eller fyra senaste veckorna.
- **En series nya avsnitt aviseras en gång per kväll**: avsnitt på var sin sida av midnatt UTC aviserades två gånger på en server öster eller väster om den.
- **Att ansluta Seerr aviserar inte gamla önskemål som klara att se**: ett önskemål som redan var tillgängligt när det först lästes, utan någon annan uppgift om när, lästes som om det kom i det ögonblicket.
- **Titlarna i ett bibliotek som tas bort i Jellyfin följer med det**: de låg kvar i sökningen, på hyllan Nyligen tillagt och i totalerna.
- **En remux förblir en direktströmning** när en avläsning tappar sina omkodningsuppgifter för ett ögonblick (ett hopp), i stället för att bli en omkodning resten av uppspelningen.
- **”Jellyfin är nere” betyder att läsningar misslyckades i följd**: enstaka överskridna tidsgränser med flera dagars mellanrum, på en server som låg tyst på socketen, räknades ihop till det.
- **En import från Streamystats visar sitt förlopp på sitt eget kort**, inte ovanpå Jellystat-importens senaste resultat.
- **En adress vars värdnamn börjar med ”web”** (`http://web:8096`) läses som ett värdnamn, i stället för att kapas som sökvägen till Jellyfins webbklient.
- **Ett fel i kön döljer hela inloggningen** i en adress även när lösenordet innehåller ett snedstreck.

## [2.0.1] - 2026-09-28

### Changed
- **En titels språk håller sig ur vägen**: sidhuvudet nämner de tre första ljud- och undertextspråken, en rad vardera, och ett nytt kort **Språk** listar vartenda ett: de kompletta tillsammans, de ofullständiga med hur många avsnitt som har dem. En lång series två dussin undertextspråk brukade trycka ner bilden en hel skärm.

### Fixed
- **En titels affisch stannar bredvid namnet**: när texten bredvid blev längre än affischen (en lång handling, många studior) sjönk affischen till botten av sidhuvudet, under en tom bild.
- **En version som FinStats inte kan läsa kallas just det**: en `app_version` som redigerats för hand till något oläsligt fick FinStats att vägra starta som ”den äldre FinStats 2.0.1”, vilket det inte var. Det vägrar fortfarande, och säger nu vilket värde den inte kunde läsa och hur det ska skrivas.

## [2.0.0] - 2026-09-25 to 2026-09-28

Återblick 2026, offentliga profiler och att titta tillsammans

### Added
- **Vem du tittade med**, i återblicken: kvällar i sällskap, tiden ni tillbringade tillsammans, titeln som förde samman flest, och upp till tre sällskap, namngivna i appen och länkade till sina profiler.
- **Serier sedda till slutet**, i återblicken: det sista avsnittet av varje serie som sågs klart det året, med samma tolkning av ”sett” som profilens förloppsstaplar, och även de som påbörjades och lämnades till senare, mindre än hälften sedda och inget spelat på två månader.
- **Vad du önskade**, i återblicken: önskemål i Seerr som gjordes under året, hur många som kom och hur många du sedan såg. Bara där det finns önskemål.
- **Året mot året innan**: timmar, uppspelningar och dagar, och hur mycket mer eller mindre.
- **Året som en berättelse**: varje kapitel som ett kort i 1080×1920, formatet som telefonernas berättelser använder (året, siffrorna, toppserier, filmer och musik, genrer, personan, timmar och dagar, varje dag under året, rekord, sällskap, avslutade serier, önskemål och året innan). Spara ett, eller alla som en ZIP, från ”Dela året som kort”.
- **Ett kort nämner aldrig någon annan**: korten ritas från en kopia av året som saknar fält för en annan person, en placering eller en app. I appen bär ett kort inget namn alls.
- **Serverns år**, för administratörer i Jellyfin: varje titel och total och husets persona, utan att någon rankas eller nämns. En brytare på återblickssidan, `scope=server` i API:t.
- **”Din årsåterblick är klar”**: en avisering i december, en gång till varje person som tittade det året.
- **Offentliga profiler**: en person kan publicera en del av sin profil via en länk som vem som helst kan öppna utan konto (totaler och topptitlar, sviter och när hen tittar, hela året som sin berättelse i kort, de tio senaste uppspelningarna), varje del avstängd tills ägaren slår på den under Inställningar → Offentlig profil. Enheter, adresser, appar, uppspelningsmetoder, sökvägar och andra personer publiceras aldrig, vad som än är påslaget, och namnet som visas är ett som ägaren skriver in, aldrig hens inloggning i Jellyfin.
- **Ingenting på en publicerad sida är nyare än ett dygn**, varken senaste uppspelningar eller någon total, så en sida som någon håller ögonen på avslöjar aldrig vem som tittar just nu.
- **Delbara kort**: varje publicerad profil får en bild i 1200×630, av profilen och av året, som Discord, Signal, Mastodon och de andra visar när länken klistras in, och som kan laddas ner för att publiceras. Ritas på servern utifrån det som sidan visar och inget mer.
- **En länk du kan ta tillbaka**: 22 slumpmässiga tecken, återställs med ett klick; den gamla länken svarar då precis som en som aldrig har funnits.
- **Offentliga profiler är avstängda tills en administratör tillåter dem**: en brytare för hela servern, en lista över vem som publicerar vad, och ett sätt att ta ner en profil. Publiceringar, ändringar, återställningar och nedtagningar står i granskningsloggen, och en nyckel kan inte göra något offentligt.
- **Tillsammans**, en egen sida: vem som tittar med vem, hur mycket och hur det har förändrats. Tid tillsammans, kvällar och andelen av tittartiden i sällskap, var och en mot perioden innan; timmar i sällskap mot timmar ensam per dag eller vecka; varje par av personer med sina kvällar, sin tid tillsammans, när de senast tittade och vad de tittar mest på; titlarna som setts mest tillsammans; de senaste kvällarna; och, för den som får se alla, varje persons tid ensam mot tiden i sällskap.
- Varje par räknas: en kväll med tre personer räknas för vart och ett av dess tre par, där kortet förut bara räknade exakt samma uppsättning personer. Ett pars tid tillsammans är den kortare av de två vistelserna.
- Kortet **Sett tillsammans** på Översikt och på en profil länkar till sidan, och en profils kort säger hur stor andel av personens tittartid som var i sällskap.
- `GET /api/stats/groups` får `pairs`, `series`, `people`, `previous`, `bucket`, och `watch_s` och `share` i sina totaler; nycklarna den redan hade är oförändrade.
- **Var folk slutar**, på varje sida för en film eller ett avsnitt: andelen uppspelningar som fortfarande pågår vid varje minut, från alla vid början till dem som nådde slutet, med tillbakaspolningar och påslagna undertexter i de uppspelningarna markerade på samma axel. Kortet säger hur många stopp som mättes (uppspelningar som FinStats registrerade) och hur många som uppskattades (importerade uppspelningar, utifrån hur länge de pågick), och behöver tre uppspelningar innan det ritar något.
- **Vilka som fortsätter titta**, på en series sida: hur många personer som påbörjade varje avsnitt, i ordning, och en kolumn **Färdigsedda** per avsnitt. ”Alla ger upp vid avsnitt tre” är personer som aldrig trycker på play på avsnitt fyra.
- **Filer som aldrig spelas**, på Uppspelning: en film eller ett avsnitt som startats tre gånger eller fler och aldrig kommit förbi trettio sekunder, med vem som försökte och i vilka appar. Jellyfins logg innehåller inga uppspelningsfel, så uppspelningarna är vittnet; minsta uppspelningslängd gäller inte här, eftersom det är exakt den som de uppspelningarna aldrig når.
- **Mest tillbakaspolade** och **Undertexter påslagna**, på Uppspelning: titlar rankade efter hopp bakåt per uppspelning med den minut där de samlas, och efter andelen uppspelningar där undertexter slogs på inom de första tio minuterna. Alla tre listorna kräver *Se allas aktivitet*. Hopp kortare än 20 sekunder registrerades aldrig, så en kort repris av en enda replik är osynlig med avsikt.
- Ett hopp behåller varifrån det kom som ett tal bredvid sin etikett (`from_s` i en uppspelnings tidslinje). Varje hopp som redan har registrerats får sitt tal från etiketten, och det gör även en återställd säkerhetskopia från tiden före detta.
- `GET /api/stats/files` för de tre listorna; `GET /api/items/{id}` får `insights`, och en series avsnitt får `users` och `finished`.
- **API-nycklar**: alla som är inloggade kan skapa nycklar åt sig själva under Inställningar → API-nycklar, för ett skript, en instrumentpanel eller en telefon. En nyckel skickas som `Authorization: Bearer fs_…`, visas en gång, lagras hashad, kan ha ett utgångsdatum och återkallas med ett klick; en administratör ser och kan återkalla allas. Den motsvarar samma person med samma behörigheter som en inloggning, läst direkt vid varje begäran, så att förlora rätten att logga in stoppar alla nycklar på en gång. En nyckel kan varken skapa eller återkalla nycklar.
- **Ett kalenderflöde**: `GET /api/calendar.ics?key=…` är det som Sonarr och Radarr har på gång, som en iCalendar som en telefon kan prenumerera på. En nyckel med omfånget *kalender* öppnar flödet och inget annat; flödet läser nyckeln från adressen eftersom en prenumererad kalender inte kan skicka något huvud, aldrig från kakan, och nämner ingen.
- **En granskningslogg** under Server → Granskning, för administratörer i Jellyfin: varje inloggning och misslyckat försök med sin adress, varje inställning eller behörighet som ändrats och till vad, varje nyckel som skapats, använts eller återkallats, varje anslutning, mottagare, säkerhetskopia och import, varje uppspelning som tagits bort och larm som lösts, med vem, varifrån, genom vilken nyckel och om det lyckades. Sparas ett år och ingår i säkerhetskopior. `GET /api/audit` bläddrar igenom den.
- **Import från Streamystats**, ett eget avsnitt under Inställningar bredvid Jellystat. Ta dess fil från **Settings → Backup & Import → Download Backup** och släpp den i FinStats: år av historik kommer på ett par sekunder, och inget du redan har räknas igen. `finstats import-streamystats <file>` gör det utan gränssnitt, `POST /api/import/streamystats` via API:t; en import körs åt gången, oavsett sort.
- **Har du kört båda statistiktjänsterna? Importera båda filerna.** En uppspelning känns igen som en som FinStats redan har genom tjänstens eget id för den och, om det inte räcker, genom att samma person tittar på samma objekt med någon av uppspelningens ändar nära en i historiken, så en export från Jellystat och en från Streamystats med samma kvällar kan båda importeras, i valfri ordning, och en kväll som FinStats själv registrerade skrivs inte över av någon av dem. ”Nära” är sammanslagningsfönstret under Inställningar → Insamling.
- Importerade rader säger vad de inte kan säga. En uppspelning som Streamystats själv registrerade har inget kvar om filen (ingen kodek, upplösning, bithastighet eller container), så de fälten förblir tomma i stället för att fyllas med en gissning, och korten som skulle visa dem döljer sig själva. En uppspelning som den hade hämtat från Jellystat bär hela sessionen, och den läses i stället: daterad efter det ögonblick den tog slut, det enda ögonblick som någon av tjänsterna sparade, och beskriven som en omkodning bara när sessionen verkligen var det.
- Rader som Streamystats skrev för att Jellyfin rapporterade ett objekt som sett, för en visning den aldrig såg, räknas och hoppas över: de är lika långa som hela filmen och skulle bli tittartid en kväll då ingen tittade. Resultatet säger hur många det var.
- Hur data från Streamystats tolkas, fält för fält, lärt av en verklig export snarare än av dokumentation: `docs/streamystats-import.md`.
- **Vilken statistiktjänst varje uppspelning kom från**, som ett filter i Aktivitet: det som FinStats själv registrerade, det som kom från Jellystat, det som kom från Streamystats. Bara de som din historik faktiskt innehåller erbjuds, och det avgränsas som alla andra listor, så den som bara får se sina egna uppspelningar får bara veta var hens egen historik kom ifrån.
- **Filtren i Aktivitet är rullgardinslistor du kryssar i**, så du kan be om filmer *och* avsnitt utan musik, två uppspelningsmetoder eller två personer på en gång, i stället för en av varje eller alla. Mediatyp fick **Övrigt** nu när den kan kombineras.
- Flera personer på en gång, överallt där användarfiltret finns (Översikt, uppspelning, bibliotek, säkerhet och resten), och kartan och varje statistik följer med. Ett användarfilter kan fortfarande bara begränsa det någon får se: utan *Se allas aktivitet* låses en begäran till den som frågar, vem adressen än nämner och hur många.
- **Ett ljust tema**: washipapper, med bläcksvart text och en vermiljonröd accent. FinStats följer enhetens ljusa eller mörka inställning tills du väljer ett.
- **En temaknapp bredvid Logga ut** växlar mellan Enhet, Ljust och Mörkt. Valet sparas i den här webbläsaren och tillämpas innan sidan ritas, så det andra temat blinkar aldrig förbi.
- **Öppna i Jellyfin**: varje sida för en film, serie eller ett avsnitt har en knapp som öppnar titelns sida i Jellyfin, i en ny flik.
- **Jellyfins adress för användare** under Inställningar → Jellyfin: dit de knapparna pekar, för när adressen som FinStats ansluter till inte är en som en webbläsare kan nå. Tom behåller adressen som FinStats ansluter till; bara en administratör i Jellyfin kan ställa in den.

### Changed
- **FinStats har ett nytt hem**: `github.com/finstats/finstats`, och avbildningen är `ghcr.io/finstats/finstats`. Byt avbildningens namn i din `docker run` eller compose-fil för att fortsätta få uppdateringar; datamappen förblir som den är.
- **Inställningar visar ett avsnitt på skärmen åt gången.** Sidan var fjorton kort i en kolumn och sexhundra ord hjälptext. Nu har varje avsnitt en egen adress (`/settings/collection`), en lista till vänster växlar mellan dem, och på en telefon blir listan en rad med chips. Länkar till de gamla ankarna hamnar fortfarande rätt.
- **En inställning är en rad**: vad den är, en rad som anger regeln, kontrollen till höger. Tal i ett avsnitt sparas tillsammans med en knapp längst ner; brytare sparar sig fortfarande själva. Statuskorten (Uppgifter, Utgående anslutningar, Databas) och länken till licenserna finns under **System**, och de två importerna har ett eget avsnitt.
- **Hitta en inställning** från rutan i sidhuvudet (`/` ger den fokus) med vilket ord som helst i dess namn; den säger i vilket avsnitt inställningen finns, och Enter landar på raden.
- **Inställningar är till för alla som är inloggade**: avsnitten som hanterar servern stannar hos dem som får hantera den, och en person som bara fick aviseringar kan äntligen nå sina egna mottagare.
- **Sidan Server visar ett avsnitt på skärmen åt gången**, precis som Inställningar: Översikt (system och lagring), Jobb, Enheter, Tillägg, Logg och Granskning, var och en på sin egen adress. Länkar till `/server#jobs` hamnar fortfarande rätt.
- **Serverloggen är ett avsnitt på sidan Server** (`/server/log`) i stället för en egen sida i menyn; `/events` skickar vidare dit med sina filter.
- En avisering om ett misslyckat jobb, en misslyckad säkerhetskopia eller en anslutning som slutade svara länkar till det avsnitt i inställningarna som hör till, i stället för till toppen av sidan.
- Det som ditt bibliotek kan berätta om en importerad uppspelning (vilken sorts sak det var, vilket avsnitt, dess speltid, vilket bibliotek den hör till) fylls i efter **varje** läsning av biblioteket, inte bara i ögonblicket för en import. Historik importeras oftast innan FinStats någonsin har läst biblioteket, så de uppspelningarna förblev gissade eller okända för gott.
- Att återställa en säkerhetskopia av FinStats lägger inte längre till en uppspelning som kom från en statistiktjänst under ett annat id, och behåller fortfarande två verkliga visningar av samma sak med några minuters mellanrum.
- En uppspelnings detaljer nämner den statistiktjänst som tog in den, i stället för att kalla allt importerat för en uppspelning från Jellystat.

### Performance
- **En omstart går nästan direkt oavsett historik.** Varje start räknade ut vem som tittade tillsammans över hela historiken och avgjorde på nytt vilka uppspelningar som var lokala, båda för att inte ändra något: 24 s och 5,5 s på tio miljoner uppspelningar. Nu gör en start bara det som förra körningen lämnade efter sig, och allt igen bara efter en uppdatering eller när inställningen bakom har ändrats.
- **En folkmassa som kommer på en gång registreras på några sekunder.** Varje ny uppspelning, varje sparat förlopp och varje enhet skrevs för sig, och varje avslutad uppspelning stängdes med en egen genomgång av titelns hela historik; nu skriver ett varv varje sort på en gång. 100 000 uppspelningar som startar samtidigt tar 3 s, där 25 000 som slutade samtidigt förut tog över fyra minuter.
- **Spelas nu går fort även med tusentals tittare.** Att hitta vilka som tittar tillsammans jämförde varje ström med varje annan och listade varje sällskap med namn: fyrtio sekunder vid 10 000 strömmar, och flera gigabyte minne utöver det. Nu jämförs strömmar inom en titel, och var och en visar några namn och hur många andra.
- Att räkna ut vem som tittade tillsammans skrev om varje grupperad uppspelning i historiken varje gång FinStats startade, importerade eller såg en uppspelning ta slut; nu skrivs bara det som har ändrats. Att hitta vilka uppspelningar som hade slutat blir inte längre långsammare med kvadraten på antalet som spelas.
- **Att känna igen en uppspelning som historiken redan har går via ett index**, så en import av 150 000 uppspelningar tar några sekunder, och omlänkning efter en läsning av biblioteket stegar igenom titlarna i stället för att läsa varje uppspelning medan den håller databasen låst.
- **Uppspelningsinsikter och översikten går snabbare över hela tiden**: deras oberoende delar räknas ut sida vid sida, diagrammet över samtidighet slår upp den lokala dagen en gång per kvart i stället för en gång per uppspelning, och genrer räknas per titel. På en miljon uppspelningar: 4,1 s till 2,4 s, och 1,6 s till 1,1 s.

### Stability
- **Ett tv-bibliotek läses inte längre som ett bibliotek som har tömts.** Att läsa ett bibliotek görs i två omgångar (varje objekt, sedan en mindre för medverkande i filmer och serier), och kontrollen som avgör om en läsning går att lita på för att ta bort det den inte såg tittade på den andra. I ett bibliotek med tv eller musik är de två antalen inte i närheten av varandra, så kontrollen vägrade varje läsning och stoppade FinStats, och installationen startade om och gjorde samma sak igen. Ingenting togs någonsin bort felaktigt, men ingen läsning av biblioteket kunde slutföras. Om ditt bibliotek med serier eller musik inte har plockat upp nya titlar är det här orsaken.
- En import kunde misslyckas med ”database is locked” när en annan skrivning (en uppspelning som uppdaterades) landade mellan dess första läsning och dess första skrivning. Båda importerna tar skrivlåset när de börjar, och databasens journal förblir inte längre lika stor som den största import som någonsin har gjorts.
- **En databasfil som är skadad, avkortad, någon annans eller skrivskyddad avvisas direkt**, med namnet på filen och beskedet att ingenting ändrades, i stället för 30 tysta sekunder; i ett annat programs databas skrivs aldrig något.
- En säkerhetskopia eller ögonblicksbild före uppdatering som avbryts halvvägs lämnar inte längre en fil efter sig, en som kunde ta en bra ögonblicksbilds plats bland de tre som sparas.
- **Att skriva över platsdatabasen medan FinStats körs kraschar den inte längre**: FinStats läser från en privat kopia av filen.
- En app för Jellyfin som rapporterade ett nytt enhets-id varje gång kunde få FinStats minne att växa så länge den körde; enheter som inte har setts på fem minuter glöms nu bort.
- En person har högst 30 inloggade sessioner; den äldsta loggas ut vid nästa inloggning.

### Fixed
- **Samma kväll räknas inte längre två gånger när ett objekt har bytt namn.** Att länka om historik till en titel som lagts till på nytt i Jellyfin kunde göra en uppspelning som importerats från en statistiktjänst till en dubblett av en som redan fanns, eftersom kontrollen som skulle ha fångat den kördes innan id:t flyttades. Nu körs den igen överallt där id:n länkas om. Bara importerade uppspelningar tas någonsin bort, aldrig en som FinStats själv registrerade, och hur många som försvann står i loggen. Första starten av 2.0 går igenom hela historiken en gång efter dem som redan finns.
- **En session följer nu personen så som hen ser ut i Jellyfin just nu.** En administratör som Jellyfin degraderade behöll full åtkomst till FinStats resten av sin session på 30 dagar, och någon som inaktiverats eller tagits bort i Jellyfin förblev inloggad. Nu förlorar de den så snart FinStats läser Jellyfins användare (var 15:e minut, eller direkt från Inställningar), precis som API-nycklar alltid har gjort.
- Inloggning skriver det Jellyfin säger om personen till FinStats användarlista, så en importerad säkerhetskopia från en statistiktjänst kan inte längre göra någon till administratör innan första läsningen av användarna rättar det.
- Någon som har tagits bort i Jellyfin markeras som borttagen även när två läsningar av användarna hamnar i samma sekund.
- **Nedladdningsfel visar aldrig en nedladdningsklients lösenord eller en indexerares API-nyckel**, vilka Sonarr och Radarr kan citera i dem.
- Uppspelningar som länkats om till en titel med nytt namn under en läsning av biblioteket kontrollerades inte för gemensamt tittande förrän vid nästa omstart; nu görs det direkt.

## [1.6.5] - 2026-09-24

### Added
- **En fullständig säkerhetskopia tas automatiskt före varje uppgradering.** Första gången en nyare FinStats öppnar din databas skriver den en komplett kopia av den (hela din historik, inställningar och allt) till `data/pre-update-backups/` *innan* den ändrar något. Om en uppgradering någon gång förstör något är dina data säkra: stoppa FinStats, lägg tillbaka kopian i stället för `finstats.db`, starta den version du hade och rapportera felet. Ingenting går förlorat. De senaste kopiorna sparas; sätt `FINSTATS_SKIP_PREUPDATE_BACKUP=1` för att stänga av det om det är ont om diskutrymme.

## [1.6.4] - 2026-09-24

### Added
- **Fem nya ställen att skicka aviseringar till**: Telegram, Slack, Pushover, Pushbullet och e-post, bredvid de mottagare för webhook, Discord, ntfy och Gotify som redan fanns. Var och en frågar bara efter det den behöver (ett chatt-id, en användarnyckel, en brevlåda), och de tre som finns hos en enda tjänst (Telegram, Pushover, Pushbullet) kan bara någonsin nås där, så en token kan inte skickas till en värd som ser likadan ut. E-post skickas över TLS från första byten eller med `STARTTLS`, och det finns inget tredje alternativ.
- **Licenser**, under **Inställningar**: FinStats egen GPL och varje licens från tredje part som det är byggt på (253 Rust-paket, de två medföljande typsnitten, kartans konturer och geolokaliseringsdatabasen), varje text i sin helhet, så som dess eget projekt skrev den. Välj en komponent så öppnas dess licens i fönstret bredvid listan; ett paket som levereras med flera filer växlar mellan dem på plats. Listan genereras från de paket som FinStats faktiskt byggs av, så ett beroende kan inte läggas till utan att dess licens registreras.

## [1.6.3] - 2026-09-24

### Fixed
- Säkerhetsspärren griper nu också in när Jellyfin inte listar *några* bibliotek eller *några* användare alls (en trasig läsning, aldrig en vanlig dag), vilket justeringen i 1.6.2 hade släppt igenom. Att förlora några bibliotek behandlas fortfarande som något vanligt; att förlora vartenda ett på en gång gör det inte.

## [1.6.2] - 2026-09-24

### Fixed
- Dataintegritetsspärren som lades till i 1.6.1 var för ivrig: ett bibliotek som verkligen förlorade de flesta av sina objekt (en handfull klipp vars filer försvann, till exempel) togs för en trasig läsning, så FinStats vägrade uppdatera det och stannade. Nu griper den bara in vid en *uppenbart* trasig läsning: ett bibliotek som är så stort att det spelar roll och som läses tillbaka helt tomt, eller ett stort som tömts till nästan ingenting. Vardagliga ändringar tillämpas som förut, och fallet ”Jellyfin returnerade ingenting” fångas fortfarande.

## [1.6.1] - 2026-09-24

### Added
- **FinStats litar inte längre på en läsning från Jellyfin som skulle radera din historik.** Om ett bibliotek som FinStats har tusentals objekt för plötsligt läses tillbaka tomt eller nästan tomt (en uppgradering av Jellyfin som ändrade dess API, ett fel förklätt till ett tomt resultat) behåller FinStats de data det har, markerar synkroniseringen som misslyckad (och aviserar) och stannar rent med ett tydligt meddelande i stället för att markera allt som borttaget. Starta om när Jellyfin är sig själv igen, eller sätt `FINSTATS_ALLOW_LIBRARY_SHRINK=1` om du verkligen har tömt ett bibliotek. Samma spärr gäller listan över bibliotek och listan över användare.

### Security
- Ett `X-Forwarded-Host`-huvud som skickades av en klient kunde gå i god för en främmande `Origin` och få en skrivning från en annan webbplats förbi kontrollen av samma ursprung. Det huvudet litas nu bara på bakom en betrodd proxy (`FINSTATS_TRUST_PROXY`), samma regel som uppslaget av klientens adress redan använder. Sessionskakan med `SameSite=Lax` var, och förblir, den första försvarslinjen.
- En personlig aviseringsmottagare kunde riktas mot `0.0.0.0` eller `::`, som når den här datorn. Kontrollen som håller personliga mottagare riktade mot offentliga adresser räknar nu dem som lokala, tillsammans med loopback och de privata intervallen.

### Fixed
- Sidan **Uppspelning** misslyckas inte längre att läsas in för ett bibliotek som innehåller en uppspelning utan registrerad metod (en rad som återställts från en säkerhetskopia äldre än kolumnen); en sådan uppspelning räknas helt enkelt som en direktuppspelning.
- Ett mycket långt ord i en sökning eller ett filter är inte längre ett serverfel: SQLite vägrar ett `LIKE`-mönster längre än 50 000 tecken, så filterord kortas till en rimlig längd före frågan.
- Sidan **Inställningar** rullar inte längre i sidled på en telefon när ett kort rapporterar tillbaka en hel mening, som resultatet av en återställning.

## [1.6.0] - 2026-09-24

Aviseringar

### Added
- Kortet **Aviseringar** under Inställningar: lägg till en mottagare, kryssa i vad den ska få höra om, tryck på Testa. Ingen mottagare betyder att ingen begäran skickas någonstans.
- Fyra sorters mottagare: en vanlig **webhook** (JSON, dokumenterad i `docs/api.md`, valfritt `Authorization`-huvud), **Discord** (en inbäddning via en kanals webhook-URL), **ntfy** (ntfy.sh eller din egen, till ett ämne) och **Gotify** (din egen server, med en applikationstoken). Titlar skickas i meddelandet och inte i ett huvud.
- Elva sorters händelser, var och en med sin egen kryssruta: omöjlig resa, ett konto som setts i ett nytt land, en våg av misslyckade inloggningar, en uppgift som misslyckades, en säkerhetskopia som inte kunde skrivas, något som slutade svara (Jellyfin, Sonarr, Radarr eller Seerr) och när det svarar igen, nya titlar i biblioteket (sammanslagna per serie och dag, som hyllan på Översikt slår ihop dem), ett önskemål som blev möjligt att se, och en uppspelning som börjar eller slutar. De två sista är avstängda som standard.
- Ett önskemål som blev möjligt att se nämner titeln, vem som önskade den och hur länge de väntade.
- Personliga mottagare: ge **Få aviseringar** så kan någon lägga till en egen, som bara får det som personen redan får se: sina egna önskemål och larm alltid, andras med *Se allas aktivitet*, adresser med *Se nätverksdetaljer*. En personlig mottagare måste peka på en offentlig adress; bara en administratör får rikta en inåt i nätverket.
- **Ta med IP-adresser och platser**, per mottagare och avstängt som standard: ett meddelande om omöjlig resa nämner annars bara platser (”Oslo, Norge och London, Storbritannien”).
- **Nyligen skickat**, i samma kort: vad som sades, till vilken mottagare och vad andra sidan svarade när det inte kom fram. Ett misslyckat meddelande försöks igen fyra gånger under ungefär en timme, och FinStats väntar in en mottagare som begränsar takten.
- **FinStats adress**, en inställning för administratörer, är det som länken i en avisering byggs från. Tom betyder att meddelanden inte har någon länk.
- Jellyfins egna schemalagda uppgifter på sidan Server: vad var och en gör, i ord i stället för med namnet på dess kod, och om det är en av de tunga. En uppgift som körs visar en förloppsstapel, ett procenttal och hur mycket som är kvar; de andra visar när de körs nästa gång, när de senast kördes och om det gick bra. Uppgifter som Jellyfin döljer på sin egen kontrollpanel tas med, och ingenting i FinStats kan starta eller stoppa en uppgift.
- Hur mycket som är kvar av en uppgift mäts från den takt som dess procenttal faktiskt har rört sig i, räknat mot den senaste avläsningen som ligger tillräckligt långt tillbaka för att säga något, i stället för genomsnittet av hela körningen, så en uppgift som går fortare eller långsammare beskrivs med den takt den har nu.
- De uppgiftslistor som FinStats redan läser av andra skäl (kontrollen av biblioteksgenomsökningen var 5:e minut, serveruppgifterna var 15:e) matar samma mätning, så en uppgift som har kört i en timme brukar ha en uppskattning i samma stund som sidan öppnas. Ingenting extra begärs av Jellyfin för det.
- En uppgift som körs och där inget ännu har mätts visar **ETA** och en snurrande ellips i stället för ett tal, och en vars procenttal inte har rört sig på en halv minut visar ”68 % i 4 min” bredvid. Ingenting ersätter en uppskattning som inte har mätts.
- En lista över alla versioner bredvid ändringsloggen: ett klick tar dig till vilken version som helst, den markerar den som läses, och på en smal skärm blir den en rad med serier ovanför ändringarna.

### Changed
- Ändringsloggen fälls inte längre ihop. Varje serie står öppen och du tar dig runt med listan över versioner, i stället för en grupp öppen åt gången och ett klick för att nå någon annan.
- **Inställningar → Utgående anslutningar** listar varje aviseringsmottagare tillsammans med Jellyfin och tjänsteanslutningarna: vad den är, dess värd, om den är påslagen och när den senast tog emot ett meddelande.
- Ingenting i insamlingen, historiken eller någon siffra på skärmen ändras. FinStats *läser* fortfarande bara från Jellyfin, Sonarr, Radarr och Seerr.

## [1.5.2] - 2026-09-22

### Changed
- En läsning av Jellyfins sessionslista och återställningen av klockan som bad om den är ett och samma anrop, så de två kan inte längre glida isär. Det var formen på felet med skenande förfrågningar som hittades medan 1.5.0 byggdes.
- Att bevisa det tar en bråkdel av en sekund i stället för sju minuter mot en verklig server, så det kontrolleras varje gång något ändras. Samma förfrågningar, samma historik, samma siffror på skärmen.

## [1.5.1] - 2026-09-22

### Fixed
- En omkodning registreras en gång, inte en gång i sekunden. När en uppspelning börjar med omkodning och sedan går tillbaka till direktuppspelning lämnar Jellyfin omkodningsuppgifterna kvar på sessionen; varje avläsning jämfördes med ett värde som FinStats just hade skrivit över, så varje avläsning såg ut som en ändring och sidan Aktivitet fylldes med rader om ”Omkodning” med ”Direktuppspelning: …” under.
- Rader som redan har skrivits rensas vid start: där en följd av dem säger samma sak behålls den första. En uppspelning som behövde omkodas räknas fortfarande som en omkodning.

## [1.5.0] - 2026-09-22

Sessionsspårning över WebSocket

### Changed
- FinStats håller en anslutning öppen till Jellyfin från det ögonblick det startar, och det är så det får höra om en uppspelning. 1.4.0 levererade det som en inställning, avstängd som standard, för att bevisa sig mot verkliga servrar; nu är det alltid på. Samma uppspelningar, pauser och hopp, uträknade av samma kod.
- Ingenting spelas: FinStats frågar Jellyfin om ingenting alls. En kväll då ingen tittar kostar inga förfrågningar.
- Något spelas: sessionslistan läses en gång i sekunden. Det är där en paus, ett hopp eller ett spårbyte får sin skärpa, och det är det som avslutar en uppspelning vars klient försvann utan att säga adjö.
- Allt som är laddat har varit pausat i tre avläsningar i rad: frågandet upphör och anslutningen tar över, där en pausad film förut kostade samma fråga varje sekund, ungefär 16 MB i timmen. Ett återupptagande, eller att någon annan startar något, besvaras inom ungefär en sekund, och pausad tid räknas fortfarande aldrig som tittartid.
- Varje utgående läsning ber om komprimerade svar och packar upp dem (Jellyfin, Sonarr, Radarr och Seerr komprimerar alla när de blir ombedda), vilket minskar de två största läsningarna med mer än hälften: sessionslistan under en uppspelning och läsningen av biblioteket. Geolokaliseringsdatabasen laddas fortfarande ner som den lagras.
- Jellyfin-kortet i Inställningar säger vilken av de två halvorna som körs, och live-pricken följer om anslutningen bär trafik snarare än vilken halva du är i.
- Bara en av de två körs åt gången: medan FinStats frågar säger det åt Jellyfin att sluta skicka, och ber det börja igen i samma stund som den sista uppspelningen tar slut. Anslutningen står öppen hela tiden.
- En Jellyfin som svarar på prenumerationen med ingenting blir trodd så länge den svarar på keep-alive: FinStats säger det en gång, läser sessionslistan själv under tiden och förblir prenumererad tills den första pushen avgör saken. ”Kan inte pusha” och ”höll på att starta om när vi frågade” ser likadana ut, så tystnad är aldrig en dom.
- Reservläget är orört: en anslutning som stängs, tystnar eller visar sig inte förstå det här faller tillbaka på att fråga med en timer redan vid nästa varv och fortsätter att återansluta, med intervallen **Medan någon tittar, kontrollera var** och **Medan inget spelas, kontrollera var**.

### Added
- Vad insamlaren gör, på `GET /api/status`: vilken av de fyra sakerna den håller på med (`idle_socket`, `playing_poll`, `paused_socket`, `fallback`), om anslutningen är öppen och prenumererad, hur ofta den frågar, hur många sessionsläsningar som gick ut den senaste minuten, och när det senast ändrades. Besvaras från minnet (ingen förfrågan, ingen databasfråga), och det innehåller inga namn, titlar eller antal.
- De fälten kontrolleras mot varandra vid varje varv, och en oenighet som består skrivs till loggen.
- En begränsare under varje sessionsläsning i processen: högst 2 i sekunden och 70 i minuten, där allt utöver det fördröjs, räknas och loggas högst en gång i minuten. Vanligt tittande håller sig gott och väl under den.

### Fixed
- Liveuppdateringar ersätter verkligen avfrågningen. När det var påslaget frågade 1.4.0 fortfarande efter sessionslistan var femte sekund och öppnade anslutningen på nytt var tjugonde till fyrtionde sekund, eftersom den bedömde anslutningen efter hur nyligen en lista hade kommit, och Jellyfin skickar en när något *ändras* och ingenting däremellan. Om anslutningen lever avgörs nu av om Jellyfin svarar överhuvudtaget, vilket kontrolleras med en keep-alive ungefär två gånger i minuten.
- En sessionslista som kom medan FinStats var mellan två varv kastades bort, i tron att nästa bara var en och en halv sekund bort. Nu sparas den och används.
- **Senast kontrollerad** under Inställningar, och kortet Utgående anslutningar, blir inte längre inaktuella medan anslutningen är tyst: Jellyfins svar på en keep-alive är det de visar.

### Removed
- Inställningen **Låt Jellyfin pusha det som spelas** under Inställningar → Insamling, och `live_socket` från inställnings-API:t.
- `socket_enabled` från insamlarens status: det finns inget ”avstängt” längre.
- `last_reconcile_at` från insamlarens status, ersatt av `socket_live`.

## [1.4.1] - 2026-09-22

### Changed
- Seerr läses bara i sin helhet när något har ändrats: var femte minut ber FinStats om en rad, det senast ändrade önskemålet, och varvet slutar där om det redan känner till det. Det är ungefär en kilobyte mot ungefär sextio för en sida med femtio. Önskemål som fortfarande är på väg kontrolleras varje kvart, allt listas en gång om dagen, och ett nytt önskemål dyker fortfarande upp inom fem minuter.
- Köerna i Sonarr och Radarr läses var femte sekund medan en sida visar dem, varje minut medan något ligger i kön och var femte minut medan den är tom, i stället för varje minut dygnet runt. Om sidan öppnas, en tjänst ansluts eller ett nytt önskemål kommer i Seerr uppdateras de direkt.

## [1.4.0] - 2026-09-21

Valfria WebSocket-sessioner, och överallt dit FinStats når

### Added
- **Låt Jellyfin pusha det som spelas** under Inställningar → Insamling, avstängt tills du slår på det: en anslutning bär sessionslistan, och ungefär 17 000 förfrågningar om dagen blir nästan inga. Uppspelningarna, pauserna, hoppen och grupperna räknas ut av exakt den kod som räknade ut dem förut. Medan det är aktivt gör FinStats fortfarande en vanlig förfrågan i minuten *medan något spelas*, och ingen när inget spelas. En anslutning som stängs, tystnar i femton sekunder eller visar sig inte förstå det faller tillbaka på timern vid nästa varv och fortsätter att återansluta.
- **Utgående anslutningar**, ett kort i Inställningar för administratörer i Jellyfin: varje mål som FinStats kan nå (din Jellyfin, tjänsten ”vad är min IP”, DB-IP:s databas, och varje Sonarr, Radarr eller Seerr), med vad det är till för, om det är påslaget och när det senast svarade. Byggt av det som FinStats redan vet, och det visar bara adresser: aldrig en nyckel, aldrig en bassökväg.
- **Slå upp nu** under Inställningar → Hemnätverk, för den dag din offentliga adress ändras.

### Changed
- Uppslaget ”vad är min IP” körs en gång, första gången FinStats behöver en adress, i stället för varje kvart.
- Jellyfin-kortet i Inställningar säger hur insamlaren får sina besked, och, när anslutningen är påslagen men inte bär trafik, varför inte.

## [1.3.0] - 2026-09-20

Sonarr, Radarr och Seerr, på en sida

### Added
- **Anslutningar** under Inställningar, för administratörer i Jellyfin: **Sonarr**, **Radarr** och **Seerr** (eller Jellyseerr/Overseerr), flera av samma sort, var och en testad innan den sparas. Din nedladdningsklient behöver ingen egen inställning, eftersom Sonarr och Radarr redan pratar med den och FinStats läser det de vet.
- **Önskemål**: vem som önskade vad, hur lång tid det tog att komma och om de någonsin såg det. Rutor för titlar som väntar och den typiska väntetiden, ett diagram över hur det har förändrats månad för månad, och listor över det som kom för flera veckor sedan och aldrig spelades, och vem som önskar mest.
- **Kommande**: nya avsnitt och filmsläpp från Sonarr och Radarr som en agenda dag för dag, markerade med om *du* tittar på serien och, för dem som får se alla, vem mer som gör det. En serie räknas som sedd när någon har spelat ett av dess avsnitt de senaste fyra månaderna. En rad ”Kommer snart” på Översikt, en per profil för de serier som personen tittar på, och vad som kommer härnäst för en titel på dess egen sida.
- **Nedladdningar**: den levande kön i varje Sonarr och Radarr (vad det är, hur långt det har kommit, hur snabbt, vad som gick fel vid import), med personen som önskade det bredvid. Ett säsongspaket är en rad hur många avsnitt det än innehåller, och det som behöver uppmärksamhet ligger överst. Under den: det som kom in under en vecka, en månad eller ett år efter indexerare, kvalitet och nedladdningsklient, och vilka nedladdningar som misslyckades. Översikt visar samma lista, förkortad.
- **Se vad som laddas ner**, en ny behörighet. Utan den ser man fortfarande hur långt ens *eget* önskemål har kommit och hur lång tid som är kvar, och inget annat: inga releasenamn, inga hastigheter, inga andra nedladdningar. Andras önskemål kräver *Se allas aktivitet*.

### Changed
- FinStats skickar `GET` till de här tjänsterna och inget annat: det finns ingen kod i FinStats som skulle kunna godkänna ett önskemål, starta en sökning eller lägga till, pausa eller ta bort en nedladdning.
- Deras API-nycklar lagras i FinStats egen databas, skickas aldrig tillbaka till webbläsaren, skrivs aldrig till en logg och ingår aldrig i en säkerhetskopia. Ingen omdirigering följs, så en nyckel kan inte färdas någonstans som du inte har angett, och certifikat verifieras om du inte stänger av det för en anslutning.

## [1.2.2] - 2026-09-20

### Added
- Ljud- och undertextspråk på varje film och avsnitt (”Ljud: japanska · engelska”).
- Hur långt en dubbning räcker: en serie eller säsong säger hur många av dess avsnitt som har varje språk (”Engelska: 13 av 26 avsnitt”), och avsnittslistan har en kolumn **Ljud**.
- **Ljudspråk** och **Undertextspråk** på bibliotekssidorna: hur många filer som kan spelas på vart och ett.
- FinStats läser om biblioteket en gång efter den här uppdateringen för att plocka upp språken. Det säger aldrig ”dubbad”, eftersom Jellyfin inte anger en titels originalspråk, och ett spår utan språktagg listas som ”Okänd”.

## [1.2.1] - 2026-09-20

### Changed
- Bilderna i README är tagna på nytt i den här versionen, med en ny av sidan **Säkerhet**. Ingenting i appen ändras.

## [1.2.0] - 2026-09-20

Varifrån folk tittar, på en karta

### Added
- **Sidan Säkerhet**, för dem som får se nätverksdetaljer och allas aktivitet: en världskarta med en prick för varje plats som dina användare tittar ifrån, vars storlek beror på hur mycket som händer där. Hemma är grönt, platser utanför hemmet är lila, liveströmmar pulserar och misslyckade inloggningar utifrån är röda. Dra för att flytta, zooma med knapparna, tangenterna eller Ctrl + hjul, och överlappande prickar slås ihop tills du zoomar in. Under den: varje plats med sina personer, uppspelningar, tittartid och inloggningar, och länderna efter uppspelningar.
- **Omöjlig resa**: ett larm när ett konto ses på två platser som inget flyg förbinder, eller på två avlägsna platser samtidigt, med båda iakttagelserna, avståndet, tiden mellan dem och hastigheten det skulle ha krävt. **Visa på kartan** ritar resan.
- **Nytt land**: första gången någon spelar eller loggar in från ett land där hen inte har setts förut.
- Lös ett larm med en anteckning, eller tysta ett par platser för den personen, för en VPN eller en telefon vars operatör sitter i huvudstaden. Larm som hittas i gammal historik, efter en import eller i en ny databas, arkiveras som lösta i stället för att översvämma listan. Larm och det du har beslutat om dem ingår i säkerhetskopior.
- Platser kommer från en stadsdatabas som läses lokalt: ingen adress skickas någonsin någonstans, och kartan ritas från konturer som följer med FinStats i stället för från en karttjänst. Ladda ner DB-IP:s kostnadsfria databas från sidan Säkerhet eller **Inställningar → Säkerhet**, låt FinStats uppdatera den varje månad (avstängt tills du slår på det), eller släpp din egen `.mmdb` (DB-IP, MaxMind GeoLite2-City) i `data/geoip/`. `FINSTATS_GEOIP_DB` pekar ut en fil någon annanstans.
- **Inställningar → Säkerhet**: databasen som används, den månatliga uppdateringen, och hur snabbt (900 km/h) och hur långt ifrån varandra (500 km) två iakttagelser måste vara för att räknas som omöjlig resa. Stadsdatabaser har ofta fel på några hundra kilometer, och det är därför avståndet finns.

## [1.1.3] - 2026-09-20

### Added
- Ctrl+Space söker också bland medverkande i allt i ditt bibliotek, under **Medverkande**, med foto, om de är skådespelare eller regissörer, och hur många av dina titlar de finns med i; att välja en öppnar deras sida. Lika förlåtande som titelsökningen (ord i valfri ordning, accenter ignoreras, en felslagen tangent tillåts), och när flera matchar lika bra kommer den som finns i flest av dina titlar först.

## [1.1.2] - 2026-09-20

### Added
- **Nyligen tillagt** på Översikt, ovanför diagrammet Aktivitet: de 30 senaste tillskotten som en rad affischer som du kan rulla i sidled, med pilar på en dator. Var och en säger när den kom och vad den är (en film, ett album eller ”Säsong 4 · 3 avsnitt”), och ett enstaka nytt avsnitt säger vilket, med säsongens egen affisch när den har en.
- **Skift + mushjul** och piltangenterna flyttar en affisch, Page Up och Page Down en skärmbredd, Home och End går till respektive ände, och svepning fungerar. Det vanliga hjulet rullar fortfarande sidan.
- Nya avsnitt av samma serie som kommer samma dag delar en post, och en hel serie som läggs till på en gång är en post (”3 säsonger · 60 avsnitt”), så en enda stor import trycker inte ut allt annat ur listan.
- Raden bryr sig inte om period- och personvalet på Översikt, eftersom den handlar om biblioteket och inte om uppspelningar, och döljer sig själv medan biblioteket är tomt.

## [1.1.1] - 2026-09-20

### Added
- Sidor öppnas direkt: en kort stund efter att FinStats har laddats hämtar det i tysthet det du sannolikt öppnar härnäst (Översikt, din profil och tidslinje, Aktivitet, Användare, Bibliotek och varje bibliotek, Uppspelning, serversidorna, återblicken och de mest aktiva personerna), så en sida visas direkt när du öppnar den och uppdateras ändå i bakgrunden.
- Filmer, serier och medverkande hämtas bara när du visar avsikt, eftersom de är tusentals: att låta pekaren vila på en länk, röra vid den eller nå den med tangentbordet hämtar just den sidan.
- Det väntar tills sidan du öppnade har laddats klart, tar en sida åt gången medan webbläsaren är ledig, pausar i en flik i bakgrunden, gör ingenting på en datasparande eller mycket långsam anslutning och glöms bort när du loggar ut.
- En liten **Repo**-länk med GitHub-märket i statusfältet, bredvid versionen.

### Fixed
- **Tidslinjen** glömde vilka bibliotek som var avstängda när sidan laddades om eller öppnades från ett bokmärke, fast valet fanns i adressen.

## [1.1.0] - 2026-09-20

En tidslinje över allt du har sett

### Added
- **Tidslinje**, bredvid **Översikt** på varje profil: ditt tittande som ett enda spår från i dag tillbaka till den första uppspelning som FinStats känner till, med en kväll av avsnitt hopslagen till ett enda stopp. Varje stopp är en affisch med vad som sågs och när (”Säsong 2 · Avsnitt 3–6”, en film och om den tog mer än en sittning, eller ett album och hur många spår), och öppnar titeln.
- Spåret följer fönstrets bredd: tre stopp per rad på en bred skärm, två på en smalare, varannan rad löper baklänges med en kurva som förbinder den med nästa, ända ner till en enda rak linje på en telefon. Det första stoppet i varje månad bär månaden.
- Äldre stopp laddas av sig själva medan du rullar, hela vägen tillbaka till där historiken börjar.
- **Bibliotek**-brytare ovanför spåret utesluter det du inte vill se, till exempel musik, och valet ingår i adressen.
- Något som fortfarande spelas säger **Spelas nu**.
- Du ser din egen tidslinje; andras kräver behörigheten ”se alla”.

## [1.0.5] - 2026-09-20

### Added
- Skydd mot nedgradering: databasen kommer ihåg den nyaste versionen av FinStats som har öppnat den, och en äldre version vägrar starta på den i stället för att tyst arbeta med data den inte helt förstår. Meddelandet säger vad du ska göra. Versioner till och med 1.0.4 släpptes innan den här kontrollen fanns.

## [1.0.4] - 2026-09-20

### Fixed
- I en uppspelnings detaljer hade kopieringsknappen bredvid ett långt **Enhets-id** hamnat på en egen rad.
- På sidan Uppspelning var sista kolumnen i **Vilka klienter som omkodar** avkapad (”Omkod…”); rubriker på två ord radbryts nu.

### Changed
- Skärmbilderna i README är aktuella igen, efter att fortfarande ha visat version 0.5.0.

## [1.0.3] - 2026-09-20

### Added
- En avsnittssida visar avsnittets egen bild, den som Jellyfin visar i sin avsnittslista, i stället för seriens affisch. Ett avsnitt som Jellyfin saknar bild för behåller affischen.

## [1.0.2] - 2026-09-20

### Fixed
- I **Ändringslogg** hade grupperna v0.9 och v0.10 ingen rubrik bredvid sin version. En grupps rubrik är nu alltid den första meningen av det som dess första version handlade om, så en lång inledning kapas inte mitt i en mening.

## [1.0.1] - 2026-09-20

### Fixed
- Den publicerade avbildningen startade inte på en ny dator: ”unable to open database file: /data/finstats.db”, efter att ha hängt i en halv minut. Docker skapar en saknad `data`-mapp som root och FinStats körs som en vanlig användare. Nu gör containern mappen till sin egen och byter sedan till den användaren, så `docker run` fungerar på första försöket. För filer som ägs av någon annan än användare 1000, sätt `PUID` och `PGID`; `--user` fungerar som förut.
- En datamapp som verkligen inte går att skriva till rapporteras direkt, med kommandot som löser det, i stället för 30 sekunder senare som ett databasfel.

## [1.0.0] - 2026-09-20

Redo för alla

### Added
- En färdig avbildning, nu på `ghcr.io/finstats/finstats`, för 64-bitars Intel/AMD- och ARM-datorer (en Raspberry Pi 4 eller 5 fungerar): `:latest` är den senaste versionen, en versionstagg står kvar där den är, och `:edge` är utvecklingsversionen.
- Varje version visas på projektets releasesida på GitHub med den här ändringsloggen.

### Changed
- Versionsnummer betyder nu vad de säger: uppdateringar inom 1.x förstör inte dina data, dina säkerhetskopior eller dina inställningar.
- Docker Compose-filen och alla instruktioner använder den publicerade avbildningen. En egenbyggd `finstats:latest` kräver bara att avbildningens namn byts; mappen `data` följer med orörd.

## [0.10.2] - 2026-09-20

### Fixed
- På en telefon kunde Inställningar och en series sida rulla i sidled: en etikett som bara var avsedd för skärmläsare rymde ut ur sin tabell, och avsnittslistorna och raden med medverkande gjorde likadant. Nu rullar de inuti sig själva.
- Återblicken rullar inte längre i sidled när ett kapitels bakgrundsord är bredare än sidan.
- En lång rad ”tittar på …” under **Spelas nu** slutar med en ellips i stället för att kapas mitt i en bokstav.
- Affischen framför varje serie på en profil var en andra, namnlös länk för skärmläsare; nu är den dold för dem och titeln bredvid är länken.

## [0.10.1] - 2026-09-20

### Fixed
- Att ladda ner en säkerhetskopia misslyckades i webbläsaren (”the source file could not be read” i Firefox) utan något på sidan som sa varför: FinStats komprimerade en säkerhetskopia, som redan är komprimerad, en gång till, och den dubbelpackade överföringen bröts innan den var klar. Säkerhetskopior skickas nu som de är, med storleken angiven i förväg, så webbläsaren kan visa verkligt förlopp.

## [0.10.0] - 2026-09-20

Säkerhetskopior, och att flytta till en ny installation

### Added
- En säkerhetskopia varje vecka, de fem senaste sparade: varje uppspelning med sin tidslinje av pauser och hopp, markeringar som sett, behörigheter, hemadresser, serverloggen och dina inställningar, i en liten fil, några hundra kB för tusentals uppspelningar. **Inställningar → Säkerhetskopior** listar dem med en **Ladda ner**-knapp för var och en, skapar en på begäran och ställer in hur ofta och hur många. De ligger i mappen `backups` i din datakatalog. En säkerhetskopia innehåller ingen API-nyckel för Jellyfin och ingen inloggningssession, men den är en komplett tittarhistorik med IP-adresser, så förvara den någonstans privat.
- Återställning slår ihop i stället för att skriva över: uppspelningar som redan finns hoppas över, så det är säkert att göra två gånger eller i en instans som har kört ett tag. Avmarkera ”Återställ även inställningar och behörigheter” för att bara få historiken. Säkerhetskopior fungerar mellan versioner åt båda hållen. Från en terminal: `finstats backup` och `finstats restore <file>`.

### Changed
- Insamlingen kontrollerar varje sekund medan någon tittar och var 5:e sekund medan ingen gör det, där den förut kontrollerade var 5:e sekund hela tiden, så pauser, hopp och spårbyten registreras på sekunden. Båda finns under **Inställningar → Insamling**; ett ändrat gammalt intervall behöver ställas in igen.

### Fixed
- En IP-adress i en uppspelnings detaljer bröts över två rader mitt i adressen. Märkningen Lokalt/Fjärr hamnar under adressen när det inte finns plats.

## [0.9.1] - 2026-09-20

### Fixed
- Personer hemma visades som fjärranslutna. En enhet i ditt eget nätverk som når Jellyfin via dess offentliga namn, till exempel via en omvänd proxy eller en domän, kommer in med hushållets offentliga IP, och bara privata adresser räknades som lokala. Nu tar FinStats reda på den offentliga adressen och räknar de uppspelningarna som lokala för hela historiken, och det kommer ihåg tidigare adresser.

### Added
- **Inställningar → Hemnätverk**: adresserna som FinStats behandlar som hemma, där du kan lägga till dina egna (en tidigare adress, ett andra hem, en VPN-utgång) eller stänga av uppslaget. Uppslaget frågar en enkel tjänst av typen ”vad är min IP” varje kvart och provar flera (Amazon, Cloudflare och andra) eftersom reklamblockerande DNS som Pi-hole ofta blockerar dem, och en av dem behöver ingen DNS alls. Förfrågan innehåller inget om dig eller din server, och med brytaren avstängd pratar FinStats inte med något annat än Jellyfin. `FINSTATS_PUBLIC_IP_URL` anger en egen tjänst.

## [0.9.0] - 2026-09-20

Varje tabell, i den ordning du vill

### Added
- Varje tabell går att sortera: klicka på en rubrik för att sortera efter den, igen för att vända ordningen, en tredje gång för den ursprungliga ordningen. Cellernas innebörd förstås, så ”3 d 2 h” sorteras som tid och ”1,4 GB” som storlek, och tomma celler hamnar sist. Det gäller användarlistan, tittare, avsnitt, enheter, tillägg, schemalagda uppgifter, tabellvyn för varje diagram, behörighetsmatrisen och stapellistorna (kodekar, upplösningar, klienter och resten), som får en egen smal rubrikrad.
- Aktivitet och Serverloggen sorteras över alla sina sidor i stället för bara raderna på skärmen, och ordningen sparas i adressen, så den överlever en omladdning eller en delad länk.
- Tabeller med tio rader eller fler får ett litet filterfält ovanför.

## [0.8.2] - 2026-09-20

### Changed
- Återblickens årsväljare är samma brytare som alla andra periodväljare i FinStats (7 d · 30 d · 90 d …).

## [0.8.1] - 2026-09-19

### Added
- Sidor för skådespelare och regissörer, från återblickens ”Mest sedda personer” eller den nya raden **Medverkande** på varje film, serie eller avsnitt: allt de finns med i på din server, hur mycket som har setts och av vem, och vad som fortfarande väntar. Den följer den period du väljer, personer som bara får se sin egen statistik ser bara sitt eget tittande där, och Esc tar dig tillbaka.

## [0.8.0] - 2026-09-19

Återblicken har ett nytt utseende, och mer att säga

### Added
- **Mest sedda personer**: skådespelarna du tillbringade mest tid med och regissörerna bakom det du såg, som en rad porträtt. Medverkande läses tillsammans med biblioteket och visas efter den första läsningen av biblioteket efter den här uppdateringen.
- **Ditt år i dagar**: varje dag under året som en ruta, ljusare ju mer du tittade, med din längsta svit och din största dag ovanför. Hovra över en dag, eller gå igenom kalendern med piltangenterna, för att se vad den innehöll.
- **I siffror**: uppspelningar, tittartid, olika titlar, dagar med tittande, längsta svit och hur mycket som var **omtittningar**, plus hur året fördelar sig mellan avsnitt, filmer och musik.
- Genrekapitlet säger hur många genrer du var inne på och hur stor andel den största tog.

### Changed
- En ny design: den öppnar med ditt år ”i repris”, rubriken bredvid affischerna som fyllde det, över en vågform av året med en stapel per vecka där de starkaste lyser, och du väljer året precis ovanför. Varje kapitel har en rubrik med sitt nyckelord i violett, en mening som bär siffrorna, och kapitlets ord stort och svagt bakom. Den slutar som filmer gör, med eftertexterna: i rollerna, regi, visad på, speltid.
- Timmar, veckodagar och månader delar på ett kapitel **Tittarmönster** med en brytare mellan dem, och rubriken följer det du tittar på (”Fredag tog hem segern”).

### Fixed
- **Spelas nu** på Översikt lämnades tomt när ingen tittade, så rubriken verkade höra till filtren under den. Rutan ”Inget spelas just nu” är tillbaka.

## [0.7.10] - 2026-09-19

### Added
- Bibliotekens bilder: varje bibliotekskort, och bibliotekets egen sida, visar bilden som Jellyfin använder för det, via FinStats bildproxy precis som affischerna. Ett bibliotek utan bild behåller sin ikon.

## [0.7.9] - 2026-09-19

### Changed
- Kortet Genrer öppnas som **lista** igen, med radarn som det andra alternativet på brytaren. Ett befintligt val behålls.

## [0.7.8] - 2026-09-19

### Added
- En radarvy för kortet Genrer, på Översikt och på profiler: en eker per genre, med en brytare mellan **Radar** och **Lista** i kortets hörn, och valet sparas. Att hovra över en punkt, eller använda piltangenterna, visar den genrens tittartid och andel.

## [0.7.7] - 2026-09-19

### Fixed
- Filmer som ingår i en samling visades som ”Inte längre i biblioteket”. Med *group movies into collections* påslaget lämnar Jellyfin ut samlingen i stället för filmerna i den, så FinStats såg dem aldrig; på en server dolde det 556 av 1 238 objekt i filmbiblioteket. Nu ber FinStats om själva filmerna, och läsningen som den här uppdateringen utlöser tar tillbaka dem och kopplar deras uppspelningar till dem igen.

## [0.7.6] - 2026-09-19

### Changed
- På profiler flyttade kortet **Serier** ner till strax ovanför Genrer, så en lång lista med serier inte längre trycker undan tittartiden, aktivitetsdiagrammet och topptitlarna. Dagsviterna står kvar överst, och en serie som du har öppnat förblir öppen när du byter period.

## [0.7.5] - 2026-09-19

### Changed
- Sökningen matchar ord för ord, i valfri ordning, med vad som helst emellan, i stället för att kräva den exakta frasen, så ”alya hides” hittar *Alya Sometimes Hides Her Feelings in Russian*. Versaler, accenter och skiljetecken spelar ingen roll, ett inledande ”The” ignoreras, och en felslagen tangent förlåts (”mentalsit” hittar *The Mentalist*); ord på tre bokstäver eller färre måste fortfarande skrivas rätt. Bättre träffar rankas först, och musik hittas också via artisten.
- Sökrutorna i Aktivitet och Serverloggen matchar ord för ord över alla sina kolumner: ”alya opera” hittar uppspelningar av den serien i Opera.

## [0.7.4] - 2026-09-19

### Changed
- Ändringsloggen grupperas efter serie: varje `0.x`-serie är en hopfällbar grupp med sina versioner, med vad serien handlade om, hur många de är och när. Bara den nyaste är öppen från början, och en stängd grupp visar ändå vilken version du kör.

## [0.7.3] - 2026-09-19

### Fixed
- Klockan i Spelas nu tvekade med några sekunders mellanrum: uppdateringen var 5:e sekund ritade också om den, så när servern låg en bråkdel före flyttade sig siffran för tidigt och stod sedan still i en sekund. Nu är det bara tickandet en gång i sekunden som flyttar den, med exakt en sekund (uppmätt 999 till 1001 ms emellan), och uppdateringen korrigerar fortfarande efter en paus, ett hopp eller verklig avdrift, och landar på tickandet.

## [0.7.2] - 2026-09-19

### Added
- Grupptittande, live: varje kort i Spelas nu säger ”Med maria” medan personer tittar på samma sak tillsammans, där en grupp förut kändes igen först när uppspelningarna hade tagit slut. Live räknas personer som tillsammans när de är på samma titel och antingen startade inom gruppfönstret eller befinner sig på nästan samma position, vilket fungerar även om FinStats startades om mitt i en ström.

## [0.7.1] - 2026-09-19

### Changed
- Spelas nu räknar varje sekund: klockan tickar och stapeln glider emellan, i stället för att hoppa varje gång servern tillfrågas. Servern tillfrågas fortfarande var 5:e sekund och förblir sanningskällan: visningen synkas om vid en paus, ett hopp eller när servern ligger före, men inte för en position som bara är några sekunder gammal, eftersom klienter rapporterar bara ungefär var tionde sekund. Pausade strömmar står still.

## [0.7.0] - 2026-09-19

Vilka som tittar tillsammans

### Added
- Grupptittande: när olika personer startar samma titel samtidigt räknar FinStats det som att de tittar tillsammans. Ett kort **Sett tillsammans** på Översikt visar grupperna, titlarna de delar och deras timmar tillsammans, varje profil visar vem den personen tittar mest med, delade uppspelningar får ett litet personmärke i Aktivitet, och uppspelningens detaljer säger vem den sågs med. Det fungerar på hela din historik, importerade uppspelningar inräknade.
- En inställning för hur nära varandra starterna måste ligga, 60 sekunder som standard: ungefär en tredjedel av verkliga gruppsessioner startar med 6 till 60 sekunders mellanrum. Personerna måste också fortsätta titta sida vid sida i ett par minuter, så två personer som råkar öppna samma avsnitt av en slump blir inte en grupp.

### Changed
- Sökningen öppnas med **Ctrl + Space** i stället för Ctrl/⌘ + K.

## [0.6.1] - 2026-09-19

### Added
- Administratörer kan öppna en annan persons återblick, överst på sidan Återblick eller med **Öppna återblick** på personens profil. Den talar då om personen med namn i stället för ”du”.

### Changed
- Alla andra ser fortfarande bara sin egen återblick, vilka behörigheter de än har, och det finns fortfarande ingen återblick för hela servern.

## [0.6.0] - 2026-09-19

Din profil, och var du är i varje serie

### Added
- Seriestatus på profiler: varje serie du har varit inne på som en stapel med ett segment per avsnitt (sett, påbörjat eller inte än), som kan öppnas säsong för säsong; serierna sorteras efter hur nära du är att se klart, med separata vyer för avslutade serier och för allt.
- **Markera som sedd** på din egen profil, per avsnitt, säsong eller serie, för det du såg medan ingenting registrerades. Jellyfins egna markeringar som spelad används också. Markeringar stannar i FinStats och ändrar aldrig något i Jellyfin; en registrerad uppspelning kan inte avmarkeras, dina egna markeringar kan det.
- Dagsviter på profiler: din längsta följd av dagar i rad med en uppspelning, din nuvarande svit och hur många dagar du har tittat på något.
- **Min profil** i sidopanelen.
- Aktivitet visar var uppspelningen stoppade (till exempel `44:15 / 45:00`) under förloppsstapeln, och datum och tid under ”för 3 tim sedan”. Uppspelningar som importerats från Jellystat har ingen stopposition, eftersom Jellystat aldrig registrerade någon.

### Changed
- Avsnitt som saknas eller inte har sänts än behandlas inte längre som en del av ditt bibliotek. Jellyfin listar dem som platshållare utan fil, och de brukade räknas in i antalet avsnitt.

## [0.5.1] - 2026-09-19

### Added
- En logotyp: en play-knapp skivad i tre diagramstaplar, i sidopanelen och på inloggningsskärmarna, som en skarp SVG med en `favicon.ico` i flera storlekar för webbläsare och bokmärkesfält som vill ha en, plus en ikon för hemskärmen på telefoner och surfplattor.

## [0.5.0] - 2026-09-19

Du bestämmer vem som ser vad

### Added
- Behörigheter under Inställningar → Åtkomst, för alla på en gång eller person för person: **se allas aktivitet** (andras statistik och historik, sidan Användare, varje liveström), **se nätverksdetaljer** (IP-adresser, enhets-id:n, lokalt eller fjärr), **se servern** (sidan Server, serverloggen, misslyckade inloggningar, sökvägar) och **hantera FinStats** (inställningar, uppgifter, import, borttagning av uppspelningar).
- Inloggning per person, för att släppa in några få utan att öppna FinStats för varje användare i Jellyfin.
- Ändringar gäller direkt, även när åtkomst tas bort, och allt upprätthålls av servern i stället för genom att knappar döljs.

### Changed
- Administratörer i Jellyfin har fortfarande allt och är de enda som kan ändra behörigheter; den som får hantera FinStats kan inte bevilja något, varken åt sig själv eller någon annan.
- Årsåterblicken förblir personlig vad som än beviljas.
- Ingenting ändras för befintliga installationer förrän du beviljar något.

## [0.4.5] - 2026-09-19

### Changed
- FinStats är licensierat under **GNU General Public License v3.0**. Ingen tidigare version publicerades någonsin, så ingenting har släppts under andra villkor. Licenserna för de medföljande typsnitten levereras tillsammans med dem.

## [0.4.4] - 2026-09-19

### Added
- Esc går tillbaka: på ett öppnat bibliotek, en användare eller en titel tar den dig tillbaka dit du kom ifrån, med listan exakt som du lämnade den. Det som är öppet stängs först (uppspelningsdetaljer, sökning, en rullgardinsmeny, mobilmenyn), och Esc lämnas i fred medan du skriver eller använder ett diagram med tangentbordet.

## [0.4.3] - 2026-09-19

### Changed
- FinStats kommer ihåg vad varje sida senast visade: att gå tillbaka till en profil, ett bibliotek eller en period som du redan har tittat på ritar upp den direkt, profilbild och allt, medan färska siffror laddas bakom, och sidan ritas om bara om något har ändrats. Det som sparas finns i webbläsarfliken och släpps när du loggar ut eller ändrar något.
- Platshållare för laddning visas bara när laddningen faktiskt är långsam. Förut visade varje sida dem i minst en tredjedels sekund.

### Fixed
- Bibliotek som tagits bort i Jellyfin och aldrig haft någon uppspelning listas inte längre. Ett borttaget bibliotek med historik visas fortfarande, markerat som borttaget.

## [0.4.2] - 2026-09-19

### Fixed
- På sidan Användare låg etiketten **Admin** på en egen rad och gjorde den raden högre än de andra. Nu sitter etiketter bredvid namnet.

## [0.4.1] - 2026-09-19

### Fixed
- Ordet ”null” visas inte längre under översiktens rubrik utanför december och januari, när det inte finns någon återblicksbanner att visa.

## [0.4.0] - 2026-09-19

Se vad som har ändrats utan att lämna appen

### Added
- Fliken **Ändringslogg** som visar varje version, med versionen du kör markerad.
- Versionen i statusfältet länkar dit, och fliken visar en prick efter en uppdatering tills du har tittat.

### Changed
- Att koppla historik till objekt som bytt namn går mycket snabbare: från 18 sekunder till en tredjedels sekund med 666 föräldralösa titlar, så starten förblir omedelbar medan historiken växer.

## [0.3.3] - 2026-09-19

### Changed
- Biblioteket läses om direkt efter att Jellyfins uppgift **Scan Media Library** är klar, aldrig på en separat timer och aldrig medan en genomsökning pågår. FinStats har alltid bara läst från Jellyfin, och det startar aldrig en genomsökning där.
- En veckovis läsning som skyddsnät täcker servrar som förlitar sig på realtidsövervakning, där genomsökningsuppgiften kanske aldrig körs.
- Det gamla intervallet är nu bara en reserv, för när följandet är avstängt eller servern inte rapporterar någon genomsökningsuppgift.

### Added
- Brytaren **Följ Jellyfins biblioteksgenomsökning** under Inställningar → Insamling, påslagen som standard.

## [0.3.2] - 2026-09-19

### Fixed
- Historiken överlever namnbyten. Jellyfin ger en fil som bytt namn ett nytt id, vilket brukade lämna dess tidigare uppspelningar strandade under det råa mappnamnet (`Title (2010) [tmdbid-…] [imdbid-…]`) utan affisch. Nu kopplas de uppspelningarna till den aktuella posten i biblioteket: via id från TMDB/IMDb/TVDB när det gamla namnet innehåller ett, annars via rensad titel och år, och avsnitt via serie plus säsongs- och avsnittsnummer. En matchning måste vara entydig, så titlar som verkligen har tagits bort förblir som de är.
- Live TV-kanaler som importerats från Jellystat räknades som filmer, eftersom Jellystat inte registrerar någon objekttyp. Nu känns de igen som Live TV, och nya uppspelningar av Live TV registreras med Jellyfins egen typ.

### Changed
- Live TV lämnas helt utanför återblicken: topplistor, totaler, persona, rekord och placering.

### Added
- `finstats relink` för att koppla tillbaka historik på begäran. Det körs också vid start och efter varje läsning av biblioteket.

## [0.3.1] - 2026-09-19

### Changed
- Återblicken är strikt personlig: alla, administratörer inräknade, ser bara sin egen, och vyn ”Alla”, användarväljaren och serverkapitlet är borta.
- Återblicken öppnas på det år som är ”klart”: innevarande år under december, annars det föregående. Andra år och ”Senaste 12 månaderna” finns fortfarande ett klick bort.

## [0.3.0] - 2026-09-19

Ditt år i sammanfattning

### Added
- Fliken **Återblick**, en rullande berättelse om ett år: timmar sedda, toppserier, filmer, spår och genrer, och en tittarpersona (nattuggla, morgonmänniska, helgkrigare, maratontittare, filmfantast, musikälskare, vanemänniska) med diagram över tid på dygnet och veckodag.
- Året månad för månad, med titeln som präglade varje månad.
- Rekord: största dagen, största maratonet, längsta svit av dagar, längsta enskilda uppspelning, mest omsedda titel, årets första uppspelning och den äldsta titeln som setts.
- Upptäckter: nya serier som påbörjats, filmer och avsnitt som setts klart, och serierna som fick exakt ett avsnitt.
- En banner på en rad på Översikt i december och januari när en återblick är klar.

## [0.2.0] - 2026-09-19

Mer än Jellystat någonsin registrerade

### Added
- En tidslinje för varje uppspelning: start, paus, återupptagande, hopp, byten av ljud och undertexter, direktuppspelning som övergår i omkodning, och stopp. Plus antal pauser och hopp, var uppspelningen återupptogs från och om tittaren var på det lokala nätverket.
- Fliken **Server**: Jellyfin-version, väntande uppdateringar och omstarter, diskanvändning per bibliotek (Jellyfin 10.11+), tillägg, resultat från schemalagda uppgifter och varje registrerad enhet.
- Insikter om **Uppspelning**: samtidiga strömmar över tid, vilka klienter som tvingar fram omkodning, hur långt folk kommer innan de slutar, lokala mot fjärranslutna uppspelningar och tittarbeteende.
- Vad ditt bibliotek består av: upplösningar, kodekar, dynamiskt omfång, containrar, titlar per decennium, tillskott per månad, de största titlarna och titlarna som ingen någonsin har sett, kontrollerat mot både FinStats historik och Jellyfins egna markeringar som spelad.
- Översikt: flest samtidiga strömmar, uppskattad mängd strömmad data, andel fjärranslutna uppspelningar, genrer efter tittartid och misslyckade inloggningar.
- Objektsidor: länkar till IMDb och TMDB, studior, bitdjup och bildfrekvens, och vem som har titeln markerad som spelad i Jellyfin.
- Användarsidor: genrer, och de filmer, avsnitt och favoriter som Jellyfin har registrerade.

### Fixed
- Tidsetiketten på ett kort för det som spelas nu rinner inte längre ut utanför kortet.
- Långa etiketter på sidan Uppspelning kapades för tidigt.
- En serie som har lämnat biblioteket visar fortfarande sin lista över säsonger och avsnitt.

## [0.1.0] - 2026-09-19

Första versionen

### Added
- Liveinsamling från Jellyfins sessioner: användare, titel, klient, enhet, IP-adress, uppspelningsmetod, orsaker till omkodning och hårdvaruacceleration, uppgifter om video, ljud och undertexter, och tid sedd mot tid pausad.
- Inloggning med ditt Jellyfin-konto. En installation i två steg skapar FinStats egen API-nyckel, och lösenord lagras aldrig. Användare som inte är administratörer kan, om man vill, logga in för att bara se sin egen statistik.
- Sidorna Översikt, Aktivitet, Användare, Bibliotek, Uppspelning och Serverlogg, en kommandopalett (Ctrl/⌘ K) och ett levande statusfält.
- Import från Jellystat: stora säkerhetskopior strömmas till disk och importeras på några sekunder, och att importera samma fil två gånger är säkert.
- En enda liten binärfil med ett inbyggt webbgränssnitt och en SQLite-fil, plus en Docker-avbildning.
