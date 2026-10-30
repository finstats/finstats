# Novità

Tutto ciò che è cambiato in FinStats, prima le versioni più recenti. Questo file viene compilato nel
programma ed è ciò che mostra la scheda **Novità** dell’app.

Formato: `## [version] - date`, oppure `## [version] - first day to release day` per una versione realizzata
in più giorni, poi `### Added`, `### Changed`, `### Security`, `### Performance`, `### Stability`, `### Fixed`
o `### Removed` (Aggiunto, Modificato, Sicurezza, Prestazioni, Stabilità, Corretto, Rimosso) con un punto per ogni modifica.
Sicurezza è qualcosa di cui qualcuno avrebbe potuto abusare, Prestazioni qualcosa che è diventato più rapido o più leggero, Stabilità qualcosa che non può più andare in
crash, bloccarsi, consumare memoria senza restituirla o perdere dati, Corretto qualcosa che dava una risposta
sbagliata. Una versione `x.y.0` ha una breve riga di titolo sotto l’intestazione: quel titolo è ciò che l’app
usa come titolo dell’intera serie.

## [2.3.1] - 2026-10-26 to 2026-10-28

### Security
- **La configurazione chiede un codice monouso**: finché non è configurato, FinStats scrive un codice nel suo log all’avvio (`docker logs finstats`), e la procedura guidata lo chiede prima di provare un indirizzo o configurare qualsiasi cosa. Altrimenti chiunque raggiungesse per primo la porta poteva puntare FinStats a un proprio Jellyfin e diventarne amministratore, o usare la prova per esplorare la rete.
- **L’accesso alle librerie segue Jellyfin**: ognuno vede solo le librerie che Jellyfin gli apre, e in esse solo i titoli che Jellyfin gli elenca secondo il suo limite di classificazione, i tag e le regole sui titoli non classificati, nelle pagine delle librerie, nella ricerca, nel cast, nelle immagini, in Aggiunti di recente, La mia lista, In arrivo, nel feed del calendario e nelle notifiche di nuovi titoli. Chiunque avesse effettuato l’accesso poteva sfogliare ogni libreria.
- Un link poteva mandare qualcuno dalla schermata di benvenuto a un altro sito subito dopo l’accesso. Ora si seguono solo le pagine di FinStats.
- Il registro del server e gli accessi non riusciti negli approfondimenti davano gli indirizzi di accesso e l’attività di tutti a chiunque avesse *Vedere il server*. Le righe di altri richiedono *Vedere l’attività di tutti* e un indirizzo *Vedere i dettagli di rete*, come ovunque.
- Una destinazione di notifica personale poteva raggiungere la rete locale tramite un nome che rispondeva in modo diverso la seconda volta che veniva risolto. Ora la connessione avviene all’indirizzo controllato, altri intervalli che non raggiungono mai internet contano come locali, anche l’indirizzo pubblico di casa viene rifiutato, e un nome che non si riesce a risolvere viene rifiutato con le stesse parole di uno locale.
- Dietro un proxy che aggiunge una propria riga `X-Forwarded-For`, o da una rete IPv6, ogni tentativo di accesso poteva arrivare da un nuovo indirizzo e il limite non si applicava mai. Ora un /48 IPv6 ha 50 tentativi in cinque minuti, un limite pieno rifiuta un nuovo indirizzo invece di dimenticarne uno vecchio, l’intero server si ferma a 60 errori al minuto, e un accesso non riuscito viene registrato nell’audit una volta per indirizzo e finestra.
- `FINSTATS_TRUST_PROXY` indica i proxy (indirizzi o intervalli) di cui si crede a `X-Forwarded-For`, `X-Forwarded-Host` e `X-Forwarded-Proto`, e `1` o `true` ora crede solo a un peer in loopback o su una rete privata. Attivato, qualsiasi peer poteva scegliersi l’indirizzo; disattivato dietro un proxy, dieci tentativi sbagliati chiudevano fuori tutti.
- Una volta letti gli utenti, un accesso con un nome sconosciuto, disattivato, rimosso o non ammesso in FinStats riceve la risposta di una password sbagliata senza chiedere a Jellyfin, e non prima di quando Jellyfin avrebbe risposto. Questi nomi potevano bloccare account Jellyfin, aggirare la regola di Jellyfin sull’accesso remoto o nascondersi dietro l’indirizzo di FinStats.
- Chi Jellyfin limita alla rete di casa (*Consenti connessioni remote* disattivato) accede a FinStats solo da lì, perché Jellyfin valutava l’opzione sull’indirizzo di FinStats, e una lettura degli utenti che non dice se qualcuno è disattivato lo tiene fuori.
- Il cookie di sessione è marcato Secure ogni volta che la pagina di accesso è in https, anche dietro un proxy che non lo dice.
- Un accesso riuscito non perdona più i tentativi precedenti. In https il cookie di sessione è un cookie `__Host-` che un sito su un nome vicino non può sostituire e che non viene accettato con nessun altro nome, e una disconnessione chiude ogni sessione inviata dal browser.
- Solo una chiave del calendario apre il feed del calendario dal suo indirizzo; qualsiasi altra chiave va nell’header.
- Una chiave API non può più concedere permessi, cambiare chi entra o cosa è pubblico, ripristinare un backup, né aggiungere, modificare o provare un servizio o una destinazione di notifica: ognuna di queste cose dura più della chiave, quindi richiede un browser con accesso effettuato.
- Ogni risposta porta `Cross-Origin-Opener-Policy: same-origin` e una `Permissions-Policy`, le risposte dell’API portano `Cross-Origin-Resource-Policy: same-origin` e sono `no-store` salvo diversa indicazione, e la CSP non consente più immagini `blob:`. Una pagina che aveva aperto FinStats poteva poi trasformare la sua scheda in una propria pagina di accesso.
- Dove FinStats conosce lo schema con cui viene raggiunto (il `X-Forwarded-Proto` di un proxy fidato, o l’indirizzo pubblico), una scrittura da un altro schema viene rifiutata come cross-site.
- `/auth/me`, la pagina di un titolo e Salute della libreria rimandano a Jellyfin al suo indirizzo pubblico; l’indirizzo a cui FinStats stesso legge Jellyfin va solo agli amministratori.
- Un’immagine viene presa da Jellyfin solo per qualcosa che FinStats ha, compresi i poster e le schede di un profilo pubblicato, quindi non si possono chiedere per id le immagini di altre librerie.
- Le chiavi di un tracker privato vengono oscurate anche dagli errori della coda di download, in base alla forma oltre che al nome, e a chi ha solo *Vedere cosa si sta scaricando* viene detto quale servizio non ha funzionato e come, mai il suo indirizzo interno né dove reindirizza.
- La pagina Server nomina l’ultimo utente di un dispositivo solo con *Vedere l’attività di tutti*, e Contenuti non collegati lo richiede oltre a *Gestire FinStats*. Senza di esso, la sezione Operazioni conta solo le proprie riproduzioni e i propri stream, una libreria rimossa viene elencata in base alle proprie riproduzioni, e una tabella di collegamento di Tautulli in attesa non si può annullare.
- Le impostazioni mostrano il servizio dell’indirizzo pubblico solo con il suo host, e l’indirizzo di Jellyfin e la cartella dei dati solo agli amministratori, e un titolo che la libreria ha perso viene descritto solo dalle riproduzioni che chi legge può vedere.
- Un backup di Tautulli le cui tabelle non erano tabelle semplici poteva tenere in attesa ogni altra scrittura finché FinStats non veniva riavviato. Un file del genere viene rifiutato, e SQLite legge il backup in modo difensivo.
- Un caricamento oltre 4 GB viene rifiutato, e un backup di Tautulli compresso non viene decompresso oltre la dimensione che dichiara; entrambi potevano riempire il disco su cui si trova il database di FinStats.
- Una risposta da Jellyfin, Sonarr, Radarr, Seerr, dalla ricerca dell’indirizzo pubblico o dal download di GeoIP non viene letta oltre il massimo consentito, contato dopo la decompressione.
- Qualsiasi utente di Jellyfin poteva impedire a FinStats di registrare le riproduzioni di tutti segnalando migliaia di brevi riproduzioni di un titolo. Capire chi ha guardato insieme ora richiede un solo passaggio e non tiene alcun lock mentre lavora.
- Le riproduzioni registrate una dopo l’altra, o un titolo riprodotto a turno da un dispositivo all’altro, non uniscono più estranei in un unico gruppo, e a chi può vedere solo sé stesso vengono indicate solo le persone accanto a cui è andata la sua riproduzione, e per quanto tempo.
- Un link poteva ancora mandare qualcuno dalla schermata di benvenuto a un altro sito dopo l’accesso, tramite un percorso come `/.//`.
- Chi può gestire FinStats non può più cambiare quanti backup conservare o ogni quanto scriverli, la finestra di unione (ora al massimo un’ora) o quando vengono letti utenti, registro attività e backup, e i suoi backup non scalzano mai quelli di un amministratore; gli indirizzi di casa e le regole di viaggio richiedono ciò che richiedono gli avvisi, e importare la cronologia richiede anche *Vedere l’attività di tutti*.
- Una chiave, un token o una password salvati vanno solo al server per cui sono stati salvati, e una destinazione di notifica si crea o si modifica solo con un accesso, mai con una chiave API. Un amministratore può disattivare o rimuovere la destinazione personale di un altro, ma non spostarla, disattivarne il controllo dei certificati o leggerne il topic ntfy o la chiave utente Pushover.
- I webhook di Discord e Slack devono essere `https`, perché il loro indirizzo è il loro token.
- Un’ondata di accessi non riusciti non può più nascondere un accesso reale né sommergere gli avvisi: i nomi che non sono un account producono una sola notifica ogni quarto d’ora che non nomina nessuno, gli avvisi partono per primi, e Slack e Discord mostrano il contenuto di un messaggio come testo e non menzionano nessuno.
- La posizione nel Riepilogo, Capacità, una notifica e un profilo pubblicato non dicono più delle pagine: nessuna posizione né quota del tempo di tutti a chi può vedere solo sé stesso, i momenti di maggior carico solo come giorni di tre stream o più senza *Vedere l’attività di tutti*, nessun job o connessione falliti a chi non può gestire FinStats, e un profilo pubblico cambia una volta al giorno.
- Un backup di Tautulli con una colonna generata viene rifiutato, e l’immagine di una riproduzione importata viene presa solo quando la libreria ha il suo titolo.
- Un ripristino non apre mai FinStats più di quanto sia: le impostazioni tornano solo se richiesto, accesso, permessi e profili pubblici diventano ciò che consentono sia questa installazione sia il backup, e gli indirizzi pubblici restano quelli di questa installazione. `finstats restore <file>` riporta solo la cronologia, come la pagina, e anche le impostazioni con `--with-settings`.
- Un ripristino della sola cronologia non riporta più indirizzi di casa, avvisi silenziati, voci di audit che si spacciano per quelle di questa installazione o titoli localizzati che la libreria non ha, e nessun ripristino riporta riproduzioni, righe di log o dispositivi datati prima del 1990 o dopo domani. Una riproduzione ripristinata conta come importata qualunque sorgente dichiari, quindi un riavvio non la prosegue mai.
- La cartella dei dati appartiene solo all’utente di FinStats (0700, i suoi file 0600, `umask 077` nell’immagine). FinStats si rifiuta di girare come root su una cartella dei dati di proprietà di un altro utente e indica il `docker exec -u` da usare, e l’entrypoint cambia il proprietario solo della cartella e delle voci proprie di FinStats.
- Il nome di una destinazione non può contenere caratteri di controllo, e il nome di un dispositivo, di una persona o di un titolo non può più falsificare una riga nel log.
- Il parametro di percorso di un link non può più raggiungere un percorso dell’API diverso da quello della sua pagina, e il costruttore di elementi di FinUI imposta un indirizzo solo se è http(s) o relativo, mai `srcdoc`.
- La ricerca, i profili pubblicati e il lavoro avviato a mano non possono più tenere occupato FinStats: una ricerca si ferma a 200 caratteri, un profilo è conservato per lingua, le schede attendono in una breve coda, e un job, la ricerca dell’indirizzo, il download del database di localizzazione e un messaggio di prova si avviano solo a intervalli, comunque vengano richiesti, con i messaggi di prova contati per mittente. Scaricare il database di localizzazione richiede anche ciò che richiede leggere la pagina Sicurezza.
- Il corpo di una richiesta deve continuare ad arrivare una volta passati i primi 15 secondi, una risposta che nessuno legge viene abbandonata dopo 30 secondi e un indirizzo tiene al massimo 64 connessioni, a parte un proxy fidato. Qualche centinaio di richieste che non inviavano mai il loro corpo poteva impedire a FinStats di rispondere a chiunque.
- Un server di posta non viene letto oltre quanto può contenere una risposta (1 MB per un'intera lettera), e un messaggio di prova si ferma dopo il tempo che può impiegare uno in coda. Un server che non terminava mai una riga poteva far trattenere a FinStats sempre più memoria finché non veniva fermato.
- La lettura di una libreria si giudica sui titoli che le letture hanno salvato, non su quelli portati da un'importazione, e un titolo Jellystat in una libreria già letta entra come rimosso. Un'importazione di titoli inventati poteva far fermare FinStats a ogni lettura della libreria, a ogni avvio.
- Il limite di 60 accessi falliti al minuto su tutto il server conta solo i tentativi inviati a Jellyfin, compresi quelli ancora in attesa di risposta. I nomi che FinStats rifiutava da sé permettevano a chiunque con pochi indirizzi di impedire a tutti di accedere, e molti tentativi inviati insieme arrivavano tutti a Jellyfin.
- Chi Jellyfin limita alla rete di casa resta limitato a ogni richiesta, sessione e chiavi comprese, non solo all'accesso; casa è una rete privata o l'indirizzo pubblico della famiglia trovato dalla ricerca, mai un indirizzo inserito come casa, e dietro un proxy che `FINSTATS_TRUST_PROXY` non nomina resta fuori. Un cookie o una chiave portati fuori casa aprivano tutto da ovunque, e un gestore poteva aprire la casa a un indirizzo a sua scelta.
- Una chiave API non può più disattivare ciò che noterebbe un suo abuso, né i backup che annullerebbero ciò che ha fatto: quanti backup si tengono e ogni quanto, la finestra di unione, gli indirizzi di casa e le regole di viaggio, le pianificazioni di utenti, registro attività e backup, silenziare o risolvere tutti gli avvisi, un backup nel cestino o la rimozione di una destinazione di notifica. Ognuno sopravviveva alla chiave una volta revocata.
- Le locandine di Sonarr e Radarr, e il prossimo episodio di una lista dei desideri, seguono anch'essi l'accesso alle librerie: una serie che Jellyfin nasconde a qualcuno non ha per lui né locandina né data. Contare gli id dei servizi rivelava quali serie nascoste stavano arrivando, e una lista nominava il prossimo episodio di una serie nascosta.
- Il capitolo Insieme del riepilogo nomina solo le persone la cui visione è andata accanto alla tua, come fa la pagina Insieme. Un gruppo raggiunto tramite qualcun altro nominava persone con cui non avevi mai guardato.
- Un'importazione da Jellystat o Streamystats legge l'intero file prima di prendere il blocco di scrittura del database, poi scrive solo le righe che tiene. Letto sotto il blocco, un file di poche righe vere e gigabyte di qualunque altra cosa teneva in attesa ogni altra scrittura, riproduzioni e accessi compresi.
- Un ripristino tiene un permesso per persona, per non più persone di una famiglia molte volte, e controlla i doppioni di una voce di audit senza rileggere ogni voce appena scritta. Un backup costruito ad arte poteva trattenere memoria e il blocco di scrittura del database sempre più a lungo durante il ripristino.
- Un ripristino con le impostazioni attiva la ricerca dell'indirizzo pubblico e il download del database delle posizioni solo dove anche questa installazione li ha attivi, e non tiene mai meno backup né li scrive meno spesso di questa installazione. Un backup che ne teneva uno poteva far rimuovere tutti gli altri al giro successivo.
- Una dimensione di file che nessun disco potrebbe contenere viene letta come nessuna dimensione, e le dimensioni si sommano in modo che nessun totale possa traboccare. Una sola dimensione importata faceva rispondere le pagine delle librerie con un errore per sempre.
- Quando FinStats sa di essere raggiunto via https (il suo indirizzo pubblico è https, o un proxy fidato lo dice), una sessione si prende solo dal suo cookie `__Host-`, e con un indirizzo pubblico https ogni nuova sessione è una sessione https. Un accesso inviato senza `Origin` tramite un proxy che non diceva nulla creava una sessione semplice che un sito vicino poteva piazzare nel browser di qualcuno.
- L'immagine viene costruita con ogni azione GitHub fissata a un commit e le due immagini di base a un digest, spostati solo da Dependabot, e una versione si costruisce senza la cache di build condivisa. Un tag spostato sull'azione di qualcun altro avrebbe potuto pubblicare qualsiasi immagine come FinStats.

### Performance
- Un profilo pubblicato viene calcolato al massimo una volta al minuto, non di nuovo per ogni immagine e scheda che contiene.
- La cronologia di una persona viene analizzata per gli avvisi di sicurezza una volta alla volta, per quante riproduzioni inizino.
- Accedere o disconnettersi, revocare una chiave o cambiare i permessi di una persona chiede di ricontrollare solo alle connessioni in tempo reale che riguarda.
- La tabella di collegamento di Tautulli legge la cronologia una volta, non una volta per ogni utente Plex.
- Un’immagine che manca dalla cache viene presa da una sola richiesta mentre le altre attendono, e una che la sorgente non è riuscita a fornire non viene richiesta di nuovo per 30 secondi.
- I file dell’interfaccia web vengono compressi con gzip una volta e conservati, e i font non vengono compressi di nuovo.

### Stability
- Un backup nel vecchio formato di Jellystat viene letto una riga alla volta anziché per intero, e una riga più lunga di quelle di qualsiasi backup viene rifiutata.
- Due caricamenti di Tautulli contemporanei non scrivono più nello stesso file.
- Un numero di pagina molto oltre la fine dei titoli di una libreria o di Salute della libreria dà una pagina vuota, non un errore.
- Un’importazione o un ripristino il cui caricamento si è interrotto non fa più rifiutare tutti gli altri fino a un riavvio, e né esso né un’importazione abbandonata prima di iniziare lascia un file dietro di sé.
- Una riga di un backup non viene letta oltre 8 MB, e arriva un backup di Tautulli alla volta.
- Una riproduzione conserva al massimo 2.000 eventi, e un’importazione salta una riproduzione che non inizia e poi finisce tra il 1990 e domani.
- Un solo numero di episodio molto sotto lo zero non blocca più ogni lettura della libreria: Salute della libreria (anche il suo riepilogo), Capacità, il ricollegamento e la localizzazione sommano e confrontano i numeri senza overflow, e una build di release controlla l’overflow, quindi qualunque cosa sfugga ferma un’operazione invece di girare in loop.
- Un numero di stagione, di episodio o di fine viene conservato solo dove Jellyfin potrebbe contenerlo (da 0 a 2.147.483.647), che venga da Jellyfin, da Jellystat o da Tautulli.
- Un backup di Tautulli con id ripetuti viene rifiutato, perché pochi MB di righe sotto un unico id esaurivano la memoria del processo, e un backup viene aperto come immutabile, senza lasciare `-wal` o `-shm` dietro di sé.
- Una connessione deve inviare la sua richiesta entro 15 secondi e se ne servono al massimo 512 alla volta: qualche centinaio di socket semiaperti mandava FinStats offline. Un frame del canale in tempo reale è limitato a 4 KB.
- Pochi kilobyte da Sonarr, Radarr o Seerr non possono più diventare gigabyte di memoria: una lista non conserva più di quanto FinStats ne legge.
- Una destinazione di notifica che non risponde mai rallenta solo sé stessa: le destinazioni vengono servite in parallelo, e una che va continuamente in timeout viene lasciata stare per dieci minuti.
- Il tempo rimanente e la velocità totale di una coda, i numeri di una riproduzione ripristinata e il giorno in cui una riproduzione esce dal cestino non possono più andare in overflow.
- Un ripristino legge l’intero file prima di prendere il lock di scrittura, rifiuta un file che si decomprime in molto più della sua dimensione, e scrive a blocchi, così il collector continua a registrare nel frattempo.
- I backup vengono nominati in UTC e non sovrascrivono mai un file esistente, e vengono eliminate solo le copie pre-aggiornamento di FinStats stesso.
- Una riga del registro del server datata nel futuro non impedisce più la lettura del registro attività: viene letto dalla sua riga più recente che non sia nel futuro.
- Le richieste che arrivano insieme su un link pubblicato calcolano la risposta una sola volta, e le sue schede vengono disegnate in una corsia a parte, così chi ha il link non può più far rispondere 503 alle schede del Riepilogo del proprietario.
- Le riproduzioni di una libreria che manca dall'elenco delle librerie di Jellyfin aspettano due giorni prima di passare alla copia dello stesso titolo in un'altra libreria. Un solo elenco che tralasciava una libreria le spostava subito, e restavano spostate quando la libreria tornava.

### Fixed
- Gli accessi non riusciti su un account sono un solo avvertimento per finestra, che diventa un avviso a venti, e un nome conta contro un account solo se scritto per intero (“x alice” non conta più contro alice).
- I messaggi di Discord e Slack vengono prima sottoposti a escape e poi tagliati: un nome di dispositivo pieno di Markdown faceva rifiutare a Discord ogni avviso che lo nominava.
- Un indirizzo IPv6 pubblico viene accettato per una destinazione personale.
- Dietro un proxy che fa accedere le persone da sé (`auth_basic` di nginx), il cookie di FinStats non viene più rifiutato.
- Un nome visualizzato che contiene `{{url}}` viene mostrato così come è scritto nell’anteprima di una pagina pubblicata.
- Un riepilogo chiesto per l'anno 9999 mostra l'anno predefinito invece di un errore.
- Una pagina dal vivo il cui permesso è stato tolto e ridato mentre era aperta torna ad aggiornarsi, senza ricaricarla.

### Removed
- Il **Synnmørsk** non è più una delle lingue di FinStats: un browser che l'aveva scelto legge il nynorsk, la lingua scritta da cui era tratto.

## [2.3.0] - 2026-10-07 to 2026-10-26

Capacità, un canale in tempo reale, modalità semplice e lingue

### Added
- **Lingue**: FinStats parla 23 lingue. L’inglese britannico e americano e il norvegese bokmål e nynorsk sono scritti e verificati dal manutentore; danese, tedesco, spagnolo, finlandese, francese, hindi, islandese, italiano, giapponese, coreano, olandese, polacco, portoghese (Brasile), russo, svedese, turco, ucraino e cinese (semplificato e tradizionale) sono traduzioni che nessuno qui ha verificato, e dicono in inglese ciò che ancora non hanno. Ogni pagina di entrambi i layout, il profilo pubblico, le Novità e ciò che il server risponde a parole (un errore, le schede del Riepilogo, cosa fa un’operazione di Jellyfin) sono nella lingua scelta in Impostazioni → Aspetto → Lingua, per browser; finché non se ne sceglie una, decide la lingua del browser. Numeri, date, unità ed elenchi sono scritti come li scrive quella lingua. Uno script può chiederne una con `Accept-Language`.
- **Modalità semplice**: un secondo layout per chi arriva da Plex, disposto come un classico server di statistiche per Plex. Una barra in alto al posto del menu, e solo le pagine di statistiche: Home, Librerie, Utenti, Cronologia, Grafici e Impostazioni, con un ingranaggio per il tuo profilo, le Impostazioni, le Novità, il tema e l’uscita. Si sceglie per browser in Impostazioni → Aspetto → Layout. Carica solo la sua metà di FinStats, quindi si avvia più in fretta; Riepilogo, Insieme, In arrivo, Sicurezza e le pagine del server restano nel layout di FinStats. Usa i colori di FinStats, chiari e scuri, e in essa non si muove nulla.
- **La Home della modalità semplice**: cosa è in riproduzione, con una scheda per stream (come viene riprodotto, da casa o da fuori, quando finisce), le statistiche di visione come schede in classifica per numero di riproduzioni o durata negli ultimi N giorni, cosa contiene ogni libreria e cosa è arrivato di recente.
- **I Grafici della modalità semplice**: riproduzioni per tipo di contenuto (giorno per giorno, per giorno della settimana e ora, le dieci app e persone principali), per modalità di riproduzione (giorno per giorno, il massimo di stream contemporanei di ogni giorno, la risoluzione del file e quella inviata, app e persone) e mese per mese, per numero di riproduzioni o durata, per tutti o per le persone scelte.
- **Librerie, Utenti e titoli nella modalità semplice**: tabelle di ogni libreria e persona con l’ultima cosa riprodotta, su cosa e da dove (l’indirizzo solo con *Vedere i dettagli di rete*), e una pagina per ogni libreria, persona e titolo con i suoi numeri e la sua cronologia.
- Per gli script: `/api/stats/split` (riproduzioni per app, persona, risoluzione, giorno o mese, divise per tipo di contenuto o per modalità di riproduzione); classifiche per numero di spettatori distinti (`sort=users`); attività ristretta alle riproduzioni viste (`watched=1`); approfondimenti solo per le parti richieste (`parts=`); il picco diviso in transcodifiche, streaming diretti e riproduzioni dirette, anche giorno per giorno; cosa contiene ogni libreria e la sua ultima riproduzione; aggiunti di recente per tipo (`kind=`); la mappa di calore per tipo di contenuto; l’ultima riproduzione di una persona; se uno stream viene riprodotto da casa.
- **Un canale in tempo reale**: la panoramica, i download di In arrivo e la barra di stato si aggiornano da soli tramite un WebSocket per scheda del browser (`/api/live`), così ciò che è in riproduzione e ciò che si sta scaricando cambia sullo schermo nel momento in cui cambia, anziché al controllo successivo. Dietro un reverse proxy che non inoltra gli upgrade WebSocket, o su una connessione caduta senza preavviso, le pagine chiedono ogni pochi secondi come prima. `/api/now-playing`, `/api/downloads` e `/api/summary` restano come sono, per gli script.
- **Server → Capacità**: questo server ce la farà, in base a ciò che FinStats ha visto? I momenti di maggior carico mai registrati e quelli degli ultimi 90 giorni: gli stream contemporanei, e quanti hanno avuto l’immagine transcodificata con accelerazione hardware, senza, o senza che fosse detto (una cronologia importata da Tautulli è quella di Plex, e resta esclusa). Ogni disco su cui si trovano le librerie e quanto in fretta lo riempie ciò che Jellyfin ha aggiunto in sei mesi (un disco riempito più di recente viene misurato a partire da un mese dopo il suo primo file, perché il primo mese è quello in cui è stato riempito) e, dove Jellyfin 10.11 o successivo riporta lo spazio libero, quanto durerà a quel ritmo. FinStats non dice mai quanti stream potrebbe reggere il server.
- **Esporta .ics** in In arrivo → Prossime uscite: l’elenco così come l’hai filtrato, scaricato come file di calendario da importare una volta. Non si aggiorna da solo; una chiave calendario da Impostazioni → Chiavi API continua a dare un calendario che si aggiorna.
- **Da tenere d’occhio**: Capacità segnala quando ogni transcodifica riportata da Jellyfin in 90 giorni è avvenuta senza accelerazione hardware mentre le transcodifiche sono frequenti, quando l’analisi della libreria di Jellyfin è in corso per un quarto del tempo o più, e quando a un disco restano meno di sei mesi di spazio. Ognuna arriva con i suoi numeri e si può mettere da parte come le segnalazioni di Salute della libreria.
- **Di cosa è fatta la tua libreria** si può restringere a una libreria o a più di una, in Librerie e nella pagina di ogni libreria (`library_id=a,b`).
- **Formati di data e ora** in Impostazioni → Aspetto: una data come 31.01.2026, 31/01/2026, 31-01-2026, 01/31/2026, 2026-01-31, 2026.01.31 o 2026/01/31, e un’ora a 24 o 12 ore, con o senza zero davanti, o ciascuna come la scrive la lingua. Conservati da questo browser.
- **Un benvenuto**: la prima volta che qualcuno accede, da qualunque browser, e alla fine della configurazione, FinStats chiede la lingua, i formati di data e ora e il layout, con un’immagine di ciascuno. Salta tiene inglese britannico, 31/01/2026, 13:05 e FinStats.

### Changed
- Il logo dentro una pagina è il solo marchio accanto al nome, composto in Inter qualunque carattere scelga un preset; dove sta da solo (la favicon) mantiene il suo riquadro arrotondato.
- Una riproduzione transcodificata via software viene registrata come tale quando Jellyfin lo dice; prima sembrava uguale a quella di un server che non lo dice. Le pagine chiamano ancora entrambe Software.
- Un aggiornamento ora esegue, una sola volta e a partire dalla versione più vecchia, ciò che una versione deve fare su un database scritto da un FinStats precedente, accanto alle sue migrazioni. Quello della 2.3.0 legge i dettagli del server al primo avvio, così Capacità è subito disponibile.

### Performance
- **Una pagina lasciata aperta non chiede nulla**: la panoramica faceva 30 richieste al minuto per restare aggiornata e i download di In arrivo 18; con il canale in tempo reale non ne fanno nessuna. Per quante schede siano aperte, FinStats non chiede a Jellyfin, Sonarr e Radarr nulla più di prima, e una scheda nascosta lascia ancora rallentare la coda dei download.
- La pagina di una persona legge solo le riproduzioni di quella persona: i suoi totali e la sua ultima riproduzione venivano calcolati su tutte le riproduzioni del server e poi ristretti a lei, circa quattro secondi su un milione di riproduzioni.
- La pagina di una libreria legge solo i titoli e le riproduzioni di quella libreria, mentre prima contava quelli di tutte e ne teneva una: un terzo del tempo su un milione di riproduzioni.
- Con l’orologio di Jellyfin avanti rispetto a quello di FinStats, *Dopo l’analisi della libreria di Jellyfin* rileggeva tutta la libreria ogni minuto finché la differenza non era trascorsa; ora ogni analisi viene seguita una volta sola.
- Un file GeoIP che non si poteva aprire veniva copiato di nuovo, cento megabyte e oltre, e ogni indirizzo veniva ricollocato, ogni quarto d’ora; ora si riprova solo quando il file cambia.
- Salute della libreria si apre subito dopo che il puntatore si è fermato sul suo link: ciò che era stato caricato in anticipo veniva conservato dove la pagina non guardava mai.

### Stability
- Un aggiornamento fallito dopo l’esecuzione di una migrazione, e subito riavviato da Docker, faceva a ogni avvio una nuova copia del database aggiornato a metà, e tre avvii eliminavano l’unica copia che la versione precedente poteva aprire. Ora la copia di prima dell’aggiornamento viene conservata.
- Un titolo eliminato in Jellyfin mentre si leggeva la libreria (un file sostituito da Radarr) poteva far saltare alla lettura un altro titolo ancora presente e farlo segnare come rimosso, e le sue riproduzioni potevano poi essere spostate su un’altra sua copia. Ora ciò che una lettura non ha visto viene prima cercato per id.
- Ripristinare un backup fatto prima che un file venisse rinominato o aggiornato ripristinava una seconda volta ogni riproduzione spostata nel frattempo sul nuovo file, raddoppiandola per sempre.
- Una richiesta nella risposta di Seerr che FinStats non riusciva a leggere (un campo rinominato in un aggiornamento di Seerr) veniva segnata come rimossa dalla lettura completa giornaliera.
- Un’operazione che incontrava un bug restava “in esecuzione” per sempre, e nessuna attivazione la faceva ripartire finché FinStats non veniva riavviato; ora termina come non riuscita.
- Uno zip di Tautulli danneggiato interrompeva il caricamento prima di rimuovere il file, che contiene i token di ogni utente Plex, fino alla pulizia delle sei ore.
- Un’importazione o un ripristino interrotto mentre il file arrivava lasciava quel file nella cartella dei dati per sempre, indirizzi compresi; ora viene rimosso all’avvio successivo.
- Un’immagine poteva essere servita scritta a metà mentre veniva messa in cache, e il browser poi la teneva così.
- Le chiavi API revocate venivano conservate per sempre, per quante se ne creassero e revocassero.
- Una lettura dei segni di visto e preferito di qualcuno tornata vuota o illeggibile li cancellava tutti fino alla lettura successiva; ora viene rifiutata, come una lettura vuota di una libreria.
- Un episodio con un numero finale che nessun episodio può avere (una data scritta lì) poteva tenere occupata senza fine la Salute della libreria dopo una lettura della libreria.

### Fixed
- Una nota con del codice dentro il grassetto (quelle della 2.0.2 e della 2.0.3) mostrava i backtick invece di disegnare il codice.
- Un grafico di conteggi poteva numerare il suo asse 0, 3, 5, 8, 10, o persino 0, 0, 1, 1, 1: passi di 2,5 scritti arrotondati. Ora i conteggi salgono a passi interi.
- Una chiave calendario creata da qualcuno che non è un gestore arrivava con un link sull’indirizzo del browser invece che su quello pubblico di FinStats, quindi funzionava solo dall’interno; la pagina chiedeva anche impostazioni che le venivano rifiutate.
- L’ultima riproduzione di una persona poteva essere di pochi secondi troppo breve per contare come riproduzione altrove, e nominare così un titolo che la sua Cronologia non mostra.
- Un ripristino da Eliminate di recente non riuscito non diceva nulla e lasciava girare il pulsante; lo stesso valeva per un *Ripristina* non riuscito in Salute della libreria e per un *Riapri* o un *Risolvi tutti* non riuscito in Sicurezza.
- In un’app di calendario il feed del calendario si chiama FinStats, non finstats.
- La coda dei download mostrava chi aveva chiesto ogni download a chiunque potesse vederla; ora per la richiesta di un altro serve *Vedere l’attività di tutti*, come in In arrivo.
- Togliere *Ricevere notifiche* non fermava ciò che veniva inviato alle destinazioni di quella persona, che poi non poteva più disattivarle.
- Una riproduzione messa in pausa e ripresa entro la finestra di unione veniva annunciata come interrotta una volta, e la ripresa e la fine vera mai.
- Le riproduzioni di un brano potevano essere spostate sul brano omonimo di un altro artista (“Intro”) quando il suo album lasciava la libreria.
- Una destinazione di notifica scritta come indirizzo IPv4 dentro un IPv6 (NAT64, 6to4) passava per pubblica mentre raggiungeva la rete locale.
- Una connessione Sonarr o Radarr puntata a un altro server continuava a mostrare per una settimana le locandine del server precedente.
- Gli avvisi di Sicurezza dicevano che non c’era nulla da controllare quando erano scelte due o più persone, e una pagina rimasta vuota dopo aver risolto il suo ultimo avviso non aveva modo di tornare indietro.
- Ogni chiave API non ancora scaduta diceva di essere scaduta “proprio ora”.
- Una finestra di dialogo restava sopra la pagina dopo Indietro, e la pagina non scorreva finché non veniva chiusa.
- Il registro di audit mostrava otto tipi di voce (una riproduzione o un backup ripristinati, il cestino svuotato, i profili pubblici, un aspetto cambiato) con i loro id, senza nulla in Dettaglio.
- Aggiungere un titolo alla lista dal suo menu contestuale falliva senza dire nulla quando il server rifiutava.
- *Visti insieme* nella panoramica trattava più persone scelte insieme come una sola.
- Il riepilogo scriveva ore e giorni sotto il dieci con la virgola decimale del browser, accanto alle virgole delle migliaia all’inglese che usava per il resto.
- Capacità diceva che un disco si sarebbe riempito "al ritmo degli ultimi 0 mesi".
- Nella pagina Sicurezza, gli indirizzi senza luogo contavano le riproduzioni di tutti da sempre, qualunque fossero la persona e il periodo a cui era ristretta la pagina.
- Chi aveva *Gestire FinStats* ma non *Vedere l’attività di tutti* poteva mettere nel cestino la riproduzione di chiunque.
- Una scheda del Riepilogo o di un profilo pubblico poteva arrivare dalla cache del browser nella lingua scelta prima.
- I download inviati dal canale in diretta erano in inglese britannico, qualunque fosse la lingua della pagina.
- Un accesso che cambiava ciò che qualcuno può vedere raggiungeva le sue altre schede aperte solo al controllo successivo, ogni mezzo minuto.
- Una scheda che cedeva la sua connessione in diretta a una più recente ne riprendeva una alla pagina successiva, così le schede di una persona continuavano a chiudersi a vicenda.
- Le tabelle ordinavano come testo le dimensioni da 1000 a 1023 unità nelle lingue che raggruppano le cifre con uno spazio, e così le percentuali in turco.
- Un titolo con un segnaposto nel nome, come "Big {device} Bunny", veniva compilato dai messaggi del server.
- Le tabelle ordinavano come testo le durate e i conteggi su un totale (5 di 26) in ogni lingua tranne l’inglese e quelle nordiche.
- Una scheda ancora aperta in una lingua sentiva il server in un’altra non appena un’altra scheda l’aveva scelta.
- Una parola accanto a un numero prendeva la forma sbagliata in polacco, russo e ucraino ("5 wieczory", "2 потоков").
- Capacità contava come crescita il primo riempimento di una libreria e divideva quanto aggiunto da ogni libreria per sei mesi, qualunque fosse il periodo misurato; un file che prendeva il posto di un altro (un aggiornamento da Radarr o Sonarr, o un file a cui Jellyfin dava un nuovo id) contava come tutto nuovo, quando nuova era solo la differenza.
- Chi guardava insieme con SyncPlay non era un gruppo quando uno si univa minuti dopo l’altro; le riproduzioni che finiscono insieme nello stesso punto ora stanno insieme, e la cronologia viene raggruppata di nuovo una volta.
- Qualcuno senza *Vedere l’attività di tutti* poteva capire da *Mai visti* di una libreria che un’altra persona aveva riprodotto un titolo; ora conta solo la sua visione.

## [2.2.0] - 2026-10-01 to 2026-10-07

Salute della libreria e FinUI

### Added
- **Un menu tutto tuo sul computer**: Impostazioni → Aspetto ne offre otto: la barra laterale com’era, una raggruppata per scopo delle pagine, una colonna di icone che si apre nella barra laterale sotto il puntatore, una barra laterale che comincia dalla ricerca, un dock, una barra comandi, pagine fissate in alto e riquadri colorati. Le barre laterali possono stare a destra. Il dock scivola via mentre scorri verso il basso e torna quando scorri verso l’alto o punti al fondo della finestra; con il dock la barra di stato passa in alto.
- **Un menu tutto tuo sul telefono**: una barra di schede con Altro (il nuovo predefinito), un pannello che si apre a metà, lo schermo intero, le pagine a ventaglio intorno al pollice, o una barra degli indirizzi che diventa il menu. Ognuno cerca tra le sue pagine e passa ogni altra parola alla ricerca.
- Entrambi sono conservati dal browser in cui sono stati scelti, così telefono e computer tengono ciascuno il proprio; entrambi sono componenti di FinUI, `desktop-nav` e `mobile-nav`.
- **Un menu contestuale**: fai clic destro su un titolo, una persona o una libreria ovunque in FinStats (una locandina nella panoramica, una riga di un elenco, un nome in una tabella, un film in In arrivo che non è ancora nella libreria), oppure tienici sopra il dito, o premi il tasto menu, e un menu nasce dal punto premuto: aprilo, aprilo in una nuova scheda, aggiungi un film o una serie alla tua lista, aprilo in Jellyfin o su TMDB e TVDB, copia il suo link. Maiusc + clic destro apre ancora il menu del browser. È il `context-menu` di FinUI.
- **Ricerca senza finestra di dialogo**: la ricerca nasce dal tuo menu (una barra laterale si allarga fino a diventarla, il dock si solleva fino a diventarla, il nome della barra comandi si trasforma in essa, il menu del telefono la contiene) oppure, se lo scegli in Impostazioni → Aspetto, prende il posto della pagina mentre il menu resta. Ctrl+Spazio la apre come prima; la finestra sopra tutto non c’è più. È la `search` di FinUI.
- **Tutto il contenuto di una libreria**: la scheda Aggiunti di recente di una libreria offre *Tutto il contenuto*, ogni film, serie o album che contiene, come locandine, per nome, anno, data di aggiunta o dimensione, filtrati mentre scrivi. Altri arrivano mentre scorri.
- **Server → Salute della libreria**: FinStats confronta ogni file con i suoi vicini ed elenca ciò che spicca, ciascuno con i numeri che lo giustificano. Si limita a segnalare; in Jellyfin non viene cambiato, eliminato o rianalizzato nulla.
- **Buchi**: episodi mancanti tra due file di una stagione (“Stagione 2: mancano gli episodi 4 e 5 tra il 3 e il 6”). Un file che contiene due episodi conta per entrambi; gli speciali e la fine di una stagione non vengono considerati.
- **Stagioni diverse** dal resto della loro serie per risoluzione, HDR o codec, ed **episodi anomali**: un file diverso dalla sua stagione, come un episodio SD in una stagione 1080p. La risoluzione si basa sulla larghezza o sull’altezza, quindi un film ritagliato a 1920×800 è 1080p.
- **Copie e versioni**: lo stesso film o episodio due volte, secondo gli id dei provider che già usano la lista e In arrivo. Due copie nella stessa risoluzione mostrano lo spazio occupato due volte; una copia 4K accanto a una 1080p è elencata come due versioni e mai contata come spreco.
- **File scarni**: un bitrate molto più basso di quello che di solito richiedono la sua risoluzione e il suo codec (“4K HEVC a 1,1 Mbps”); gli episodi scarni di una stagione sono un’unica segnalazione.
- **Doppiaggi interrotti**: una lingua audio che alcune stagioni hanno in ogni episodio e altre in nessuno.
- **Mai identificati**: film e serie che Jellyfin ha abbinato senza alcun id TMDB, TVDB o IMDb.
- **Ignora** una segnalazione che va bene così di proposito, con una nota. Ricompare da sola quando i suoi file cambiano. Può ignorarle chi può gestire FinStats e vedere il server; le segnalazioni ignorate sono nel registro di audit e fanno parte dei backup.
- La pagina di una libreria dice quante cose ci sono da controllare lì, e porta a esse.
- **FinUI**, i componenti con cui è costruito FinStats, è un progetto a sé (`github.com/finstats/finui`): ogni pulsante, scheda e controllo in ogni stato e in entrambi i temi su `finui.finstats.no`, e **FinUI create** per cambiarne colori, angoli, densità e altro mentre guardi, e portartelo a casa con un solo comando. **Impostazioni → Sistema** porta lì.
- **Impostazioni → Aspetto**: ognuno sceglie come gli appare FinStats, con il codice di un preset di FinUI: colore di base, accento, colori dei grafici, contrasto, angoli, densità, bordi, schede, pulsanti, campi, tabelle, evidenziazione, movimento, icone, il menu, la pagina dietro a tutto, l’anello di focus, come sono composte le intestazioni, e i caratteri per testo, titoli e numeri. La scelta è solo tua; mostra cosa significa il codice e porta a FinUI create per cambiarlo.
- **Stili**: undici aspetti completi da cui partire con una sola scelta in FinUI create, il cui codice viene accettato da Impostazioni → Aspetto. Sono Washi (FinStats così come viene distribuito), Studio, Noir, Matinee, Console, Pastille, Ledger, Gazette, Arcade, Canopy e Harbour. FinStats stesso ha un solo aspetto e non offre un elenco di stili.
- **Icone che si muovono**: ogni icona in un link, un pulsante o una scheda fa il suo movimento una volta quando la si punta (quella di aggiornamento gira, la freccia di download cade nel suo vassoio, il cestino solleva il coperchio), e un pulsante di aggiornamento premuto continua a girare finché non arriva la risposta. Ferme con il movimento ridotto, e con Movimento: Disattivato.
- **Movimento, con FinMotion**: ora i componenti di FinUI si muovono: la manopola di un interruttore viene lanciata dall’altra parte, la sottolineatura di una scheda avanza fino alla scheda successiva, un campo rifiutato scuote la testa, una copia viene spuntata con un tratto solo, gli avvisi si impilano come una mano di carte e si aprono a ventaglio quando li punti, il pannello di un telefono cede come un elastico quando lo tiri, le righe di una tabella ordinata si spostano al loro posto, un numero che cambia scorre come un contachilometri e i grafici si disegnano da soli la prima volta che vengono visti. Tutto segue la scelta Movimento di Impostazioni → Aspetto e resta fermo con il movimento ridotto. FinMotion è un progetto a sé (`github.com/finstats/finmotion`), messo sopra FinUI.
- **Più icone in FinUI**: ora sono 239, erano 71, ciascuna con un movimento suo (volume su e giù, una cuffia e una silenziata, una matita che scrive, una campanella che suona, un telefono che vibra), e ognuna si può scaricare dalla galleria di FinUI come SVG, ferma o animata.
- **Caratteri**: trentaquattro famiglie con licenza aperta tra cui scegliere (Geist, Roboto, Manrope, IBM Plex Sans, Montserrat, Lora, Playfair Display, Fira Code e altre), incluse come Inter, mai scaricate da altrove, e caricate solo quando vengono scelte.
- **Più basi e accenti**: Nord, Solarized, Gruvbox, Catppuccin, Rosé Pine, Tokyo Night, Dracula, Everforest e Oceano; sedici accenti in più, dieci dei quali di un colore di giorno e di un altro di notte. Ogni base ora ha di giorno una carta tutta sua: nel tema chiaro i grigi si somigliavano.
- **Eliminare si può annullare per 30 giorni**: una riproduzione o un backup eliminati aspettano nel cestino, e **Annulla** sull’avviso che segue li riporta indietro. Attività e Impostazioni → Backup elencano ciascuna ciò che è stato eliminato di recente, con la data in cui sparirà per sempre, ogni volta che ce n’è. Una riproduzione nel cestino è esclusa da ogni totale, grafico, riepilogo e profilo, e un’importazione o un ripristino non la riportano come nuova; chi l’ha guardata con qualcuno risulta da solo finché non torna.

### Changed
- Dark Reader e le estensioni simili lasciano stare le pagine di FinStats (`darkreader-lock`): FinStats disegna da sé entrambi i temi, e una pagina ridipinta entrava in conflitto con loro.
- Sul telefono il menu non scivola più da sinistra: la barra delle schede sta in basso, a meno che non se ne scelga un altro.
- Dopo questo aggiornamento FinStats legge la libreria una volta, per sapere dove finiscono i file che contengono più episodi.
- Nella pagina di un titolo, **Aggiungi alla mia lista** è alto quanto **Apri in Jellyfin** che gli sta accanto; prima era di una taglia più piccolo.
- I filtri di periodo e di persone restano in cima allo schermo mentre una pagina scorre, così 7g diventa Tutto anche dal fondo di una pagina lunga; lo stesso fanno i filtri delle richieste e delle prossime uscite di In arrivo.
- Il pulsante per tornare in cima sta lontano da qualunque menu occupi il fondo della finestra: in basso nell’angolo accanto al dock, appena sopra la barra delle schede, il pulsante del pollice o la barra degli indirizzi di un telefono, e altrimenti sopra la barra di stato. È il `to-top` di FinUI.
- Eliminare una riproduzione o un backup lo sposta nel cestino invece di rimuoverlo subito (`PUT` con `{"deleted": true}`; `DELETE` non esiste più nell’API). I backup oltre il numero conservato vengono ancora rimossi per sempre.
- Le **Novità** sono un unico foglio invece di una pila di schede: ogni nota è una riga a sé con il suo tipo (Aggiunto, Modificato, Corretto e gli altri) e l’icona di quel tipo in una colonna davanti, sotto un’intestazione per ogni serie. L’elenco delle versioni resta a destra.

### Performance
- Aprire FinStats scarica solo ciò che è cambiato: un foglio di stile, uno script o un carattere che il browser ha già viene confermato in pochi byte invece di essere inviato di nuovo. Ogni visita scaricava per intero tutti i fogli di stile e i moduli di FinStats, anche se ognuno aveva un tag per distinguerli.

### Stability
- Ripristinare un backup legge le impostazioni solo dalla riga delle impostazioni: un file di un FinStats più recente che lì conserva di più perdeva le impostazioni senza dire nulla.
- Una scheda aperta non cresce più a ogni pagina visitata: ogni pagina lasciata restava in memoria dietro quella sullo schermo (circa 2.600 elementi a visita), trattenuta dall’elenco di ciò che FinMotion muove e dal listener di scorrimento della barra dei filtri, finché la scheda non veniva chiusa.
- Un comando scritto male (`finstats bakcup`), o `restore` senza il suo file, non crea né aggiorna più un database prima di dire che non sa cosa fare.

### Fixed
- Il piccolo grafico di un riquadro non esce più dal riquadro quando il numero accanto è largo (“56g 23h”), o quando quattro riquadri condividono una riga più stretta: il grafico cede e il numero mantiene la sua larghezza.
- Nel tema chiaro un pulsante principale diventava viola sotto il puntatore; ora mantiene il colore del sigillo.
- In Impostazioni un interruttore sta allineato al nome e all’aiuto della sua impostazione; prima restava in cima alla riga.
- In un elenco in classifica senza nulla sotto un nome, come Utenti principali nella panoramica, il nome stava in cima alla riga accanto a un’immagine centrata; ora è centrato.
- Una fila di riquadri statistici a una larghezza tra telefono e desktop (quattro riquadri, tre per riga) lasciava l’ultimo da solo su una riga; ora quattro riquadri vanno quattro, due o uno per riga, e tre vanno tre o uno.
- La lettura del server contava un dispositivo su cui nessuno aveva effettuato l’accesso, che poi non conserva: l’operazione diceva tre dispositivi dove la pagina Server ne elencava due.
- Un secondo, un minuto, un’ora: gli avvisi dicevano “a 1 secondi di distanza” e “tra 1 minuti”, e un’operazione di Jellyfin con intervallo di un minuto veniva eseguita “ogni 1 minuti”.
- `finstats --help` nomina ogni variabile che FinStats legge; mancavano `FINSTATS_PUBLIC_IP_URL`, `FINSTATS_ALLOW_LIBRARY_SHRINK` e `FINSTATS_SKIP_PREUPDATE_BACKUP`.
- La configurazione guidata diceva di un indirizzo non valido che “non riusciva a connettersi”, in una frase che si fondeva con la successiva; ora dice che non è un indirizzo.
- In Impostazioni → Backup, una domanda posta mentre si scriveva un backup spariva quando il backup terminava.
- Con il menu a colonna di icone che si apre, chiudere la ricerca mostrava insieme la ricerca aperta e la colonna chiusa, e poi lasciava la colonna aperta finché il puntatore non entrava e usciva. Ora Esc restituisce il focus della tastiera a dove si trovava quando la ricerca è stata aperta, e lascia andare quello di un clic.
- Sul telefono, chiudere la ricerca lasciava la pagina oscurata finché la ricerca non era sparita, per poi tornare luminosa di colpo in un fotogramma; ora l’oscuramento svanisce insieme alla ricerca.
- I passaggi dell’importazione da Tautulli dicevano che il backup si scarica dopo **Backup Database**. Tautulli lo salva sulla propria macchina: ora i passaggi dicono dove (la sua cartella `backups`, `/config` con Docker), quale file prendere e di copiarlo qui.
- Una ricerca chiusa prima di aver finito di aprirsi lasciava senza animazione la colonna di icone che si apre: da quel momento si apriva e chiudeva di colpo.
- La colonna di icone che si apre restava aperta dopo averci scelto una pagina, fino a un clic altrove, e su alcuni browser, o con lo zoom, disegnava una barra di scorrimento sotto le icone. Ora si apre solo per il puntatore e la tastiera, e da chiusa non disegna barre di scorrimento.
- Nella pagina Sicurezza, un indirizzo che il database colloca in un paese ma non sulla mappa conta per quel paese, invece che tra gli indirizzi senza un luogo noto.

## [2.1.2] - 2026-10-01

### Added
- **Importa da Tautulli**: porta qui la tua cronologia di Plex. Carica il backup del database di Tautulli (il `.db`, o lo `.zip` che lo contiene) in **Impostazioni → Importa**; le sue riproduzioni hanno una voce tutta loro nel filtro *Registrato da* di Attività.
- **Un pannello di cablaggio per dire chi è chi**: gli utenti Plex da una parte, quelli di Jellyfin dall’altra. Trascina un cavo da ogni utente Plex a chi è adesso, oppure fai clic su uno e poi sull’altro, o usa la tastiera. Chi resta scollegato non viene importato, e due account Plex possono confluire in una sola persona.
- Film ed episodi vengono abbinati alla tua libreria per nome (un film per titolo e anno, un episodio per serie e numero), e uno non ancora presente sul server viene conservato e collegato quando arriva. Una visione ripresa è una sola riproduzione; la musica non viene importata.
- Il backup contiene i token di accesso di Plex: FinStats non li legge mai, e rimuove il file appena importi o ricominci da capo.
- **Contenuti non collegati** in **Impostazioni → Dati**: ogni titolo da Tautulli, Jellystat o Streamystats che non corrisponde a nulla nella tua libreria, con la sua provenienza e il punto in cui con ogni probabilità si trova già. **Trova** lo collega da una delle ipotesi migliori o da una ricerca (un film può essere trovato come speciale di una serie), e FinStats ricorda la scelta, così la stessa cronologia importata di nuovo si collega da sola. Le scelte fanno parte dei backup.

### Performance
- **Il riepilogo si apre da tre a cinque volte più in fretta**: gli attori e i registi più visti dell’anno venivano calcolati riesaminando l’anno una volta per ogni persona presente. Misurato su una cronologia reale, un anno è passato da 0,29 s a 0,09 s e gli ultimi 12 mesi da 0,53 s a 0,12 s.

### Fixed
- **La cronologia importata trova i titoli che hanno un altro nome**: una riproduzione si abbina anche al titolo in lingua originale, così una serie che Plex o Tautulli conoscevano come 오징어 게임 si collega a Squid Game, e *Im Westen nichts Neues* ad *All Quiet on the Western Front*. I titoli originali della tua libreria vengono letti alla prossima analisi.
- I nomi vengono confrontati come li confronta la ricerca (maiuscole, accenti, punteggiatura, un anno scritto nel nome, un “The” iniziale), e l’anno di uscita può differire di uno, così “Star Wars: Episode I - The Phantom Menace” trova “… – The Phantom Menace” e un Kingsman del 2014 trova quello del 2015. Un titolo noto con un nome del tutto diverso non viene comunque mai indovinato.
- Un’importazione o un ripristino da Streamystats non riuscito veniva annunciato nelle notifiche con il suo nome interno.
- Il pulsante *Condividi l’anno in schede* del riepilogo e la posizione in classifica accanto non avevano spazio tra loro.

## [2.1.1] - 2026-10-01

### Changed
- **La lista è offerta ovunque ci sia un titolo**: sulle schede In arrivo della panoramica, come in In arrivo → Prossime uscite, e sulla pagina di un episodio o di una stagione, che aggiunge alla lista la sua serie.
- Su una locandina della panoramica il pulsante della lista compare mentre punti la scheda o la raggiungi con la tastiera, invece di coprire l’angolo di ogni locandina; su uno schermo touch c’è sempre.

### Stability
- Salvare una connessione Sonarr, Radarr o Seerr mentre qualcuno aveva aperto **Impostazioni → Connessioni in uscita** poteva bloccare entrambe le parti in un’attesa reciproca senza fine.
- **Impostazioni illeggibili fermano FinStats all’avvio**, nominando l’impostazione colpevole, invece di riportare in silenzio ogni impostazione al suo valore predefinito.
- **Due modifiche alle impostazioni nello stesso momento** (la pagina delle impostazioni e la pianificazione di un’operazione, per esempio) non si annullano più a vicenda.
- Una pausa, un salto o un cambio di traccia visti mentre il database era occupato vengono scritti al passaggio successivo invece di andare persi.
- Una lettura della libreria non fallisce più su una lista con più voci che risultano essere un solo titolo.

### Fixed
- **Un file sostituito non è “ora sul server”**: aggiornare il file di un film nella lista di qualcuno non gli dice più che è appena arrivato.
- Una lettura non riuscita dei plugin, delle operazioni pianificate o dello spazio di archiviazione di Jellyfin non svuota più quell’elenco nella pagina Server fino alla lettura successiva.
- **Una riproduzione di Streamystats si ferma dove si è fermata, su ogni pagina**: la curva di abbandono di un titolo e il “finito” di un episodio si basavano invece sulla sua durata, e la contavano come stimata.

## [2.1.0] - 2026-09-29 to 2026-09-30

La mia lista

### Added
- **Una lista tutta tua**: aggiungi un film o una serie dalla sua pagina, da Prossime uscite, da Aggiunti di recente o dalla ricerca (Ctrl+Invio), e ritrovalo in **La mia lista** sul tuo profilo.
- **Ogni voce dice a che punto è adesso**: sul server, “5 di 26 episodi”, richiesto, “In arrivo venerdì”, non sul server, uscito dalla libreria, o visto. Viene calcolato ogni volta che guardi, con la stessa lettura di “visto” dell’avanzamento delle tue serie. I titoli visti restano, in un gruppo a parte, finché non li togli.
- **Un titolo non ancora sul tuo server** può finire nella lista direttamente da ciò che Sonarr o Radarr stanno aspettando. Viene collegato al titolo da solo quando arriva, e un file rinominato si porta dietro la sua voce.
- **“Dalla tua lista: ora disponibile”**: una notifica per le tue destinazioni quando arriva qualcosa che aspettavi. Disattivata finché non la spunti.
- **Nessun altro vede la tua lista**, nemmeno un amministratore, e la sua notifica va solo alle tue destinazioni. Non viene scritto nulla in Jellyfin, Seerr, Sonarr o Radarr.
- Le liste fanno parte dei backup, e un ripristino le unisce senza duplicare nulla.

### Changed
- Le voci di `/api/upcoming` portano gli id TVDB e TMDB del loro titolo, e `/api/auth/me` dice se il server conserva le liste.

## [2.0.4] - 2026-09-29

### Added
- **Pianificazione delle operazioni, come fa Jellyfin**: ogni operazione che FinStats svolge da solo ha delle attivazioni (ogni giorno a un’ora, ogni settimana in un giorno, a intervalli, all’avvio o dopo l’analisi della libreria di Jellyfin), ciascuna con un limite di tempo facoltativo. Fai clic su un’operazione per vederne e cambiarne la pianificazione; *Aggiungi attivazione* apre la stessa finestra che ha Jellyfin.
- **Operazioni è una sezione a sé** in Impostazioni, con l’ultima esecuzione di ogni operazione a parole (“Ultima esecuzione 38 minuti fa, durata 6 minuti”) e un pulsante Esegui; prima stava in cima a Sistema.
- **Modifiche ai metadati**: ogni ora FinStats legge ciò che Jellyfin ha salvato dall’ultima volta (nomi, trame, generi, valutazioni, cast e troupe, dettagli dei file e immagini), così una modifica in Jellyfin compare senza aspettare un’analisi della libreria.
- Anche il backup e il database di geolocalizzazione si possono eseguire da Operazioni; Esc torna dalla pianificazione di un’operazione all’elenco.

### Changed
- **Il tema è un interruttore a tre posizioni**, Chiaro · Dispositivo · Scuro, dove prima c’era il pulsante che le alternava: fai clic su un terzo, trascina la manopola o usa i tasti freccia. La manopola porta l’icona della scelta, che entra con un movimento quando cambia.
- **L’uscita è in Impostazioni → Account**, che dice chi ha effettuato l’accesso qui; la barra laterale tiene l’interruttore del tema.
- **Le vecchie impostazioni di pianificazione ora sono attivazioni**: *Segui l’analisi della libreria di Jellyfin* con il suo intervallo di riserva, *Backup ogni N giorni* e l’interruttore GeoIP *Mantieni aggiornato il database* sono diventati le attivazioni corrispondenti sulle loro operazioni, con i valori che avevi. Raccolta, Backup e Sicurezza rimandano invece alla pianificazione.
- Un avvio non esegue più di nuovo ogni operazione: viene conservato il momento in cui ciascuna è terminata l’ultima volta.
- Un’operazione di Jellyfin ancora senza stima mostra “circa … rimanenti”, con i puntini che scorrono dove andrà il tempo, invece di “ETA…”.

### Performance
- **La mappa di Sicurezza non va più a scatti**: trascinare e zoomare spostano la mappa già disegnata e la ridisegnano una volta quando il gesto si ferma, e il battito di ciò che è in riproduzione non fa più ridipingere l’intera mappa 120 volte al secondo mentre nessuno la tocca. Un trascinamento è passato da 34 a 60 fotogrammi al secondo su un computer lento.

### Fixed
- **Una locandina o un ritratto sostituiti in Jellyfin compaiono in FinStats**: le immagini restavano in cache una settimana sul server e un’altra nel browser, qualunque cosa Jellyfin avesse nel frattempo. Ora sono in cache con il tag dell’immagine di Jellyfin, così un’immagine nuova è un file nuovo, e il browser verifica che la sua copia sia aggiornata ogni volta che ne mostra una. Sostituire un ritratto salva di nuovo solo la persona, quindi Modifiche ai metadati legge anche le persone.
- **Server → Operazioni mostra una stima appena si apre la pagina**: il tempo rimanente veniva misurato solo dalle letture fatte dalla pagina stessa mentre era aperta, quindi ogni visita cominciava con “ETA…” finché non aveva visto muoversi la percentuale. Ora FinStats cronometra da sé un’operazione di Jellyfin in corso, ogni 10 secondi finché è in esecuzione, e la pagina riceve la stima. Un’operazione che Jellyfin riporta allo 0% per tutto il tempo (la prima fase di un’analisi della libreria) mantiene i puntini finché non si muove: niente viene indovinato.
- **I tooltip della mappa compaiono sul punto**: in Firefox il tooltip del punto in riproduzione poteva finire lontano, e dove i punti si sovrappongono (qualcuno che guarda da casa) quale tooltip compariva dipendeva da un pixel. Ora un unico tooltip nomina tutto ciò che sta sotto il puntatore.
- **La barra di stato ha un’unica forma al passaggio del puntatore**: ogni link (gli stream, Repo, la versione) si illumina come lo stesso piccolo riquadro, e i punti tra le voci stanno centrati negli spazi. Passare sulla versione illuminava anche il punto e lo spazio prima di essa.
- **Un percorso di file occupa tutta la larghezza della sua scheda**: nella scheda File di un titolo, e per le cartelle di Jellyfin nella pagina Server, un percorso veniva schiacciato in una sola cella stretta e scendeva per nove righe accanto a spazio vuoto.
- Una finestra di dialogo che sta svanendo non inghiotte più un clic sulla pagina sottostante.

## [2.0.3] - 2026-09-28

### Security
- La pagina Server dava l’id di ogni dispositivo a chiunque avesse *Vedere il server*; gli id richiedono anche *Vedere i dettagli di rete*, come ovunque.
- Due interruttori dei permessi azionati prima che il primo fosse salvato inviavano due volte lo stesso elenco di partenza, e il secondo annullava il primo: un permesso tolto restava concesso, una sezione del profilo pubblico disattivata restava pubblicata, mentre entrambi gli interruttori mostravano la scelta fatta.

### Stability
- **Una sola importazione o un solo ripristino alla volta**: due caricamenti inviati insieme superavano entrambi il controllo, e un ripristino non guardava mai le importazioni; il secondo aspettava che il database si liberasse e falliva dopo minuti di caricamento.
- **Un indirizzo con un escape `%` rotto apre la pagina non trovata**, invece di lasciare uno spinner per sempre.
- **Un numero di pagina grande in un elenco è una pagina vuota**: andava in overflow, e tornava come la prima pagina.

### Fixed
- **Le tabelle ordinano bene i numeri in un browser tedesco o norvegese**: “1,204” veniva letto come un numero decimale, e tutto ciò che superava il milione come testo.
- **La vista Tabella di un grafico ordina date, giorni della settimana e posizioni in ordine di tempo**, non in ordine alfabetico.
- **La cronologia dei download dice da dove vengono le importazioni**: Sonarr e Radarr nominano l’indexer e il protocollo solo al momento del grab, quindi quelle suddivisioni erano sempre vuote. Un season pack non riuscito è un solo fallimento, non uno per episodio.
- **Eliminare una riproduzione vista insieme** ricalcola i gruppi di quel titolo, invece di lasciare un “gruppo” di una persona.
- **Il tempo insieme, per una persona, non supera mai quanto è rimasta**: la pagina Insieme le attribuiva quello dell’intera serata.
- **Una serata in compagnia a cavallo della mezzanotte di San Silvestro** (o del limite degli “ultimi N giorni”) appartiene interamente al periodo in cui è iniziata, invece di diventare due serate da soli.
- **Le riproduzioni di episodi importate prendono il nome dell’episodio** dove Jellystat aveva conservato solo quello della serie.
- **La geolocalizzazione trova il database delle città accanto a quelli Country e ASN** che scrive `geoipupdate`, invece di disattivarsi da sola.
- **Il punto caldo dei più riavvolti segue i filtri della pagina**, come i suoi conteggi.
- **“Il tuo anno è pronto”** arriva solo a chi ha qualcosa nel riepilogo.
- **Risolvere di nuovo un avviso silenziato mantiene silenziata la sua coppia**; solo riaprirlo annulla il silenziamento.
- **Una chiave API scaduta non conta più tra le venti.**
- **Il limite di sessioni lascia andare la sessione più vecchia** anche quando molti accessi arrivano nello stesso secondo.
- **Un indirizzo di casa trovato dalla ricerca automatica resta** quando lo stesso viene tolto dall’elenco scritto a mano.
- **Immagini nel registro di audit e nell’elenco delle chiavi**, dove c’erano solo le iniziali.
- **L’elenco delle notifiche di un amministratore dice di chi è una destinazione personale**, invece di “Tua” su quelle di tutti; **Inviati di recente** mostra ogni destinazione del suo ultimo evento.
- **Esc su un titolo o una persona non esce mai da FinStats**, e chiude solo la finestra di dialogo in primo piano.
- **Una pagina viene disegnata anche quando un salvataggio annulla il precaricamento che stava aspettando.**
- **In riproduzione mostra uno spettatore in pausa che scorre** dove si trova adesso, non dove era quando ha premuto play.
- **Rimuovere una connessione aggiorna quali pagine esistono**, come fa salvarne una.
- **La data di creazione del database di geolocalizzazione mantiene l’anno** in inglese americano.

## [2.0.2] - 2026-09-28

### Security
- Dietro un proxy, il limite di accessi contava il primo indirizzo `X-Forwarded-For`, che è quello inviato dal client: un nuovo indirizzo inventato a ogni tentativo, e il limite non si applicava mai. Ora conta l’ultimo, quello scritto dal proxy.
- *Includi indirizzi IP e luoghi* inviava indirizzi a una destinazione personale il cui proprietario non ha *Vedere i dettagli di rete*, che nessuna pagina gli avrebbe mostrato. Ora per gli accessi non riusciti di altri serve anche *Vedere il server*, come nella pagina Sicurezza.

### Stability
- **Un arresto non va mai perso**: quando una lettura della libreria veniva rifiutata nei primi istanti dopo l’avvio, prima che il server fosse in ascolto, il rifiuto veniva annotato e FinStats continuava invece di fermarsi.
- **Un Jellyfin che risponde in una forma che FinStats non sa leggere viene rifiutato**, non creduto: ogni utente, o ogni titolo di una libreria, arrivato sotto chiavi sconosciute (`id` per `Id`) superava il controllo con un conteggio pieno e veniva poi segnato come rimosso.
- **La fine di una riproduzione non va mai persa**: quando il database restava occupato oltre la sua attesa (un’importazione che lo teneva), una riproduzione terminata in quel momento restava segnata come in corso fino al riavvio successivo. Ora viene scritta al passaggio successivo.
- **Le impostazioni che un ripristino non sa leggere vengono lasciate stare**: un backup le cui impostazioni non corrispondevano più a questa versione azzerava ogni impostazione al ripristino: finestra di unione, permessi, indirizzi di casa, pianificazione.
- **Un carattere di controllo in un nome o in un titolo non rompe più le schede**: la scheda del profilo, ogni scheda della storia che mostrava quel titolo e il loro ZIP rispondevano con un errore.

### Fixed
- **Le notifiche si fermano per chi viene disattivato o eliminato in Jellyfin**: alle sue destinazioni personali continuava ad arrivare tutto ciò che i suoi vecchi permessi consentivano.
- **Una notifica in coda viene ricontrollata prima di essere inviata**: a una destinazione disattivata, o di un proprietario che aveva perso un permesso, veniva ancora inviato ciò che aspettava di essere ritentato.
- **Una destinazione che risponde sempre `Retry-After` viene abbandonata** dopo cinque tentativi come qualunque altra, invece di essere ritentata per trenta giorni.
- **Gli id dei dispositivi nella pagina di una persona** vengono mostrati solo con *Vedere i dettagli di rete*, come ovunque.
- **Un anno pubblicato aspetta un giorno per tutto**: le sue serie finite e abbandonate seguivano ogni riproduzione, quindi premere play su una serie abbandonata cambiava la pagina pubblica in pochi secondi.
- **Una serie richiesta conta come vista** quando lo è uno dei suoi episodi: ogni serie richiesta risultava mai vista, nel capitolo e nella scheda Richieste.
- **Il capitolo Insieme del riepilogo esclude la TV in diretta**, come il resto dell’anno: un canale lasciato acceso insieme poteva diventare il titolo principale, e la quota poteva superare il 100%.
- **La scheda dei giorni degli “ultimi 12 mesi” finisce con il giorno più recente**: la griglia perdeva le ultime tre o quattro settimane.
- **I nuovi episodi di una serie vengono annunciati una volta per serata**: gli episodi a cavallo della mezzanotte UTC venivano annunciati due volte su un server a est o a ovest di Greenwich.
- **Collegare Seerr non annuncia vecchie richieste come pronte da guardare**: una richiesta già disponibile alla prima lettura, senza altra indicazione di quando, risultava arrivata in quel momento.
- **I titoli di una libreria eliminata in Jellyfin se ne vanno con lei**: restavano nella ricerca, nella fila Aggiunti di recente e nei totali.
- **Un remux resta uno streaming diretto** quando una lettura perde per un momento i dettagli della transcodifica (un salto), invece di diventare una transcodifica per il resto della riproduzione.
- **“Jellyfin non risponde” vuol dire letture fallite una dopo l’altra**: su un server inattivo sul socket bastavano, sommati tra loro, timeout sporadici a giorni di distanza.
- **Un’importazione da Streamystats mostra l’avanzamento sulla propria scheda**, non sopra l’ultimo risultato dell’importazione da Jellystat.
- **Un indirizzo il cui host inizia con “web”** (`http://web:8096`) viene letto come host, non troncato come se fosse il percorso del client web di Jellyfin.
- **Un errore della coda oscura per intero le credenziali** di un indirizzo anche quando la password contiene una barra.

## [2.0.1] - 2026-09-28

### Changed
- **Le lingue di un titolo non ingombrano più**: l’intestazione nomina le prime tre lingue dell’audio e dei sottotitoli, una riga ciascuna, e una nuova scheda **Lingue** le elenca tutte: quelle complete insieme, quelle parziali con quanti episodi le hanno. Le due dozzine di lingue dei sottotitoli di una serie lunga spingevano l’immagine di sfondo giù di uno schermo.

### Fixed
- **La locandina di un titolo resta accanto al suo nome**: quando il testo accanto era più lungo della locandina (una lunga trama, molti studi), la locandina scendeva in fondo all’intestazione, sotto un’immagine di sfondo vuota.
- **Una versione che FinStats non sa leggere viene chiamata così**: un `app_version` modificato a mano in qualcosa di illeggibile faceva rifiutare a FinStats l’avvio come “il FinStats 2.0.1 più vecchio”, che non era. Lo rifiuta ancora, e ora dice quale valore non ha saputo leggere e come scriverlo.

## [2.0.0] - 2026-09-25 to 2026-09-28

Riepilogo 2026, profili pubblici e visione insieme

### Added
- **Con chi hai guardato**, nel riepilogo: le serate in compagnia, il tempo trascorso così, il titolo che ha riunito di più le persone, e fino a tre compagni, nominati nell’app e collegati ai loro profili.
- **Serie viste fino alla fine**, nel riepilogo: l’ultimo episodio di ciascuna visto quell’anno, con la stessa lettura di “visto” delle barre di avanzamento del profilo, e anche quelle iniziate e lasciate per dopo, viste per meno della metà e senza nulla riprodotto da due mesi.
- **Cosa hai chiesto**, nel riepilogo: le richieste a Seerr fatte nell’anno, quante sono arrivate e quante poi ne hai guardate. Solo dove esistono richieste.
- **L’anno a confronto con quello precedente**: ore, riproduzioni e giorni, e quanto di più o di meno.
- **L’anno come storia**: ogni capitolo come scheda 1080×1920, il formato delle storie sui telefoni (l’anno, i numeri, serie, film e musica preferiti, generi, la personalità, ore e giorni, ogni giorno dell’anno, record, compagnia, serie finite, richieste e l’anno precedente). Salvane una, o tutte come ZIP, da “Condividi l’anno in schede”.
- **Una scheda non nomina mai nessun altro**: le schede sono disegnate da una copia dell’anno senza alcun campo per un’altra persona, una posizione in classifica o un’app. Nell’app una scheda non porta alcun nome.
- **L’anno del server**, per gli amministratori di Jellyfin: ogni titolo e totale e la personalità della casa, senza nessuno in classifica o nominato. Un interruttore nella pagina del riepilogo, `scope=server` nell’API.
- **“Il riepilogo dell’anno è pronto”**: una notifica a dicembre, una volta per ogni persona che ha guardato qualcosa quell’anno.
- **Profili pubblici**: una persona può pubblicare parte del suo profilo a un link che chiunque può aprire senza account (totali e titoli principali, giorni di fila e quando guarda, l’intero anno come storia di schede, le ultime dieci riproduzioni), ogni parte disattivata finché il proprietario non la attiva in Impostazioni → Profilo pubblico. Dispositivi, indirizzi, app, metodi di riproduzione, percorsi dei file e altre persone non vengono mai pubblicati, qualunque cosa sia attiva, e il nome mostrato è uno che il proprietario scrive, mai il suo login di Jellyfin.
- **Niente su una pagina pubblicata è più recente di un giorno**, riproduzioni recenti e ogni totale allo stesso modo, così una pagina che qualcuno tiene d’occhio non dice mai chi sta guardando in questo momento.
- **Schede da condividere**: ogni profilo pubblicato ha un’immagine 1200×630, del profilo e dell’anno, che Discord, Signal, Mastodon e gli altri mostrano quando il link viene incollato, e che si può scaricare per pubblicarla. Disegnata sul server da ciò che mostra la pagina e nient’altro.
- **Un link che puoi ritirare**: 22 caratteri casuali, reimpostati con un clic; il vecchio link risponde poi esattamente come uno mai esistito.
- **I profili pubblici sono disattivati finché un amministratore non li consente**: un interruttore per tutto il server, un elenco di chi pubblica cosa, e un modo per rimuovere un profilo. Pubblicazioni, modifiche, reimpostazioni e rimozioni sono nel registro di audit, e una chiave non può rendere pubblico nulla.
- **Insieme**, una pagina a sé: chi guarda con chi, quanto, e come è cambiato. Tempo insieme, serate e quota del tempo di visione trascorso in compagnia, ciascuno rispetto al periodo precedente; ore in compagnia contro ore da soli al giorno o a settimana; ogni coppia di persone con le sue serate, il tempo insieme, quando ha guardato l’ultima volta e cosa guarda di più; i titoli più visti insieme; le serate recenti; e, per chi può vedere tutti, il tempo da solo di ogni persona rispetto al tempo in compagnia.
- Ogni coppia conta: una serata in tre conta per ciascuna delle sue tre coppie, mentre la scheda contava solo l’esatto gruppo di persone. Il tempo insieme di una coppia è la più breve delle due permanenze.
- La scheda **Visti insieme** nella panoramica e su un profilo porta alla pagina, e la scheda di un profilo dice quale quota del tempo di visione di quella persona è stata in compagnia.
- `GET /api/stats/groups` acquisisce `pairs`, `series`, `people`, `previous`, `bucket`, e `watch_s` e `share` nei totali; le chiavi che aveva restano invariate.
- **Dove le persone si fermano**, sulla pagina di ogni film ed episodio: la quota di riproduzioni ancora in corso a ogni minuto, da tutti all’inizio a chi è arrivato alla fine, con i riavvolgimenti e le attivazioni dei sottotitoli di quelle riproduzioni segnati sullo stesso asse. La scheda dice quante interruzioni sono state misurate (riproduzioni registrate da FinStats) e quante stimate (riproduzioni importate, in base a quanto sono durate), e servono tre riproduzioni prima che disegni qualcosa.
- **Chi continua a guardare**, sulla pagina di una serie: quante persone hanno iniziato ogni episodio, in ordine, e una colonna **Finiti** per episodio. “Tutti mollano al terzo episodio” vuol dire persone che non premono mai play sul quarto.
- **File che non partono mai**, in Riproduzione: un film o un episodio avviato tre volte o più che non ha mai superato i trenta secondi, con chi ci ha provato e su quali app. Il registro di Jellyfin non riporta errori di riproduzione, quindi le riproduzioni fanno da testimone; la durata minima di una riproduzione qui non si applica, perché è proprio ciò che quelle riproduzioni non raggiungono mai.
- **I più riavvolti** e **Sottotitoli attivati**, in Riproduzione: titoli in classifica per salti all’indietro per riproduzione, con il minuto in cui si concentrano, e per quota di riproduzioni in cui i sottotitoli sono stati attivati entro i primi dieci minuti. Tutti e tre gli elenchi richiedono *Vedere l’attività di tutti*. I salti più brevi di 20 secondi non sono mai stati registrati, quindi riascoltare una sola battuta è invisibile per scelta.
- Un salto conserva da dove è partito come numero accanto alla sua etichetta (`from_s` nella linea del tempo di una riproduzione). Ogni salto già registrato prende il suo numero dall’etichetta, e così anche un backup ripristinato di prima di questa versione.
- `GET /api/stats/files` per i tre elenchi; `GET /api/items/{id}` acquisisce `insights`, e gli episodi di una serie acquisiscono `users` e `finished`.
- **Chiavi API**: chiunque abbia effettuato l’accesso può creare chiavi per sé in Impostazioni → Chiavi API, per uno script, una dashboard o un telefono. Una chiave si invia come `Authorization: Bearer fs_…`, viene mostrata una volta sola, conservata come hash, può avere una scadenza e si revoca con un clic; un amministratore vede e può revocare quelle di tutti. Corrisponde alla stessa persona con gli stessi permessi di un accesso, letti in tempo reale a ogni richiesta, così perdere il diritto di accedere ferma subito ogni chiave. Una chiave non può né creare né revocare chiavi.
- **Un feed del calendario**: `GET /api/calendar.ics?key=…` è ciò che Sonarr e Radarr hanno in arrivo, come iCalendar a cui un telefono può iscriversi. Una chiave con ambito *calendario* apre il feed e nient’altro; il feed legge la chiave dall’indirizzo perché un calendario sottoscritto non può inviare header, mai dal cookie, e non nomina nessuno.
- **Un registro di audit** in Server → Audit, per gli amministratori di Jellyfin: ogni accesso e tentativo non riuscito con il suo indirizzo, ogni impostazione o permesso cambiato e in cosa, ogni chiave creata, usata o revocata, ogni connessione, destinazione, backup e importazione, ogni riproduzione eliminata e avviso risolto, con chi, da dove, tramite quale chiave e se è andato a buon fine. Conservato un anno, parte dei backup. `GET /api/audit` lo scorre a pagine.
- **Importa da Streamystats**, una sezione a sé in Impostazioni accanto a Jellystat. Prendi il suo file **Settings → Backup & Import → Download Backup** e trascinalo qui: anni di cronologia arrivano in un paio di secondi, e nulla di ciò che hai già viene contato di nuovo. `finstats import-streamystats <file>` lo fa senza interfaccia, `POST /api/import/streamystats` tramite l’API; si esegue un’importazione alla volta, di qualunque tipo.
- **Hai usato entrambi i tracker? Importa entrambi i file.** Una riproduzione viene riconosciuta come già presente in FinStats dall’id che le dà il tracker e, in mancanza, dalla stessa persona che guarda lo stesso elemento con una delle due estremità della riproduzione vicina a una della cronologia, così un’esportazione di Jellystat e una di Streamystats delle stesse serate possono essere importate entrambe, in qualunque ordine, e una serata che FinStats ha registrato da sé non viene sovrascritta da nessuna delle due. “Vicino” vuol dire entro la finestra di unione in Impostazioni → Raccolta.
- Le righe importate dicono ciò che non possono dire. Una riproduzione registrata da Streamystats stesso non conserva nulla del file (nessun codec, risoluzione, bitrate o contenitore), quindi questi campi restano vuoti invece di essere riempiti con un’ipotesi, e le schede che li mostrerebbero si nascondono. Una che aveva preso da Jellystat porta con sé l’intera sessione, e viene letta quella: datata dal momento in cui è terminata, l’unico momento che entrambi i tracker conservavano, e descritta come transcodifica solo quando la sessione lo era davvero.
- Le righe che Streamystats scriveva perché Jellyfin segnalava un elemento come visto, per una visione che non aveva mai osservato, vengono contate e saltate: durano quanto l’intero film e diventerebbero tempo di visione in una serata in cui nessuno ha guardato. Il risultato dice quante erano.
- Come vengono interpretati i dati di Streamystats, campo per campo, imparato da un’esportazione reale anziché dalla documentazione: `docs/streamystats-import.md`.
- **Da quale tracker arriva ogni riproduzione**, come filtro in Attività: ciò che FinStats ha registrato da sé, ciò che è arrivato da Jellystat, ciò che è arrivato da Streamystats. Vengono offerti solo quelli che la tua cronologia contiene davvero, e il filtro è limitato come ogni altro elenco, quindi chi può vedere solo le proprie riproduzioni sa solo da dove viene la propria cronologia.
- **I filtri di Attività sono menu a tendina con caselle da spuntare**, così puoi chiedere film *ed* episodi senza musica, due metodi di riproduzione o due persone insieme, invece di uno per tipo o di tutti. Il tipo di contenuto ha guadagnato **Altro** ora che si possono combinare.
- Più persone alla volta, ovunque compaia il filtro utenti (la panoramica, la riproduzione, le librerie, la sicurezza e il resto), con la mappa e ogni statistica che seguono. Un filtro utenti continua solo a restringere ciò che qualcuno può vedere: senza *Vedere l’attività di tutti* una richiesta è vincolata a chi la fa, chiunque nomini l’indirizzo e per quanti siano.
- **Un tema chiaro**: carta washi, con testo color inchiostro e un accento vermiglio. FinStats segue l’impostazione chiara o scura del dispositivo finché non ne scegli una.
- **Un pulsante del tema accanto a Esci** passa tra Dispositivo, Chiaro e Scuro. La scelta resta in questo browser e viene applicata prima che la pagina sia disegnata, così non lampeggia mai l’altro tema.
- **Apri in Jellyfin**: ogni pagina di film, serie ed episodio ha un pulsante che apre la pagina di quel titolo in Jellyfin, in una nuova scheda.
- **L’indirizzo di Jellyfin per le persone** in Impostazioni → Jellyfin: dove portano quei pulsanti, per quando l’indirizzo a cui si collega FinStats non è raggiungibile da un browser. Se è vuoto si usa l’indirizzo a cui si collega FinStats; solo un amministratore di Jellyfin può impostarlo.

### Changed
- **FinStats ha una nuova casa**: `github.com/finstats/finstats`, e l’immagine è `ghcr.io/finstats/finstats`. Cambia il nome dell’immagine nel tuo `docker run` o nel file compose per continuare a ricevere aggiornamenti; la cartella dei dati resta com’è.
- **Impostazioni mostra una sezione alla volta.** La pagina era quattordici schede in una colonna e seicento parole di aiuto. Ora ogni sezione ha il suo indirizzo (`/settings/collection`), un elenco a sinistra passa dall’una all’altra, e sul telefono l’elenco diventa una fila di chip. I link alle vecchie ancore funzionano ancora.
- **Un’impostazione è una riga**: cos’è, una riga che ne dice la regola, il controllo a destra. I numeri di una sezione si salvano insieme con un pulsante in fondo; gli interruttori si salvano ancora da soli. Le schede di stato (Operazioni, Connessioni in uscita, Database) e il link alle licenze stanno in **Sistema**, e i due importatori hanno una sezione a sé.
- **Trova un’impostazione** dalla casella nell’intestazione (`/` la mette a fuoco) con qualsiasi parola del suo nome; dice in quale sezione si trova l’impostazione, e Invio porta alla riga.
- **Impostazioni è per tutti quelli che hanno effettuato l’accesso**: le sezioni che gestiscono il server restano a chi può gestirlo, e chi aveva solo il permesso delle notifiche può finalmente raggiungere le proprie destinazioni.
- **La pagina Server mostra una sezione alla volta**, come Impostazioni: Panoramica (sistema e archiviazione), Operazioni, Dispositivi, Plugin, Registro e Audit, ciascuna al suo indirizzo. I link a `/server#jobs` funzionano ancora.
- **Il registro del server è una sezione della pagina Server** (`/server/log`) invece di una pagina a sé nel menu; `/events` rimanda lì con i suoi filtri.
- Una notifica su un’operazione non riuscita, un backup non riuscito o una connessione che ha smesso di rispondere porta alla sezione delle impostazioni che la riguarda, invece che in cima alla pagina.
- Ciò che la tua libreria può dire di una riproduzione importata (che tipo di cosa era, quale episodio, la durata, a quale libreria appartiene) viene completato dopo **ogni** lettura della libreria, non solo al momento di un’importazione. Di solito la cronologia viene importata prima che FinStats abbia mai letto la libreria, quindi quelle riproduzioni restavano per sempre ipotizzate o sconosciute.
- Ripristinare un backup di FinStats non aggiunge più una riproduzione arrivata da un tracker con un id diverso, e conserva comunque due visioni reali della stessa cosa a pochi minuti di distanza.
- I dettagli di una riproduzione nominano il tracker che l’ha portata, invece di chiamare riproduzione di Jellystat tutto ciò che è importato.

### Performance
- **Un riavvio è quasi istantaneo con qualsiasi cronologia.** Ogni avvio ricalcolava chi aveva guardato insieme su tutta la cronologia e ridecideva quali riproduzioni erano locali, entrambe le cose per non cambiare nulla: 24 s e 5,5 s su dieci milioni di riproduzioni. Ora un avvio fa solo ciò che l’ultima esecuzione ha lasciato in sospeso, e tutto di nuovo solo dopo un aggiornamento o quando cambia l’impostazione che c’è dietro.
- **Una folla che arriva tutta insieme viene registrata in pochi secondi.** Ogni nuova riproduzione, ogni salvataggio dell’avanzamento e ogni dispositivo venivano scritti singolarmente, e ogni riproduzione terminata veniva chiusa con un proprio esame dell’intera cronologia del titolo; ora un passaggio scrive ogni tipo in blocco. 100.000 riproduzioni che partono insieme richiedono 3 s, mentre 25.000 che finivano insieme richiedevano più di quattro minuti.
- **In riproduzione resta rapido con migliaia di spettatori.** Per capire chi guarda insieme si confrontava ogni stream con tutti gli altri e si elencava ogni compagno per nome: quaranta secondi a 10.000 stream, e gigabyte di memoria per giunta. Ora gli stream vengono confrontati all’interno di un titolo, e ciascuno mostra pochi nomi e quanti altri ci sono.
- Calcolare chi aveva guardato insieme riscriveva ogni riproduzione di gruppo della cronologia ogni volta che FinStats si avviava, importava o vedeva finire una riproduzione; ora scrive solo ciò che è cambiato. Trovare quali riproduzioni sono terminate non rallenta più con il quadrato del numero di quelle in corso.
- **Riconoscere una riproduzione già presente nella cronologia passa per un indice**, così un’importazione di 150.000 riproduzioni richiede pochi secondi, e il ricollegamento dopo una lettura della libreria scorre i titoli invece di leggere ogni riproduzione tenendo occupato il database.
- **Gli approfondimenti sulla riproduzione e la panoramica sono più rapidi su tutto il periodo**: le loro parti indipendenti vengono calcolate in parallelo, il grafico degli stream contemporanei cerca il giorno locale una volta ogni quarto d’ora invece che per ogni riproduzione, e i generi vengono contati per titolo. Su un milione di riproduzioni: da 4,1 s a 2,4 s, e da 1,6 s a 1,1 s.

### Stability
- **Una libreria di serie TV non viene più scambiata per una libreria svuotata.** Leggere una libreria richiede due passaggi (ogni elemento, poi uno più piccolo solo per cast e troupe di film e serie), e il controllo che decide se ci si può fidare di una lettura per rimuovere ciò che non ha visto guardava il secondo. In una libreria di serie TV o di musica i due conteggi sono lontanissimi, quindi il controllo rifiutava ogni lettura e fermava FinStats, e l’installazione si riavviava e faceva di nuovo lo stesso. Nulla è mai stato rimosso per errore, ma nessuna lettura della libreria poteva terminare. Se la tua libreria di serie o di musica non riceveva più nuovi titoli, il motivo è questo.
- Un’importazione poteva fallire con “database is locked” quando un’altra scrittura (una riproduzione in aggiornamento) arrivava tra la sua prima lettura e la sua prima scrittura. Entrambi gli importatori prendono il blocco di scrittura appena iniziano, e il journal del database non resta più grande quanto la più grande importazione mai fatta.
- **Un file di database danneggiato, troncato, di qualcun altro o di sola lettura viene rifiutato subito**, nominando il file e dicendo che non è stato cambiato nulla, invece di 30 secondi di silenzio; non si scrive mai nel database di un altro programma.
- Un backup o uno snapshot pre-aggiornamento interrotto a metà non lascia più un file dietro di sé, che poteva prendere il posto di uno snapshot valido tra i tre conservati.
- **Sovrascrivere il database dei luoghi mentre FinStats è in esecuzione non lo manda più in crash**: FinStats legge da una copia privata del file.
- Un’app di Jellyfin che segnalava ogni volta un nuovo id dispositivo poteva far crescere la memoria di FinStats per tutto il tempo in cui restava in esecuzione; ora i dispositivi non visti da cinque minuti vengono dimenticati.
- Una persona tiene al massimo 30 sessioni attive; la più vecchia viene disconnessa al successivo accesso.

### Fixed
- **La stessa serata non viene più contata due volte quando un elemento è stato rinominato.** Ricollegare la cronologia a un titolo aggiunto di nuovo in Jellyfin poteva trasformare una riproduzione importata da un tracker in un doppione di una già presente, perché il controllo che l’avrebbe intercettata veniva eseguito prima che l’id si spostasse. Ora viene rieseguito ovunque gli id vengano ricollegati. Vengono rimosse solo riproduzioni importate, mai una registrata da FinStats stesso, e quante ne sono state rimosse è scritto nel log. Il primo avvio della 2.0 ripulisce una volta tutta la cronologia da quelle già presenti.
- **Una sessione ora segue la persona com’è adesso in Jellyfin.** Un amministratore declassato da Jellyfin manteneva pieno accesso a FinStats per il resto della sua sessione di 30 giorni, e chi veniva disattivato o eliminato in Jellyfin restava connesso. Ora lo perdono appena FinStats legge gli utenti di Jellyfin (ogni 15 minuti, o subito da Impostazioni), come è sempre successo per le chiavi API.
- L’accesso scrive ciò che Jellyfin dice della persona nell’elenco utenti di FinStats, così un backup importato da un tracker non può più rendere qualcuno amministratore prima che la prima lettura degli utenti lo corregga.
- Chi viene eliminato in Jellyfin viene segnato come rimosso anche quando due letture degli utenti cadono nello stesso secondo.
- **Gli errori di download non mostrano mai la password di un client di download o la chiave API di un indexer**, che Sonarr e Radarr possono citare.
- Le riproduzioni ricollegate a un titolo rinominato durante una lettura della libreria non venivano controllate per la visione insieme fino al riavvio successivo; ora lo sono, subito.

## [1.6.5] - 2026-09-24

### Added
- **Prima di ogni aggiornamento viene fatto automaticamente un backup completo.** La prima volta che un FinStats più recente apre il tuo database, ne scrive una copia completa (tutta la tua cronologia, impostazioni comprese) in `data/pre-update-backups/` *prima* di cambiare qualsiasi cosa. Se un aggiornamento dovesse rompere qualcosa, i tuoi dati sono al sicuro: ferma FinStats, rimetti la copia al posto di `finstats.db`, avvia la versione che avevi e segnala il bug. Non si perde nulla. Vengono conservate le copie più recenti; imposta `FINSTATS_SKIP_PREUPDATE_BACKUP=1` per disattivarlo se lo spazio su disco scarseggia.

## [1.6.4] - 2026-09-24

### Added
- **Cinque nuove destinazioni per le notifiche**: Telegram, Slack, Pushover, Pushbullet ed e-mail, accanto alle destinazioni webhook, Discord, ntfy e Gotify già presenti. Ognuna chiede solo ciò che le serve (un ID chat, una chiave utente, una casella di posta), e le tre che vivono presso un unico servizio (Telegram, Pushover, Pushbullet) possono essere raggiunte solo lì, così un token non può essere inviato a un host che le imita. La posta parte su TLS fin dal primo byte o con `STARTTLS`, e non c’è una terza opzione.
- **Licenze**, in **Impostazioni**: la GPL di FinStats e ogni licenza di terze parti su cui è costruito (253 crate Rust, i due caratteri inclusi, i contorni della mappa e il database di geolocalizzazione), ogni testo per intero, così come l’ha scritto il suo progetto. Scegli un componente e la sua licenza si apre nel riquadro accanto all’elenco; un crate che include più file passa dall’uno all’altro sul posto. L’elenco è generato dai crate con cui FinStats è effettivamente costruito, quindi non si può aggiungere una dipendenza senza che la sua licenza venga registrata.

## [1.6.3] - 2026-09-24

### Fixed
- La protezione di sicurezza ora interviene anche quando Jellyfin non elenca *nessuna* libreria o *nessun* utente (una lettura rotta, mai una giornata normale), cosa che la regolazione della 1.6.2 aveva lasciato passare. Perdere alcune librerie è ancora considerato normale; perderle tutte insieme no.

## [1.6.2] - 2026-09-24

### Fixed
- La protezione dell’integrità dei dati aggiunta nella 1.6.1 era troppo zelante: una libreria che perdeva davvero la maggior parte dei suoi elementi (una manciata di clip i cui file erano spariti, per esempio) veniva scambiata per una lettura rotta, quindi FinStats si rifiutava di aggiornarla e si fermava. Ora interviene solo per una lettura *chiaramente* rotta: una libreria abbastanza grande da contare che risulta completamente vuota, o una grande ridotta quasi a nulla. Le modifiche di tutti i giorni si applicano come prima, e il caso “Jellyfin non ha restituito nulla” viene ancora intercettato.

## [1.6.1] - 2026-09-24

### Added
- **FinStats non si fida più di una lettura di Jellyfin che cancellerebbe la tua cronologia.** Se una libreria per cui FinStats conserva migliaia di elementi risulta all’improvviso vuota o quasi (un aggiornamento di Jellyfin che ha cambiato la sua API, un errore travestito da risultato vuoto), FinStats tiene i dati che ha, segna la sincronizzazione come non riuscita (e lo notifica) e si ferma in modo pulito con un messaggio chiaro invece di segnare tutto come rimosso. Riavvialo quando Jellyfin è tornato in sé, oppure imposta `FINSTATS_ALLOW_LIBRARY_SHRINK=1` se hai davvero svuotato una libreria. La stessa protezione copre l’elenco delle librerie e quello degli utenti.

### Security
- Un header `X-Forwarded-Host` inviato da un client poteva garantire per un `Origin` estraneo e far passare una scrittura cross-site oltre la protezione same-origin. Ora quell’header è considerato affidabile solo dietro un proxy fidato (`FINSTATS_TRUST_PROXY`), la stessa regola che usa già la ricerca dell’indirizzo del client. Il cookie di sessione `SameSite=Lax` era, e resta, la prima linea di difesa.
- Una destinazione di notifica personale poteva essere puntata a `0.0.0.0` o `::`, che raggiungono questa macchina. Il controllo che tiene le destinazioni personali puntate a indirizzi pubblici ora li considera locali, insieme al loopback e agli intervalli privati.

### Fixed
- La pagina **Riproduzione** non fallisce più il caricamento per una libreria che contiene una riproduzione senza metodo registrato (una riga ripristinata da un backup più vecchio della colonna); una riproduzione del genere conta semplicemente come riproduzione diretta.
- Una parola molto lunga in una ricerca o in un filtro non è più un errore del server: SQLite rifiuta un pattern `LIKE` oltre i 50.000 caratteri, quindi le parole dei filtri vengono accorciate a una lunghezza ragionevole prima della query.
- La pagina **Impostazioni** non scorre più di lato sul telefono quando una scheda riporta un’intera frase, come il risultato di un ripristino.

## [1.6.0] - 2026-09-24

Notifiche

### Added
- Scheda **Notifiche** in Impostazioni: aggiungi una destinazione, spunta ciò di cui deve essere avvisata, premi Prova. Nessuna destinazione vuol dire che non viene fatta alcuna richiesta a nulla.
- Quattro tipi di destinazione: un semplice **webhook** (JSON, documentato in `docs/api.md`, header `Authorization` facoltativo), **Discord** (un embed tramite l’URL del webhook di un canale), **ntfy** (ntfy.sh o il tuo, su un argomento) e **Gotify** (il tuo server, con un token dell’applicazione). I titoli viaggiano nel messaggio invece che in un header.
- Undici tipi di evento, ciascuno con la sua casella: viaggio impossibile, un account visto in un nuovo paese, una raffica di accessi non riusciti, un’operazione non riuscita, un backup che non è stato possibile scrivere, qualcosa che ha smesso di rispondere (Jellyfin, Sonarr, Radarr o Seerr) e quando risponde di nuovo, nuovi titoli nella libreria (raggruppati per serie e giorno, come li raggruppa la fila della panoramica), una richiesta diventata guardabile, e una riproduzione che inizia o finisce. Gli ultimi due sono disattivati per impostazione predefinita.
- Una richiesta diventata guardabile nomina il titolo, chi l’ha chiesto e quanto ha aspettato.
- Destinazioni personali: concedi **Ricevere notifiche** e qualcuno può aggiungerne una propria, a cui viene inviato solo ciò che può già vedere: le proprie richieste e i propri avvisi sempre, quelli degli altri con *Vedere l’attività di tutti*, gli indirizzi con *Vedere i dettagli di rete*. Una destinazione personale deve puntare a un indirizzo pubblico; solo un amministratore può puntarne una all’interno della rete.
- **Includi indirizzi IP e luoghi**, per destinazione e disattivato per impostazione predefinita: altrimenti un messaggio su un viaggio impossibile nomina solo i luoghi (“Oslo, Norvegia e Londra, Regno Unito”).
- **Inviati di recente**, nella stessa scheda: cosa è stato detto, a quale destinazione, e cosa ha risposto l’altra parte quando il messaggio non è arrivato. Un messaggio non riuscito viene ritentato quattro volte nell’arco di circa un’ora, e una destinazione che limita la frequenza viene attesa.
- **L’indirizzo di FinStats**, un’impostazione per gli amministratori, è ciò da cui si costruisce il link di una notifica. Vuoto vuol dire che i messaggi non portano alcun link.
- Le operazioni pianificate di Jellyfin nella pagina Server: cosa fa ciascuna a parole invece che con il nome del suo codice, e se è una di quelle pesanti. Un’operazione in corso mostra una barra di avanzamento, una percentuale e quanto manca; le altre mostrano quando verranno eseguite, quando sono state eseguite l’ultima volta e se è andata bene. Sono incluse le operazioni che Jellyfin nasconde dal proprio pannello, e nulla in FinStats può avviare o fermare un’operazione.
- Quanto manca alla fine di un’operazione viene misurato dalla velocità con cui la sua percentuale si è effettivamente mossa, rispetto alla lettura più recente abbastanza lontana da significare qualcosa, invece che dalla media dell’intera esecuzione, così un’operazione che accelera o rallenta viene descritta con il ritmo che ha adesso.
- Gli elenchi delle operazioni che FinStats legge già per altri motivi (il controllo dell’analisi della libreria ogni 5 minuti, i dettagli del server ogni 15) alimentano la stessa misura, così un’operazione in corso da un’ora di solito ha una stima nel momento in cui si apre la pagina. Per questo non viene chiesto nulla in più a Jellyfin.
- Un’operazione in corso senza ancora nulla di misurato mostra **ETA** e dei puntini che scorrono invece di un numero, e una la cui percentuale non si è mossa da mezzo minuto mostra accanto “68% da 4m”. Nulla prende il posto di una stima non misurata.
- Un elenco di tutte le versioni accanto alle novità: un clic porta a qualsiasi versione, segna quella che si sta leggendo, e su uno schermo stretto diventa una fila di serie sopra le note.

### Changed
- Le novità non si richiudono più. Ogni serie resta aperta e l’elenco delle versioni è il modo per spostarsi, invece di un gruppo aperto alla volta e un clic per raggiungere qualsiasi altro.
- **Impostazioni → Connessioni in uscita** elenca ogni destinazione di notifica accanto a Jellyfin e alle connessioni dei servizi: cos’è, il suo host, se è attiva e quando ha ricevuto l’ultimo messaggio.
- Nulla cambia nella raccolta, nella cronologia o in qualsiasi numero sullo schermo. FinStats continua soltanto a *leggere* da Jellyfin, Sonarr, Radarr e Seerr.

## [1.5.2] - 2026-09-22

### Changed
- Una lettura dell’elenco delle sessioni di Jellyfin e l’azzeramento dell’orologio che l’ha richiesta sono un’unica chiamata, quindi non possono più separarsi. Era questa la forma del guasto delle richieste a valanga trovato mentre si costruiva la 1.5.0.
- Dimostrarlo richiede una frazione di secondo invece di sette minuti contro un server reale, quindi viene verificato a ogni modifica. Le stesse richieste, la stessa cronologia, gli stessi numeri sullo schermo.

## [1.5.1] - 2026-09-22

### Fixed
- Una transcodifica viene registrata una volta, non una volta al secondo. Quando una riproduzione parte transcodificando e poi torna alla riproduzione diretta, Jellyfin lascia i dettagli della transcodifica sulla sessione; ogni lettura veniva confrontata con un valore che FinStats aveva appena sovrascritto, così ogni lettura sembrava un cambiamento e la pagina Attività si riempiva di righe “Transcodifica” con sotto “Riproduzione diretta: …”.
- Le righe già scritte vengono ripulite all’avvio: dove una sequenza di righe dice la stessa cosa, si tiene la prima. Una riproduzione che ha richiesto la transcodifica conta ancora come transcodifica.

## [1.5.0] - 2026-09-22

Monitoraggio delle sessioni via WebSocket

### Changed
- FinStats tiene aperta una connessione con Jellyfin dal momento in cui si avvia, ed è così che viene a sapere di una riproduzione. La 1.4.0 l’aveva introdotta come impostazione, disattivata per impostazione predefinita, perché desse prova di sé su server reali; ora è sempre attiva. Le stesse riproduzioni, pause e salti, calcolati dallo stesso codice.
- Nulla in riproduzione: FinStats non chiede assolutamente nulla a Jellyfin. Una serata in cui nessuno guarda non costa alcuna richiesta.
- Qualcosa in riproduzione: l’elenco delle sessioni viene letto una volta al secondo. È da lì che una pausa, un salto o un cambio di traccia prendono la loro precisione, ed è ciò che chiude una riproduzione il cui client è sparito senza salutare.
- Tutto ciò che è caricato è in pausa per tre letture di fila: le domande si fermano e la connessione se ne fa carico, mentre prima un film in pausa costava la stessa domanda ogni secondo, circa 16 MB l’ora. Una ripresa, o qualcun altro che avvia qualcosa, ottiene risposta entro circa un secondo, e il tempo in pausa non conta mai come tempo di visione.
- Ogni lettura in uscita chiede risposte compresse e le decomprime (Jellyfin, Sonarr, Radarr e Seerr comprimono tutti quando richiesto), il che toglie più della metà alle due letture più grandi: l’elenco delle sessioni durante una riproduzione e la lettura della libreria. Il database di geolocalizzazione viene ancora scaricato così com’è conservato.
- La scheda Jellyfin in Impostazioni dice quale delle due metà è in funzione, e l’indicatore in tempo reale dice se la connessione sta trasportando dati, non in quale metà ti trovi.
- Ne funziona una sola alla volta: mentre FinStats chiede, dice a Jellyfin di smettere di inviare, e gli chiede di ricominciare nel momento in cui finisce l’ultima riproduzione. La connessione resta aperta per tutto il tempo.
- Un Jellyfin che risponde all’iscrizione con il silenzio viene creduto finché risponde ai keep-alive: FinStats lo dice una volta, nel frattempo legge da sé l’elenco delle sessioni, e resta iscritto finché il primo invio non risolve la questione. “Non può inviare” e “si stava riavviando quando abbiamo chiesto” si somigliano, quindi il silenzio non è mai un verdetto.
- Il ripiego resta invariato: una connessione che si chiude, tace o si rivela incapace di parlare questo protocollo torna a chiedere a intervalli già al passaggio successivo e continua a riconnettersi, con gli intervalli **Mentre qualcuno guarda, controlla ogni** e **Quando non c’è nulla in riproduzione, controlla ogni**.

### Added
- Cosa sta facendo la raccolta, su `GET /api/status`: in quale delle quattro situazioni si trova (`idle_socket`, `playing_poll`, `paused_socket`, `fallback`), se la connessione è aperta e iscritta, quanto spesso chiede, quante letture delle sessioni sono partite nell’ultimo minuto, e quando tutto questo è cambiato l’ultima volta. La risposta arriva dalla memoria (nessuna richiesta, nessuna query), e non porta nomi, titoli o conteggi.
- Quei campi vengono confrontati tra loro a ogni passaggio, e un disaccordo che dura viene scritto nel log.
- Un unico limitatore sotto ogni lettura delle sessioni nel processo: al massimo 2 al secondo e 70 al minuto, con tutto ciò che supera ritardato, contato e scritto nel log al massimo una volta al minuto. Una visione normale resta ben al di sotto.

### Fixed
- Gli aggiornamenti in tempo reale sostituiscono davvero il polling. Quando era attivata, la 1.4.0 chiedeva ancora l’elenco delle sessioni ogni cinque secondi e riapriva la connessione ogni 20–40 secondi, perché giudicava la connessione da quanto di recente fosse arrivato un elenco, e Jellyfin ne invia uno quando qualcosa *cambia* e nient’altro nel frattempo. Ora la connessione è viva se Jellyfin risponde, verificato con un keep-alive circa due volte al minuto.
- Un elenco delle sessioni arrivato mentre FinStats era tra un passaggio e l’altro veniva scartato, supponendo che un altro arrivasse un secondo e mezzo dopo. Ora viene conservato e usato.
- **Ultimo controllo** in Impostazioni, e la scheda Connessioni in uscita, non restano più indietro mentre la connessione tace: mostrano la risposta di Jellyfin a un keep-alive.

### Removed
- L’impostazione **Lascia che Jellyfin invii ciò che è in riproduzione** in Impostazioni → Raccolta, e `live_socket` dall’API delle impostazioni.
- `socket_enabled` dallo stato della raccolta: non esiste più “disattivato”.
- `last_reconcile_at` dallo stato della raccolta, sostituito da `socket_live`.

## [1.4.1] - 2026-09-22

### Changed
- Seerr viene letto per intero solo quando qualcosa è cambiato: ogni cinque minuti FinStats chiede una riga, la richiesta modificata più di recente, e il passaggio finisce lì se la conosce già. Circa un kilobyte contro circa sessanta per una pagina da cinquanta. Le richieste ancora in corso vengono riesaminate ogni quarto d’ora, tutto viene elencato una volta al giorno, e una nuova richiesta compare comunque entro cinque minuti.
- Le code di Sonarr e Radarr vengono lette ogni cinque secondi mentre una pagina le mostra, ogni minuto mentre c’è qualcosa in coda e ogni cinque minuti quando è vuota, invece che ogni minuto a qualsiasi ora. Le aggiornano subito l’apertura della pagina, il collegamento di un servizio o una nuova richiesta in Seerr.

## [1.4.0] - 2026-09-21

Sessioni via WebSocket facoltative, e ovunque FinStats possa collegarsi

### Added
- **Lascia che Jellyfin invii ciò che è in riproduzione** in Impostazioni → Raccolta, disattivato finché non lo attivi: un’unica connessione trasporta l’elenco delle sessioni, e circa 17.000 richieste al giorno diventano quasi nessuna. Riproduzioni, pause, salti e gruppi vengono calcolati esattamente dallo stesso codice di prima. Mentre è attivo FinStats fa ancora una normale richiesta al minuto *mentre qualcosa è in riproduzione*, e nessuna quando non c’è nulla. Una connessione che si chiude, tace per quindici secondi o si rivela incapace di parlare questo protocollo torna al timer al passaggio successivo e continua a riconnettersi.
- **Connessioni in uscita**, una scheda in Impostazioni per gli amministratori di Jellyfin: ogni destinazione che FinStats può raggiungere (il tuo Jellyfin, il servizio “qual è il mio IP”, il database di DB-IP, e ogni Sonarr, Radarr o Seerr), con a cosa serve, se è attiva e quando ha risposto l’ultima volta. Costruita da ciò che FinStats sa già, e mostra solo indirizzi: mai una chiave, mai un percorso di base.
- **Cerca ora** in Impostazioni → Rete domestica, per il giorno in cui il tuo indirizzo pubblico cambia.

### Changed
- La ricerca “qual è il mio IP” viene eseguita una volta, la prima volta che FinStats ha bisogno di un indirizzo, invece che ogni quindici minuti.
- La scheda Jellyfin in Impostazioni dice come viene informata la raccolta e, quando la connessione è attiva ma non trasporta dati, perché.

## [1.3.0] - 2026-09-20

Sonarr, Radarr e Seerr, in un’unica pagina

### Added
- **Connessioni** in Impostazioni, per gli amministratori di Jellyfin: **Sonarr**, **Radarr** e **Seerr** (o Jellyseerr/Overseerr), anche più di uno per tipo, ciascuno verificato prima di essere salvato. Il tuo client di download non richiede alcuna configurazione, perché Sonarr e Radarr ci parlano già e FinStats legge ciò che sanno.
- **Richieste**: chi ha chiesto cosa, quanto ci è voluto perché arrivasse, e se poi l’ha mai guardato. Riquadri per i titoli in attesa e l’attesa tipica, un grafico di come è cambiata mese per mese, ed elenchi di ciò che è arrivato settimane fa e non è mai stato riprodotto, e di chi chiede di più.
- **Prossime uscite**: nuovi episodi e uscite di film da Sonarr e Radarr come un’agenda per giorno, con l’indicazione se *tu* guardi quella serie e, per chi può vedere tutti, chi altro la guarda. Una serie conta come seguita quando qualcuno ne ha riprodotto un episodio negli ultimi quattro mesi. Una fila “In arrivo” nella panoramica, una per profilo per le serie che quella persona guarda, e cosa viene dopo per un titolo sulla sua pagina.
- **Download**: la coda in tempo reale di ogni Sonarr e Radarr (cos’è, a che punto è, quanto va veloce, cosa è andato storto nell’importazione), con accanto la persona che l’ha chiesto. Un season pack è una riga sola per quanti episodi contenga, e ciò che richiede attenzione sta in cima. Sotto, ciò che è arrivato in una settimana, un mese o un anno per indexer, qualità e client di download, e quali download sono falliti. La panoramica mostra lo stesso elenco, accorciato.
- **Vedere cosa si sta scaricando**, un nuovo permesso. Senza, le persone vedono comunque a che punto è la *propria* richiesta e quanto manca, e nient’altro: niente nomi delle release, niente velocità, niente altri download. Le richieste degli altri richiedono *Vedere l’attività di tutti*.

### Changed
- FinStats invia a questi servizi solo `GET` e nient’altro: non contiene codice che possa approvare una richiesta, avviare una ricerca, o aggiungere, mettere in pausa o rimuovere un download.
- Le loro chiavi API sono conservate nel database di FinStats, mai rimandate al browser, mai scritte in un log e mai incluse in un backup. Nessun reindirizzamento viene seguito, quindi una chiave non può finire in un posto che non hai indicato, e i certificati vengono verificati a meno che tu non lo disattivi per una connessione.

## [1.2.2] - 2026-09-20

### Added
- Lingue dell’audio e dei sottotitoli su ogni film ed episodio (“Audio: giapponese · inglese”).
- Fin dove arriva un doppiaggio: una serie o una stagione dice quanti dei suoi episodi hanno ciascuna lingua (“Inglese: 13 di 26 episodi”), e l’elenco degli episodi ha una colonna **Audio**.
- **Lingue dell’audio** e **Lingue dei sottotitoli** nelle pagine delle librerie: quanti file si possono riprodurre in ciascuna.
- Dopo questo aggiornamento FinStats rilegge la libreria una volta per raccogliere le lingue. Non dice mai “doppiato”, perché Jellyfin non fornisce la lingua originale di un titolo, e una traccia senza tag di lingua viene elencata come “Sconosciuta”.

## [1.2.1] - 2026-09-20

### Changed
- Le immagini del README sono state rifatte con questa versione, con una nuova della pagina **Sicurezza**. Nell’app non cambia nulla.

## [1.2.0] - 2026-09-20

Da dove guardano le persone, su una mappa

### Added
- **Pagina Sicurezza**, per chi può vedere i dettagli di rete e l’attività di tutti: una mappa del mondo con un punto per ogni luogo da cui guardano i tuoi utenti, grande in proporzione a quanto vi accade. La casa è verde, i luoghi fuori casa sono viola, gli stream in corso pulsano e gli accessi non riusciti dall’esterno sono rossi. Trascina per spostarti, zooma con i pulsanti, i tasti o Ctrl + rotellina, e i punti sovrapposti si uniscono finché non ingrandisci. Sotto, ogni luogo con le sue persone, riproduzioni, tempo di visione e accessi, e i paesi per riproduzioni.
- **Viaggio impossibile**: un avviso quando un account viene visto in due luoghi che nessun volo collega, o in due luoghi lontani nello stesso momento, con entrambi gli avvistamenti, la distanza, il tempo tra i due e la velocità che sarebbe servita. **Mostra sulla mappa** disegna il viaggio.
- **Nuovo paese**: la prima volta che qualcuno riproduce o accede da un paese in cui non era mai stato visto.
- Risolvi un avviso con una nota, oppure silenzia una coppia di luoghi per quella persona, per una VPN o un telefono il cui operatore ha sede nella capitale. Gli avvisi trovati nella cronologia passata, dopo un’importazione o su un database nuovo, vengono archiviati come risolti invece di inondare l’elenco. Gli avvisi e ciò che hai deciso su di essi fanno parte dei backup.
- I luoghi vengono da un database delle città letto localmente: nessun indirizzo viene mai inviato da nessuna parte, e la mappa è disegnata da contorni inclusi in FinStats invece che da un servizio di mappe. Scarica il database gratuito di DB-IP dalla pagina Sicurezza o da **Impostazioni → Sicurezza**, lascia che FinStats lo aggiorni ogni mese (disattivato finché non lo attivi), oppure metti il tuo `.mmdb` (DB-IP, MaxMind GeoLite2-City) in `data/geoip/`. `FINSTATS_GEOIP_DB` indica un file altrove.
- **Impostazioni → Sicurezza**: il database in uso, l’aggiornamento mensile, e quanto veloci (900 km/h) e quanto distanti (500 km) devono essere due avvistamenti per contare come viaggio impossibile. I database delle città spesso sbagliano di qualche centinaio di chilometri, ed è a questo che serve la distanza.

## [1.1.3] - 2026-09-20

### Added
- Ctrl+Spazio cerca anche tra cast e troupe di tutto ciò che c’è nella tua libreria, in **Cast e troupe**, con la foto, se recitano o dirigono, e in quanti dei tuoi titoli compaiono; sceglierne uno apre la sua pagina. Tollerante quanto la ricerca dei titoli (parole in qualsiasi ordine, accenti ignorati, un errore di battitura concesso), e quando più persone corrispondono allo stesso modo, viene prima quella presente in più titoli.

## [1.1.2] - 2026-09-20

### Added
- **Aggiunti di recente** nella panoramica, sopra il grafico Attività: i 30 arrivi più recenti come una fila di locandine che puoi scorrere di lato, con le frecce sul computer. Ognuno dice quando è arrivato e cos’è (un film, un album, o “Stagione 4 · 3 episodi”), e un singolo nuovo episodio dice quale, con la locandina della stagione quando ne ha una.
- **Maiusc + rotellina del mouse** e i tasti freccia spostano di una locandina, Pagina su e Pagina giù di una schermata, Home e Fine vanno a un estremo o all’altro, e lo scorrimento con il dito funziona. La rotellina da sola scorre ancora la pagina.
- I nuovi episodi della stessa serie arrivati nello stesso giorno condividono una voce, e un’intera serie aggiunta in una volta è una voce sola (“3 stagioni · 60 episodi”), così una grande importazione non spinge tutto il resto fuori dall’elenco.
- La fila ignora il periodo e la persona scelti nella panoramica, perché riguarda la libreria e non le riproduzioni, e si nasconde quando la libreria è vuota.

## [1.1.1] - 2026-09-20

### Added
- Le pagine si aprono subito: un attimo dopo che FinStats si è caricato, scarica in silenzio ciò che probabilmente aprirai dopo (la panoramica, il tuo profilo e la linea del tempo, Attività, Utenti, Librerie e ogni libreria, Riproduzione, le pagine del server, il riepilogo e le persone più attive), così una pagina, quando la apri, compare all’istante e si aggiorna comunque dietro le quinte.
- Film, serie e membri del cast vengono scaricati solo quando servono, visto che sono migliaia: fermare il puntatore su un link, toccarlo o raggiungerlo con la tastiera scarica quella sola pagina.
- Aspetta che la pagina aperta abbia finito di caricarsi, lavora una pagina alla volta mentre il browser è inattivo, si ferma in una scheda in background, non fa nulla con il risparmio dati o una connessione molto lenta, e viene dimenticato quando esci.
- Un piccolo link **Repo** con il logo di GitHub nella barra di stato, accanto alla versione.

### Fixed
- La **Linea del tempo** dimenticava quali librerie erano state escluse quando la pagina veniva ricaricata o aperta da un segnalibro, anche se la scelta era nell’indirizzo.

## [1.1.0] - 2026-09-20

Una linea del tempo di tutto ciò che hai guardato

### Added
- **Linea del tempo**, accanto a **Panoramica** su ogni profilo: ciò che hai guardato come un unico percorso da oggi fino alla prima riproduzione che FinStats conosce, con una serata di episodi raccolta in una sola tappa. Ogni tappa è una locandina con cosa è stato visto e quando (“Stagione 2 · Episodi 3-6”, un film e se ha richiesto più di una seduta, o un album e quanti brani), e apre il titolo.
- Il percorso segue la larghezza della finestra: tre tappe per riga su uno schermo largo, due su uno più stretto, una riga su due che va all’indietro con una curva che la unisce alla successiva, fino a un’unica linea dritta sul telefono. La prima tappa di ogni mese porta il nome del mese.
- Le tappe più vecchie si caricano da sole mentre scorri, fino all’inizio della cronologia.
- Gli interruttori **Librerie** sopra il percorso escludono ciò che non vuoi vedere, come la musica, e la scelta fa parte dell’indirizzo.
- Ciò che è ancora in riproduzione dice **In riproduzione**.
- Vedi la tua linea del tempo; quelle degli altri richiedono il permesso “Vedere l’attività di tutti”.

## [1.0.5] - 2026-09-20

### Added
- Protezione dal downgrade: il database ricorda la versione più recente di FinStats che lo ha aperto, e una versione più vecchia si rifiuta di avviarsi su di esso invece di lavorare in silenzio su dati che non comprende del tutto. Il messaggio dice cosa fare. Le versioni fino alla 1.0.4 sono uscite prima che esistesse questo controllo.

## [1.0.4] - 2026-09-20

### Fixed
- Nei dettagli di una riproduzione, il pulsante di copia accanto a un **ID dispositivo** lungo era finito su una riga a sé.
- Nella pagina Riproduzione l’ultima colonna di **Quali client transcodificano** veniva tagliata (“Transc…”); ora le intestazioni di due parole vanno a capo.

### Changed
- Gli screenshot del README sono di nuovo aggiornati; mostravano ancora la versione 0.5.0.

## [1.0.3] - 2026-09-20

### Added
- La pagina di un episodio mostra l’immagine di quell’episodio, quella che Jellyfin mostra nel suo elenco degli episodi, invece della locandina della serie. Un episodio per cui Jellyfin non ha un’immagine mantiene la locandina.

## [1.0.2] - 2026-09-20

### Fixed
- Nelle **Novità**, i gruppi v0.9 e v0.10 non avevano un titolo accanto alla versione. Ora il titolo di un gruppo è sempre la prima frase di ciò di cui parlava la sua prima versione, così una lunga introduzione non viene tagliata a metà frase.

## [1.0.1] - 2026-09-20

### Fixed
- L’immagine pubblicata non si avviava su una macchina appena installata: “unable to open database file: /data/finstats.db”, dopo essere rimasta bloccata per mezzo minuto. Docker crea una cartella `data` mancante come root e FinStats gira come utente normale. Ora il container rende sua la cartella e poi passa a quell’utente, così `docker run` funziona al primo colpo. Per file di proprietà di un utente diverso da 1000, imposta `PUID` e `PGID`; `--user` funziona come prima.
- Una cartella dei dati davvero non scrivibile viene segnalata subito, con il comando che la sistema, invece che 30 secondi dopo come errore del database.

## [1.0.0] - 2026-09-20

Pronto per tutti

### Added
- Un’immagine pronta, ora su `ghcr.io/finstats/finstats`, per macchine Intel/AMD e ARM a 64 bit (un Raspberry Pi 4 o 5 va bene): `:latest` è la versione più recente, un tag di versione resta dov’è, e `:edge` è la versione di sviluppo.
- Ogni versione compare nella pagina delle release del progetto su GitHub con queste note.

### Changed
- I numeri di versione ora significano ciò che dicono: gli aggiornamenti 1.x non romperanno i tuoi dati, i tuoi backup o le tue impostazioni.
- Il file Docker Compose e tutte le istruzioni usano l’immagine pubblicata. Un `finstats:latest` compilato da sé richiede solo di cambiare il nome dell’immagine; la cartella `data` passa intatta.

## [0.10.2] - 2026-09-20

### Fixed
- Sul telefono, Impostazioni e la pagina di una serie potevano scorrere di lato: un’etichetta pensata solo per i lettori di schermo usciva dalla sua tabella, e gli elenchi degli episodi e la fila del cast facevano lo stesso. Ora scorrono al loro interno.
- Il riepilogo non scorre più di lato quando la parola di sfondo di un capitolo è più larga della pagina.
- Una lunga riga “sta guardando su…” sotto **In riproduzione** finisce con i puntini di sospensione invece di essere tagliata a metà lettera.
- La locandina davanti a ogni serie su un profilo era un secondo link senza nome per i lettori di schermo; ora è nascosta a loro e il link è il titolo accanto.

## [0.10.1] - 2026-09-20

### Fixed
- Scaricare un backup falliva nel browser (“the source file could not be read” in Firefox) senza nulla nella pagina che spiegasse perché: FinStats comprimeva una seconda volta un backup, che è già compresso, e il trasferimento doppiamente compresso si interrompeva prima della fine. Ora i backup vengono inviati così come sono, con la dimensione indicata in anticipo, così il browser può mostrare l’avanzamento reale.

## [0.10.0] - 2026-09-20

Backup, e trasloco su una nuova installazione

### Added
- Un backup ogni settimana, conservati i cinque più recenti: ogni riproduzione con la sua linea del tempo di pause e salti, i segni di visto, i permessi, gli indirizzi di casa, il registro del server e le tue impostazioni, in un unico piccolo file, poche centinaia di KB per migliaia di riproduzioni. **Impostazioni → Backup** li elenca, ciascuno con un pulsante **Scarica**, ne crea uno su richiesta e imposta ogni quanto e quanti. Stanno nella cartella `backups` della tua directory dei dati. Un backup non contiene la chiave API di Jellyfin né sessioni di accesso, ma è una cronologia di visione completa con indirizzi IP, quindi tienilo in un posto riservato.
- Il ripristino unisce invece di sovrascrivere: le riproduzioni già presenti vengono saltate, quindi è sicuro farlo due volte o su un’istanza in funzione da un po’. Togli la spunta da “Ripristina anche impostazioni e permessi” per la sola cronologia. I backup funzionano tra versioni in entrambe le direzioni. Da un terminale: `finstats backup` e `finstats restore <file>`.

### Changed
- La raccolta controlla ogni secondo mentre qualcuno guarda e ogni 5 secondi quando nessuno guarda, invece di ogni 5 secondi in ogni caso, così pause, salti e cambi di traccia vengono registrati al secondo. Entrambi gli intervalli stanno in **Impostazioni → Raccolta**; un vecchio intervallo modificato va reimpostato.

### Fixed
- Un indirizzo IP nei dettagli di una riproduzione andava a capo a metà indirizzo. Il segno Locale/Remoto passa sotto quando non c’è spazio.

## [0.9.1] - 2026-09-20

### Fixed
- Le persone a casa venivano mostrate come remote. Un dispositivo sulla tua rete che raggiunge Jellyfin tramite il suo nome pubblico, come un reverse proxy o un dominio, arriva con l’IP pubblico della casa, e solo gli indirizzi privati contavano come locali. Ora FinStats impara l’indirizzo pubblico e conta quelle riproduzioni come locali per tutta la cronologia, e ricorda gli indirizzi precedenti.

### Added
- **Impostazioni → Rete domestica**: gli indirizzi che FinStats considera di casa, con la possibilità di aggiungere i tuoi (un indirizzo precedente, una seconda casa, l’uscita di una VPN) o di disattivare la ricerca. La ricerca chiede a un semplice servizio “qual è il mio IP” ogni 15 minuti, provandone diversi (Amazon, Cloudflare e altri) perché i DNS che bloccano la pubblicità, come Pi-hole, spesso li bloccano, e uno di essi non ha affatto bisogno del DNS. La richiesta non contiene nulla su di te o sul tuo server, e con l’interruttore spento FinStats non parla con nient’altro che Jellyfin. `FINSTATS_PUBLIC_IP_URL` imposta un servizio tuo.

## [0.9.0] - 2026-09-20

Ogni tabella, nell’ordine che vuoi

### Added
- Ogni tabella si ordina: fai clic su un’intestazione per ordinare in base a essa, di nuovo per invertire, una terza volta per l’ordine originale. Il significato delle celle viene compreso, così “3g 2h” si ordina come tempo e “1,4 GB” come dimensione, e le celle vuote vanno in fondo. Vale per l’elenco degli utenti, gli spettatori, gli episodi, i dispositivi, i plugin, le operazioni pianificate, la vista tabella di ogni grafico, la matrice dei permessi e gli elenchi a barre (codec, risoluzioni, client e il resto), che ricevono una sottile intestazione propria.
- Attività e il registro del server ordinano su tutte le loro pagine invece che sulle sole righe sullo schermo, e l’ordine resta nell’indirizzo, così sopravvive a un ricaricamento o a un link condiviso.
- Le tabelle con dieci righe o più hanno un piccolo campo di filtro sopra.

## [0.8.2] - 2026-09-20

### Changed
- Il selettore dell’anno del riepilogo è lo stesso interruttore di ogni altro selettore di periodo in FinStats (7g · 30g · 90g …).

## [0.8.1] - 2026-09-19

### Added
- Pagine per attori e registi, da “Le persone più viste” del riepilogo o dalla nuova fila **Cast e troupe** su ogni film, serie o episodio: tutto ciò in cui compaiono sul tuo server, quanto è stato visto e da chi, e cosa aspetta ancora. Seguono il periodo che scegli, chi può vedere solo le proprie statistiche vi vede solo ciò che ha guardato, ed Esc ti riporta indietro.

## [0.8.0] - 2026-09-19

Il riepilogo ha un nuovo aspetto, e più cose da dire

### Added
- **Le persone più viste**: gli attori con cui hai passato più tempo e i registi dietro ciò che hai guardato, come una fila di ritratti. Cast e troupe vengono letti insieme alla libreria e compaiono dopo la prima lettura della libreria successiva a questo aggiornamento.
- **Il tuo anno in giorni**: ogni giorno dell’anno come un quadratino, più luminoso quanto più hai guardato, con sopra il tuo record di giorni di fila e la giornata più intensa. Passa sopra un giorno, o percorri il calendario con i tasti freccia, per vedere cosa conteneva.
- **In cifre**: riproduzioni, tempo di visione, titoli diversi, giorni di visione, record di giorni di fila e quanto è stato **rivisto**, più come l’anno si divide tra episodi, film e musica.
- Il capitolo dei generi dice quanti generi hai toccato e quanta parte si è preso il primo.

### Changed
- Un nuovo design: si apre sul tuo anno “rivissuto”, il titolo accanto alle locandine che lo hanno riempito, sopra una forma d’onda dell’anno con una barra per settimana e la più alta illuminata, e scegli l’anno subito sopra. Ogni capitolo ha un titolo con la sua parola chiave in viola, una frase che porta i numeri, e la parola del capitolo grande e tenue sullo sfondo. Finisce come finiscono i film, con i titoli di coda: interpretato da, diretto da, proiettato su, durata.
- Ore, giorni della settimana e mesi condividono un unico capitolo **Ritmi di visione** con un interruttore tra loro, e il titolo segue ciò che stai guardando (“Il venerdì si prende la corona”).

### Fixed
- **In riproduzione** nella panoramica restava vuoto quando nessuno guardava, così l’intestazione sembrava appartenere ai filtri sotto. È tornato il suo riquadro “Nulla in riproduzione al momento”.

## [0.7.10] - 2026-09-19

### Added
- Immagini delle librerie: ogni scheda di libreria, e la pagina della libreria stessa, mostra l’immagine che Jellyfin usa per essa, tramite il proxy delle immagini di FinStats come le locandine. Una libreria senza immagine mantiene la sua icona.

## [0.7.9] - 2026-09-19

### Changed
- La scheda Generi si apre di nuovo come **Elenco**, con il radar come seconda opzione dell’interruttore. Una scelta già fatta viene mantenuta.

## [0.7.8] - 2026-09-19

### Added
- Una vista radar per la scheda Generi, nella panoramica e sui profili: un raggio per genere, si passa da **Radar** a **Elenco** nell’angolo della scheda, e la scelta viene ricordata. Passare sopra un punto, o usare i tasti freccia, mostra il tempo di visione e la quota di quel genere.

## [0.7.7] - 2026-09-19

### Fixed
- I film che appartengono a una collezione venivano mostrati come “Non più nella libreria”. Con *group movies into collections* attivo, Jellyfin consegna la collezione invece dei film che contiene, quindi FinStats non li vedeva mai; su un server questo nascondeva 556 dei 1.238 elementi della libreria Film. Ora FinStats chiede i film stessi, e la lettura avviata da questo aggiornamento li riporta e ricollega le loro riproduzioni.

## [0.7.6] - 2026-09-19

### Changed
- Sui profili la scheda **Serie** è scesa appena sopra Generi, così un lungo elenco di serie non spinge più fuori vista il tempo di visione, il grafico dell’attività e i titoli principali. I giorni di fila restano in cima, e una serie che hai aperto resta aperta quando cambi il periodo.

## [0.7.5] - 2026-09-19

### Changed
- La ricerca confronta parola per parola, in qualsiasi ordine, con qualsiasi cosa in mezzo, invece di richiedere la frase esatta, così “alya hides” trova *Alya Sometimes Hides Her Feelings in Russian*. Maiuscole, accenti e punteggiatura non contano, un “The” iniziale viene ignorato, e un errore di battitura viene perdonato (“mentalsit” trova *The Mentalist*); le parole di tre lettere o meno vanno comunque scritte giuste. I risultati migliori vengono prima, e la musica si trova anche per artista.
- Le caselle di ricerca di Attività e del registro del server confrontano parola per parola su tutte le loro colonne: “alya opera” trova le riproduzioni di quella serie in Opera.

## [0.7.4] - 2026-09-19

### Changed
- Le novità sono raggruppate per serie: ogni serie `0.x` è un gruppo richiudibile con le sue versioni, con l’argomento della serie, quante versioni contiene e quando. Solo la più recente parte aperta, e un gruppo chiuso mostra comunque quale versione stai usando.

## [0.7.3] - 2026-09-19

### Fixed
- L’orologio di In riproduzione esitava ogni pochi secondi: anche l’aggiornamento ogni 5 secondi lo ridisegnava, così ogni volta che il server era avanti di una frazione il numero si muoveva in anticipo e poi restava fermo per un secondo. Ora lo muove solo il battito di ogni secondo, esattamente di un secondo (misurati da 999 a 1001 ms di distanza), e l’aggiornamento corregge ancora dopo una pausa, un salto o una deriva reale, cadendo sul battito.

## [0.7.2] - 2026-09-19

### Added
- Visione di gruppo, in tempo reale: ogni scheda di In riproduzione dice “Con maria” mentre più persone guardano la stessa cosa insieme, laddove prima un gruppo veniva riconosciuto solo a riproduzioni finite. In tempo reale, le persone contano come insieme quando sono sullo stesso titolo e hanno iniziato entro la finestra del gruppo oppure sono quasi alla stessa posizione, cosa che funziona anche se FinStats è stato riavviato a metà stream.

## [0.7.1] - 2026-09-19

### Changed
- In riproduzione conta ogni secondo: l’orologio scatta e la barra scorre fluida nel mezzo, invece di saltare ogni volta che si interroga il server. Il server viene ancora interrogato ogni 5 secondi e resta la fonte di verità: la visualizzazione si risincronizza a una pausa, a un salto o quando il server è avanti, ma non per una posizione vecchia solo di qualche secondo, visto che i client riferiscono più o meno ogni dieci secondi. Gli stream in pausa restano fermi.

## [0.7.0] - 2026-09-19

Chi guarda insieme

### Added
- Visione di gruppo: quando persone diverse avviano lo stesso titolo nello stesso momento, FinStats la conta come visione insieme. Una scheda **Visti insieme** nella panoramica mostra i gruppi, i titoli che condividono e le loro ore insieme, ogni profilo mostra con chi quella persona guarda di più, le riproduzioni condivise portano un piccolo segno di persone in Attività, e i dettagli della riproduzione dicono con chi è stata vista. Funziona su tutta la tua cronologia, riproduzioni importate comprese.
- Un’impostazione per quanto vicini devono essere gli avvii, 60 secondi per impostazione predefinita: circa un terzo delle vere sessioni di gruppo inizia a una distanza tra 6 e 60 secondi. Le persone devono anche continuare a guardare insieme per un paio di minuti, così due persone che aprono lo stesso episodio per caso non sono un gruppo.

### Changed
- La ricerca si apre con **Ctrl + Spazio** invece di Ctrl/⌘ + K.

## [0.6.1] - 2026-09-19

### Added
- Gli amministratori possono aprire il riepilogo di un’altra persona, in cima alla pagina Riepilogo o con **Apri il riepilogo** sul suo profilo. Allora parla di quella persona per nome invece che dando del tu.

### Changed
- Tutti gli altri vedono ancora solo il proprio riepilogo, qualunque permesso abbiano, e non c’è ancora un riepilogo dell’intero server.

## [0.6.0] - 2026-09-19

Il tuo profilo, e a che punto sei di ogni serie

### Added
- Avanzamento delle serie sui profili: ogni serie che hai toccato come una barra con un segmento per episodio (visto, iniziato o non ancora), apribile stagione per stagione, ordinata per quanto sei vicino a finirla, con viste separate per le serie finite e per tutto.
- **Segna come visto** sul tuo profilo, per episodio, stagione o serie, per ciò che hai guardato mentre nulla registrava. Vengono usati anche i segni di riprodotto di Jellyfin. I segni restano in FinStats e non cambiano mai nulla in Jellyfin; a una riproduzione registrata non si può togliere il segno, ai tuoi segni sì.
- Giorni di fila sui profili: il tuo record di giorni consecutivi con una riproduzione, i giorni di fila in corso, e in quanti giorni hai guardato qualcosa.
- **Il mio profilo** nella barra laterale.
- Attività mostra dove si è fermata la riproduzione (per esempio `44:15 / 45:00`) sotto la barra di avanzamento, e la data e l’ora sotto “3 h fa”. Le riproduzioni importate da Jellystat non hanno una posizione di arresto, perché Jellystat non ne ha mai registrata una.

### Changed
- Gli episodi mancanti o non ancora andati in onda non vengono più trattati come parte della tua libreria. Jellyfin li elenca come segnaposto senza file, e prima contavano nei totali degli episodi.

## [0.5.1] - 2026-09-19

### Added
- Un logo: un pulsante play tagliato in tre barre di grafico, nella barra laterale e nelle schermate di accesso, come SVG nitido con una `favicon.ico` in più dimensioni per i browser e le barre dei segnalibri che ne vogliono una, più un’icona per la schermata Home di telefoni e tablet.

## [0.5.0] - 2026-09-19

Decidi tu chi vede cosa

### Added
- Permessi in Impostazioni → Accesso, per tutti insieme o persona per persona: **vedere l’attività di tutti** (statistiche e cronologia degli altri, la pagina Utenti, ogni stream in corso), **vedere i dettagli di rete** (indirizzi IP, id dei dispositivi, locale o remoto), **vedere il server** (la pagina Server, il registro del server, gli accessi non riusciti, i percorsi dei file) e **gestire FinStats** (impostazioni, operazioni, importazione, eliminazione di riproduzioni).
- Accesso per singola persona, per far entrare poche persone senza aprire FinStats a ogni utente di Jellyfin.
- Le modifiche si applicano subito, compreso il ritiro dell’accesso, e tutto viene imposto dal server invece che nascondendo pulsanti.

### Changed
- Gli amministratori di Jellyfin hanno ancora tutto e sono gli unici che possono cambiare i permessi; chi può gestire FinStats non può concedere nulla, né a sé né ad altri.
- Il riepilogo annuale resta personale qualunque cosa venga concessa.
- Per le installazioni esistenti non cambia nulla finché non concedi qualcosa.

## [0.4.5] - 2026-09-19

### Changed
- FinStats è distribuito con licenza **GNU General Public License v3.0**. Nessuna versione precedente è mai stata pubblicata, quindi nulla è uscito con condizioni diverse. Le licenze dei caratteri inclusi sono distribuite insieme a loro.

## [0.4.4] - 2026-09-19

### Added
- Esc torna indietro: su una libreria, un utente o un titolo aperti ti riporta da dove venivi, con l’elenco esattamente come l’avevi lasciato. Prima si chiude qualsiasi cosa aperta (dettagli di una riproduzione, ricerca, un menu a tendina, il menu mobile), ed Esc viene lasciato stare mentre scrivi o usi un grafico con la tastiera.

## [0.4.3] - 2026-09-19

### Changed
- FinStats ricorda cosa ha mostrato l’ultima volta ogni pagina: tornare a un profilo, una libreria o un periodo che hai già guardato lo mostra subito, foto del profilo compresa, mentre i numeri aggiornati si caricano dietro, e la pagina si ridisegna solo se qualcosa è cambiato. Ciò che viene ricordato vive nella scheda del browser e viene scartato quando esci o cambi qualcosa.
- I segnaposto di caricamento compaiono solo quando il caricamento è davvero lento. Prima ogni pagina li mostrava per almeno un terzo di secondo.

### Fixed
- Le librerie eliminate in Jellyfin che non hanno mai avuto una riproduzione non vengono più elencate. Una libreria eliminata con una cronologia viene ancora mostrata, segnata come rimossa.

## [0.4.2] - 2026-09-19

### Fixed
- Nella pagina Utenti il tag **Amministratore** stava su una riga a sé e rendeva quella riga più alta delle altre. Ora i tag stanno accanto al nome.

## [0.4.1] - 2026-09-19

### Fixed
- La parola “null” non compare più sotto il titolo della panoramica fuori da dicembre e gennaio, quando non c’è un banner del riepilogo da mostrare.

## [0.4.0] - 2026-09-19

Guarda cosa è cambiato senza lasciare l’app

### Added
- Scheda **Novità** che mostra ogni versione, con quella in uso evidenziata.
- La versione nella barra di stato porta lì, e la scheda mostra un punto dopo un aggiornamento finché non l’hai guardata.

### Changed
- Ricollegare la cronologia agli elementi rinominati è molto più rapido: da 18 secondi a un terzo di secondo con 666 titoli orfani, così l’avvio resta istantaneo man mano che la cronologia cresce.

## [0.3.3] - 2026-09-19

### Changed
- La libreria viene riletta subito dopo che l’operazione **Scan Media Library** di Jellyfin finisce, mai con un timer separato e mai mentre un’analisi è in corso. FinStats è sempre stato di sola lettura verso Jellyfin, e non vi avvia mai un’analisi.
- Una lettura settimanale di sicurezza copre i server che si affidano al monitoraggio in tempo reale, dove l’operazione di analisi potrebbe non essere mai eseguita.
- Il vecchio intervallo ora è solo un ripiego, per quando l’opzione di seguire l’analisi è disattivata o il server non riporta alcuna operazione di analisi.

### Added
- Interruttore **Segui l’analisi della libreria di Jellyfin** in Impostazioni → Raccolta, attivo per impostazione predefinita.

## [0.3.2] - 2026-09-19

### Fixed
- La cronologia sopravvive ai cambi di nome. Jellyfin dà a un file rinominato un nuovo id, cosa che lasciava le sue riproduzioni precedenti bloccate sotto il nome grezzo della cartella (`Title (2010) [tmdbid-…] [imdbid-…]`) senza locandina. Ora quelle riproduzioni vengono ricollegate alla voce attuale della libreria: tramite id TMDB/IMDb/TVDB quando il vecchio nome ne contiene uno, altrimenti tramite titolo ripulito e anno, e gli episodi tramite serie più numero di stagione e di episodio. Una corrispondenza deve essere univoca, quindi i titoli davvero eliminati restano come sono.
- I canali TV in diretta importati da Jellystat venivano contati come film, perché Jellystat non registra il tipo di elemento. Ora vengono riconosciuti come TV in diretta, e le nuove riproduzioni di TV in diretta vengono registrate con il tipo che dà Jellyfin.

### Changed
- La TV in diretta è esclusa del tutto dal riepilogo: classifiche, totali, personalità, record e posizione.

### Added
- `finstats relink` per ricollegare la cronologia su richiesta. Viene eseguito anche all’avvio e dopo ogni lettura della libreria.

## [0.3.1] - 2026-09-19

### Changed
- Il riepilogo è strettamente personale: tutti, amministratori compresi, vedono solo il proprio, e la vista “Tutti”, il selettore di utenti e il capitolo del server non ci sono più.
- Il riepilogo si apre sull’anno “pronto”: l’anno in corso durante dicembre, altrimenti quello precedente. Gli altri anni e “Ultimi 12 mesi” restano a un clic di distanza.

## [0.3.0] - 2026-09-19

Il tuo anno in rassegna

### Added
- Scheda **Riepilogo**, una storia a scorrimento di un anno: ore guardate, serie, film, brani e generi principali, e una personalità da spettatore (nottambulo, mattiniero, guerriero del weekend, maratoneta di serie, cinefilo, melomane, abitudinario) con grafici per ora del giorno e giorno della settimana.
- L’anno mese per mese, con il titolo che ha segnato ogni mese.
- Record: la giornata più intensa, la maratona più lunga, il record di giorni di fila, la riproduzione singola più lunga, il titolo più rivisto, la prima riproduzione dell’anno e il titolo più vecchio guardato.
- Scoperte: nuove serie iniziate, film ed episodi finiti, e le serie a cui è toccato esattamente un episodio.
- Un banner di una riga nella panoramica a dicembre e gennaio quando un riepilogo è pronto.

## [0.2.0] - 2026-09-19

Più di quanto Jellystat abbia mai registrato

### Added
- Una linea del tempo per ogni riproduzione: avvio, pausa, ripresa, salto, cambi di audio e sottotitoli, riproduzione diretta che diventa transcodifica, e arresto. In più i conteggi di pause e salti, il punto da cui è ripresa la riproduzione, e se lo spettatore era sulla rete locale.
- Scheda **Server**: versione di Jellyfin, aggiornamenti e riavvii in sospeso, uso del disco per libreria (Jellyfin 10.11+), plugin, risultati delle operazioni pianificate e ogni dispositivo registrato.
- Approfondimenti su **Riproduzione**: stream contemporanei nel tempo, quali client forzano le transcodifiche, fin dove arrivano le persone prima di fermarsi, riproduzioni locali contro remote e abitudini di visione.
- Di cosa è fatta la tua libreria: risoluzioni, codec, gamma dinamica, contenitori, titoli per decennio, aggiunte per mese, i titoli più grandi, e i titoli che nessuno ha mai guardato, verificati sia con la cronologia di FinStats sia con i segni di riprodotto di Jellyfin.
- Panoramica: picco di stream contemporanei, dati trasmessi stimati, quota di riproduzioni remote, generi per tempo di visione e accessi non riusciti.
- Pagine degli elementi: link IMDb e TMDB, studi, profondità di bit e frame rate, e chi ha il titolo segnato come riprodotto in Jellyfin.
- Pagine degli utenti: generi, e i film, gli episodi e i preferiti che Jellyfin ha registrato.

### Fixed
- L’etichetta del tempo su una scheda di riproduzione in corso non esce più dalla scheda.
- Le etichette lunghe nella pagina Riproduzione venivano tagliate troppo presto.
- Una serie che ha lasciato la libreria mostra ancora l’elenco delle stagioni e degli episodi.

## [0.1.0] - 2026-09-19

Prima versione

### Added
- Raccolta in tempo reale dalle sessioni di Jellyfin: utente, titolo, client, dispositivo, indirizzo IP, metodo di riproduzione, motivi della transcodifica e accelerazione hardware, dettagli di video, audio e sottotitoli, e tempo guardato contro tempo in pausa.
- Accesso con il tuo account Jellyfin. Una configurazione in due passaggi crea la chiave API di FinStats, e le password non vengono mai salvate. Gli utenti non amministratori possono, facoltativamente, accedere per vedere solo le proprie statistiche.
- Le pagine Panoramica, Attività, Utenti, Librerie, Riproduzione e Registro del server, una palette dei comandi (Ctrl/⌘ K) e una barra di stato in tempo reale.
- Importazione da Jellystat: i backup grandi vengono trasmessi su disco e importati in pochi secondi, e importare due volte lo stesso file è sicuro.
- Un unico piccolo binario con un’interfaccia web integrata e un solo file SQLite, più un’immagine Docker.
