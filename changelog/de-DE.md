# Versionshinweise

Alles, was sich in FinStats geändert hat, das Neueste zuerst. Diese Datei wird in das Programm
einkompiliert und ist das, was der Tab **Versionshinweise** in der App zeigt.

Format: `## [version] - date`, oder `## [version] - first day to release day` für eine Version, die über
mehrere Tage entstand, dann `### Added`, `### Changed`, `### Security`, `### Performance`, `### Stability`,
`### Fixed` oder `### Removed` (Neu, Geändert, Sicherheit, Leistung, Stabilität, Behoben, Entfernt) mit einem
Punkt pro Änderung. Sicherheit ist etwas, das jemand hätte missbrauchen können, Leistung etwas, das schneller
oder sparsamer wurde, Stabilität etwas, das nicht mehr abstürzen, anhalten, Lecks haben oder Daten verlieren
kann, Behoben etwas, das eine falsche Antwort gab. Eine `x.y.0`-Version trägt unter ihrer Überschrift eine
kurze Titelzeile: Diesen Titel nutzt die App als Schlagzeile der ganzen Reihe.

## [2.3.1] - 2026-10-26 to 2026-10-28

### Security
- **Die Einrichtung fragt nach einem Einmalcode**: Solange FinStats nicht eingerichtet ist, schreibt es beim Start einen Code in sein Protokoll (`docker logs finstats`), und der Assistent fragt danach, bevor er eine Adresse testet oder etwas einrichtet. Sonst konnte, wer den Port zuerst erreichte, FinStats auf ein eigenes Jellyfin richten und dessen Administrator werden oder mit dem Test das Netzwerk auskundschaften.
- **Der Zugriff auf Mediatheken folgt Jellyfin**: Jede Person sieht nur die Mediatheken, die Jellyfin ihr öffnet, und darin nur die Titel, die Jellyfin ihr unter ihrer Altersfreigabe, ihren Tags und ihren Regeln für Titel ohne Freigabe auflistet, in Mediatheksseiten, Suche, Besetzung, Bildern, Kürzlich hinzugefügt, der Merkliste, Demnächst, dem Kalender-Feed und Benachrichtigungen über neue Titel. Jede angemeldete Person konnte jede Mediathek durchsuchen.
- Ein Link konnte jemanden direkt nach der Anmeldung von der Begrüßung auf eine andere Website schicken. Jetzt werden nur noch die eigenen Seiten von FinStats aufgerufen.
- Das Serverprotokoll und die fehlgeschlagenen Anmeldungen in den Einblicken gaben jedem mit *Den Server sehen* die Anmeldeadressen und die Aktivität aller. Zeilen anderer erfordern *Aktivität aller sehen* und eine Adresse *Netzwerkdetails sehen*, wie überall sonst.
- Ein persönliches Benachrichtigungsziel konnte das lokale Netzwerk über einen Namen erreichen, der beim zweiten Nachschlagen anders antwortete. Verbunden wird jetzt mit der geprüften Adresse, weitere Bereiche, die nie ins Internet führen, gelten als lokal, die eigene öffentliche Adresse des Haushalts wird ebenfalls abgelehnt, und ein Name, der sich nicht nachschlagen lässt, wird mit denselben Worten abgelehnt wie ein lokaler.
- Hinter einem Proxy, der eine eigene `X-Forwarded-For`-Zeile hinzufügt, oder aus einem IPv6-Netzwerk konnte jeder Anmeldeversuch von einer neuen Adresse kommen, und das Limit griff nie. Ein IPv6-/48 bekommt jetzt 50 Versuche in fünf Minuten, ein volles Limit lehnt eine neue Adresse ab, statt eine alte zu vergessen, der ganze Server stoppt bei 60 Fehlversuchen pro Minute, und eine fehlgeschlagene Anmeldung wird einmal pro Adresse und Zeitfenster protokolliert.
- `FINSTATS_TRUST_PROXY` nennt die Proxys (Adressen oder Bereiche), deren `X-Forwarded-For`, `X-Forwarded-Host` und `X-Forwarded-Proto` geglaubt werden, und `1` oder `true` glaubt jetzt nur einer Gegenstelle auf Loopback oder in einem privaten Netzwerk. Eingeschaltet konnte jede Gegenstelle ihre eigene Adresse wählen; ausgeschaltet hinter einem Proxy sperrten zehn falsche Rateversuche alle aus.
- Sobald die Benutzer gelesen sind, wird eine Anmeldung mit einem Namen, der unbekannt, deaktiviert, entfernt oder nicht für FinStats zugelassen ist, ohne Rückfrage bei Jellyfin als falsches Passwort beantwortet, und nicht früher, als Jellyfin geantwortet hätte. Solche Namen konnten Jellyfin-Konten sperren, Jellyfins Regel für den Fernzugriff umgehen oder sich hinter der Adresse von FinStats verstecken.
- Wen Jellyfin auf das Heimnetz beschränkt (*Fernverbindungen erlauben* aus), meldet sich bei FinStats nur von dort an, da Jellyfin den Schalter anhand der eigenen Adresse von FinStats beurteilte, und ein Lesen der Benutzer, das nicht sagt, ob jemand deaktiviert ist, lässt diese Person draußen.
- Ein Sitzungscookie wird als Secure markiert, sobald die Anmeldeseite über https läuft, auch hinter einem Proxy, der das nicht mitteilt.
- Eine erfolgreiche Anmeldung vergibt nicht mehr die Rateversuche davor. Unter https ist das Sitzungscookie ein `__Host-`-Cookie, das eine Website unter einem benachbarten Namen nicht ersetzen kann und das unter keinem anderen Namen angenommen wird, und eine Abmeldung beendet jede Sitzung, die der Browser mitgeschickt hat.
- Nur ein Kalenderschlüssel öffnet den Kalender-Feed über seine Adresse; jeder andere Schlüssel gehört in den Header.
- Ein API-Schlüssel kann keine Berechtigungen mehr vergeben, nicht mehr ändern, wer hineinkommt oder was öffentlich ist, keine Sicherung wiederherstellen und keinen Dienst und kein Benachrichtigungsziel hinzufügen, ändern oder testen: Jedes davon überdauert den Schlüssel und braucht daher einen angemeldeten Browser.
- Jede Antwort trägt `Cross-Origin-Opener-Policy: same-origin` und eine `Permissions-Policy`, API-Antworten tragen `Cross-Origin-Resource-Policy: same-origin` und sind `no-store`, sofern sie nichts anderes sagen, und die CSP erlaubt keine `blob:`-Bilder mehr. Eine Seite, die FinStats geöffnet hatte, konnte ihren Tab später in eine eigene Anmeldeseite verwandeln.
- Wo FinStats das Schema kennt, über das es erreicht wird (das `X-Forwarded-Proto` eines vertrauenswürdigen Proxys oder die öffentliche Adresse), wird ein Schreibvorgang von einem anderen Schema als websiteübergreifend abgelehnt.
- `/auth/me`, die Seite eines Titels und Mediathek-Zustand verlinken auf Jellyfin unter seiner öffentlichen Adresse; die Adresse, unter der FinStats selbst Jellyfin liest, geht nur an Administratoren.
- Ein Bild wird nur für etwas aus Jellyfin geholt, das FinStats kennt, auch die Poster und Karten eines veröffentlichten Profils, sodass sich Bilder anderer Mediatheken nicht über ihre ID anfordern lassen.
- Die Schlüssel eines privaten Trackers werden jetzt auch aus den Fehlern der Download-Warteschlange entfernt, nach ihrer Form wie nach ihrem Namen, und wer nur *Downloads sehen* hat, erfährt, welcher Dienst fehlschlug und wie, nie aber seine interne Adresse oder wohin er umleitet.
- Die Serverseite nennt den letzten Benutzer eines Geräts nur mit *Aktivität aller sehen*, und Nicht verknüpfte Medien erfordert das zusätzlich zu *FinStats verwalten*. Ohne diese Berechtigung zählt der Bereich Aufgaben nur die eigenen Wiedergaben und Streams, eine entfernte Mediathek wird nach den eigenen Wiedergaben aufgeführt, und ein wartendes Tautulli-Steckbrett lässt sich nicht abbrechen.
- Die Einstellungen zeigen den Dienst für die öffentliche Adresse nur mit seinem Host und die Adresse von Jellyfin und den Datenordner nur Administratoren, und ein Titel, den die Mediathek verloren hat, wird nur aus Wiedergaben beschrieben, die der Lesende sehen darf.
- Eine Tautulli-Sicherung, deren Tabellen keine einfachen Tabellen waren, konnte jeden anderen Schreibvorgang warten lassen, bis FinStats neu gestartet wurde. Eine solche Datei wird abgelehnt, und SQLite liest die Sicherung defensiv.
- Ein Upload über 4 GB wird abgelehnt, und eine gezippte Tautulli-Sicherung wird nicht weiter entpackt als bis zur angegebenen Größe; beides konnte die Festplatte füllen, auf der die Datenbank von FinStats liegt.
- Eine Antwort von Jellyfin, Sonarr, Radarr, Seerr, der Abfrage der öffentlichen Adresse oder dem GeoIP-Download wird nur bis zur zulässigen Höchstmenge gelesen, gezählt nach dem Entpacken.
- Jeder Jellyfin-Benutzer konnte FinStats dazu bringen, für niemanden mehr Wiedergaben aufzuzeichnen, indem er Tausende kurzer Wiedergaben eines Titels meldete. Herauszufinden, wer zusammen geschaut hat, braucht jetzt einen Durchgang und hält dabei keine Sperre.
- Direkt aufeinanderfolgende Wiedergaben oder ein Titel, der reihum von Gerät zu Gerät abgespielt wird, fassen keine Fremden mehr zu einer Gruppe zusammen, und wer nur sich selbst sehen darf, erfährt nur von den Personen, neben deren Wiedergabe die eigene lief, und für die Zeit, in der sie es tat.
- Ein Link konnte jemanden nach der Anmeldung über einen Pfad wie `/.//` immer noch von der Begrüßung auf eine andere Website schicken.
- Wer FinStats verwaltet, kann nicht mehr ändern, wie viele Sicherungen behalten oder wie oft sie geschrieben werden, das Zusammenführungsfenster (jetzt höchstens eine Stunde) oder wann Benutzer, das Aktivitätsprotokoll und Sicherungen gelesen werden, und die Sicherungen eines Verwalters verdrängen nie die eines Administrators; Heimadressen und die Reiseregeln erfordern dasselbe wie die Warnungen, und das Importieren des Verlaufs auch *Aktivität aller sehen*.
- Ein gespeicherter Schlüssel, ein Token oder ein Passwort geht nur an den Server, für den es gespeichert wurde, und ein Benachrichtigungsziel wird nur angemeldet angelegt oder geändert, nie mit einem API-Schlüssel. Ein Administrator darf das persönliche Ziel eines anderen ausschalten oder entfernen, aber nicht umleiten, seine Zertifikatsprüfung nicht abschalten und sein ntfy-Topic oder seinen Pushover-Benutzerschlüssel nicht lesen.
- Discord- und Slack-Webhooks müssen `https` sein, da ihre Adresse ihr Token ist.
- Eine Flut fehlgeschlagener Anmeldungen kann keine echte mehr verbergen oder die Warnungen verschütten: Namen ohne Konto ergeben eine Meldung pro Viertelstunde, die niemanden nennt, Warnungen gehen zuerst hinaus, und Slack und Discord zeigen den Inhalt einer Nachricht als Text und pingen niemanden.
- Der Rang im Jahresrückblick, Kapazität, eine Benachrichtigung und ein veröffentlichtes Profil verraten nicht mehr als die Seiten: kein Rang und kein Anteil an der Zeit aller für jemanden, der nur sich selbst sehen darf, Spitzenzeiten ohne *Aktivität aller sehen* nur als Tage mit drei oder mehr Streams, kein fehlgeschlagener Job und keine Verbindung für jemanden, der FinStats nicht verwalten darf, und ein öffentliches Profil ändert sich einmal am Tag.
- Eine Tautulli-Sicherung mit einer generierten Spalte wird abgelehnt, und ein Bild für eine importierte Wiedergabe wird erst geholt, wenn die Mediathek ihren Titel hat.
- Eine Wiederherstellung öffnet FinStats nie weiter, als es ist: Einstellungen kommen nur auf Wunsch zurück, Anmeldung, Berechtigungen und öffentliche Profile werden zu dem, was sowohl diese Installation als auch die Sicherung erlauben, und die öffentlichen Adressen bleiben die dieser Installation. `finstats restore <file>` bringt wie die Seite nur den Verlauf zurück, mit `--with-settings` auch die Einstellungen.
- Eine Wiederherstellung nur des Verlaufs bringt keine Heimadressen, stummgeschalteten Warnungen, Audit-Einträge, die als eigene dieser Installation durchgehen, oder zugeordnete Titel, die der Mediathek fehlen, mehr zurück, und keine Wiederherstellung bringt Wiedergaben, Protokollzeilen oder Geräte zurück, die vor 1990 oder nach morgen datiert sind. Eine wiederhergestellte Wiedergabe gilt als importiert, egal welche Quelle sie angibt, sodass ein Neustart sie nie fortsetzt.
- Der Datenordner gehört allein dem eigenen Benutzer von FinStats (0700, seine Dateien 0600, `umask 077` im Image). FinStats weigert sich, als root über einem Datenordner zu laufen, der einem anderen Benutzer gehört, und nennt das zu verwendende `docker exec -u`, und der Entrypoint ändert den Eigentümer nur des Ordners und der eigenen Einträge von FinStats.
- Der Name eines Ziels darf keine Steuerzeichen enthalten, und der Name eines Geräts, einer Person oder eines Titels kann keine Zeile im Protokoll mehr fälschen.
- Der Routenparameter eines Links kann keinen anderen API-Pfad als den der eigenen Seite mehr erreichen, und der Elementbaukasten von FinUI setzt eine Adresse nur, wenn sie http(s) oder relativ ist, nie `srcdoc`.
- Suche, veröffentlichte Profile und von Hand gestartete Arbeit können FinStats nicht mehr dauerhaft beschäftigen: Eine Suche endet bei 200 Zeichen, ein Profil wird pro Sprache vorgehalten, Karten warten in einer kurzen Warteschlange, und ein Job, die Adressabfrage, der Download der Standortdatenbank und eine Testnachricht starten nur in Abständen, wie auch immer sie angefordert werden, Testnachrichten pro Absender gezählt. Das Abrufen der Standortdatenbank erfordert außerdem dasselbe wie das Lesen der Sicherheitsseite.
- Der Inhalt einer Anfrage muss nach den ersten 15 Sekunden weiter eintreffen, eine Antwort, die niemand liest, wird nach 30 Sekunden aufgegeben, und eine Adresse hält höchstens 64 Verbindungen, ein vertrauenswürdiger Proxy ausgenommen. Einige hundert Anfragen, die ihren Inhalt nie schickten, konnten FinStats davon abhalten, irgendwem zu antworten.
- Ein Mailserver wird nicht weiter gelesen, als eine Antwort sein kann (1 MB für einen ganzen Brief), und eine Testnachricht endet nach so langer Zeit, wie eine Nachricht in der Warteschlange dauern darf. Ein Server, der nie eine Zeile beendete, konnte FinStats immer mehr Speicher halten lassen, bis es gestoppt wurde.
- Ein Lesen der Bibliothek wird an den Titeln gemessen, die Lesevorgänge gespeichert haben, nicht an denen eines Imports, und ein Jellystat-Titel in einer bereits gelesenen Bibliothek kommt als entfernt herein. Ein Import erfundener Titel konnte jedes Lesen der Bibliothek FinStats anhalten lassen, bei jedem Start.
- Die serverweite Grenze von 60 fehlgeschlagenen Anmeldungen pro Minute zählt nur Versuche, die an Jellyfin gehen, auch solche, die noch auf die Antwort warten. Namen, die FinStats selbst abwies, ließen jeden mit wenigen Adressen alle Anmeldungen verhindern, und viele gleichzeitig gesendete Versuche erreichten alle Jellyfin.
- Wen Jellyfin auf das Heimnetz beschränkt, der bleibt bei jeder Anfrage darauf beschränkt, Sitzung und Schlüssel eingeschlossen, nicht nur bei der Anmeldung; zu Hause ist ein privates Netz oder die nachgeschlagene öffentliche Adresse des Haushalts, nie eine als Zuhause eingetragene Adresse, und hinter einem Proxy, den `FINSTATS_TRUST_PROXY` nicht nennt, bleibt die Person draußen. Ein Cookie oder Schlüssel, aus dem Haus mitgenommen, öffnete alles von überall, und ein Verwalter konnte das Haus für eine Adresse seiner Wahl öffnen.
- Ein API-Schlüssel kann nicht mehr abschalten, was seinen Missbrauch bemerken würde, oder die Sicherungen, die rückgängig machen würden, was er tat: wie viele Sicherungen behalten werden und wie oft, das Zusammenführungsfenster, Heimadressen und Reiseregeln, die Zeitpläne für Benutzer, Aktivitätsprotokoll und Sicherungen, das Stummschalten oder Erledigen aller Warnungen, eine Sicherung in den Papierkorb oder das Entfernen eines Benachrichtigungsziels. Jedes davon überdauerte den Schlüssel, wenn er widerrufen wurde.
- Die Poster von Sonarr und Radarr und die nächste Folge auf der Merkliste folgen ebenfalls dem Bibliothekszugriff: Eine Serie, die Jellyfin vor jemandem verbirgt, hat für diese Person weder Poster noch Datum. Das Durchzählen der IDs der Dienste verriet, welche verborgenen Serien kamen, und eine Liste nannte die nächste Folge einer verborgenen Serie.
- Das Kapitel Gemeinsam im Jahresrückblick nennt nur die Personen, deren Wiedergabe neben deiner lief, wie die Seite Gemeinsam. Eine Gruppe, in die man über jemand anderen kam, nannte Leute, neben denen man nie geschaut hatte.
- Ein Import aus Jellystat oder Streamystats liest die ganze Datei, bevor er die Schreibsperre der Datenbank nimmt, und schreibt dann nur die Zeilen, die er behält. Unter der Sperre gelesen, ließ eine Datei aus wenigen echten Zeilen und Gigabytes von irgendetwas anderem jedes andere Schreiben warten, Wiedergaben und Anmeldungen eingeschlossen.

### Performance
- Ein veröffentlichtes Profil wird höchstens einmal pro Minute berechnet, nicht erneut für jedes Bild und jede Karte darauf.
- Der Verlauf einer Person wird immer nur einmal zur gleichen Zeit nach Sicherheitswarnungen durchsucht, egal wie viele Wiedergaben beginnen.
- An- oder Abmelden, das Widerrufen eines Schlüssels oder das Ändern der Berechtigungen einer Person lässt nur die Live-Verbindungen erneut nachsehen, die es betrifft.
- Das Tautulli-Steckbrett liest den Verlauf einmal, nicht einmal pro Plex-Benutzer.
- Ein Bild, das im Cache fehlt, wird mit einer Anfrage geholt, während die anderen warten, und eines, das die Quelle nicht liefern konnte, wird 30 Sekunden lang nicht erneut angefragt.
- Die Dateien der Weboberfläche werden einmal mit gzip komprimiert und aufbewahrt, und Schriften werden nicht erneut komprimiert.

### Stability
- Eine Sicherung im älteren Format von Jellystat wird zeilenweise statt am Stück gelesen, und eine Zeile, die länger ist als in jeder Sicherung, wird abgelehnt.
- Zwei gleichzeitige Tautulli-Uploads schreiben nicht mehr in dieselbe Datei.
- Eine Seitenzahl weit hinter dem Ende der Titel einer Mediathek oder des Mediathek-Zustands ergibt eine leere Seite, keinen Fehler.
- Ein Import oder eine Wiederherstellung, deren Upload abbrach, lehnt nicht mehr jeden weiteren bis zu einem Neustart ab, und weder er noch ein Import, der vor seinem Beginn aufgegeben wurde, hinterlässt eine Datei.
- Eine Zeile einer Sicherung wird höchstens bis 8 MB gelesen, und es kommt jeweils nur eine Tautulli-Sicherung an.
- Eine Wiedergabe behält höchstens 2.000 Ereignisse, und ein Import überspringt eine Wiedergabe, die nicht zwischen 1990 und morgen beginnt und endet.
- Eine einzige stark negative Episodennummer lässt nicht mehr jedes Lesen der Mediatheken hängen: Mediathek-Zustand (auch seine Zusammenfassung), Kapazität, das Neuverknüpfen und das Zuordnen addieren und vergleichen Zahlen ohne Überlauf, und ein Release-Build prüft auf Überlauf, sodass alles Übersehene eine Aufgabe stoppt, statt in einer Schleife zu laufen.
- Eine Staffel-, Episoden- oder Endnummer wird nur behalten, wo Jellyfin sie halten könnte (0 bis 2.147.483.647), aus Jellyfin, Jellystat und Tautulli gleichermaßen.
- Eine Tautulli-Sicherung mit sich wiederholenden IDs wird abgelehnt, da ein paar MB Zeilen unter einer ID dem Prozess den Speicher ausgehen ließen, und eine Sicherung wird unveränderlich geöffnet und hinterlässt kein `-wal` oder `-shm`.
- Eine Verbindung muss ihre Anfrage innerhalb von 15 Sekunden senden, und höchstens 512 werden gleichzeitig bedient: Ein paar hundert halboffene Sockets nahmen FinStats vom Netz. Ein Frame auf dem Live-Kanal ist auf 4 KB begrenzt.
- Ein paar Kilobyte von Sonarr, Radarr oder Seerr können nicht mehr zu Gigabyte an Speicher werden: Eine Liste behält nicht mehr, als FinStats davon liest.
- Ein Benachrichtigungsziel, das nie antwortet, hält nur sich selbst auf: An Ziele wird nebeneinander gesendet, und eines, bei dem es immer wieder zur Zeitüberschreitung kommt, wird zehn Minuten in Ruhe gelassen.
- Die Restzeit und Gesamtgeschwindigkeit einer Warteschlange, die Zahlen einer wiederhergestellten Wiedergabe und der Tag, an dem eine Wiedergabe den Papierkorb verlässt, können nicht mehr überlaufen.
- Eine Wiederherstellung liest die ganze Datei, bevor sie die Schreibsperre nimmt, lehnt eine ab, die sich auf weit mehr als ihre Größe entpackt, und schreibt in Stapeln, sodass der Collector währenddessen weiter aufzeichnet.
- Sicherungen werden in UTC benannt und nie über eine bestehende Datei geschrieben, und nur die eigenen Kopien von FinStats vor einem Update werden aufgeräumt.
- Eine Zeile im Serverprotokoll, die in der Zukunft datiert ist, hält das Lesen des Aktivitätsprotokolls nicht mehr auf: Es wird ab seiner neuesten Zeile gelesen, die nicht in der Zukunft liegt.
- Gleichzeitig eintreffende Anfragen an einen veröffentlichten Link berechnen seine Antwort einmal, und seine Karten werden auf einer eigenen Spur gezeichnet, sodass jemand mit dem Link die Rückblick-Karten des Besitzers nicht mehr mit 503 antworten lassen kann.

### Fixed
- Fehlgeschlagene Anmeldungen bei einem Konto ergeben eine Warnung pro Zeitfenster, die bei zwanzig zu einem Alarm wird, und ein Name zählt nur gegen ein Konto, wenn er vollständig eingegeben wird ("x alice" zählt nicht mehr gegen alice).
- Discord- und Slack-Nachrichten werden zuerst maskiert und danach gekürzt: Ein Gerätename voller Markdown ließ Discord jede Warnung ablehnen, die ihn nannte.
- Eine öffentliche IPv6-Adresse wird für ein persönliches Ziel akzeptiert.
- Hinter einem Proxy, der selbst anmeldet (nginx `auth_basic`), wird das Cookie von FinStats nicht mehr abgelehnt.
- Ein Anzeigename mit `{{url}}` wird in der Vorschau einer veröffentlichten Seite so gezeigt, wie er geschrieben ist.
- Ein Rückblick für das Jahr 9999 zeigt das Standardjahr statt eines Fehlers.
- Eine Live-Seite, deren Berechtigung entzogen und wieder erteilt wurde, während sie offen war, bleibt wieder aktuell, ohne neu zu laden.

### Removed
- **Synnmørsk** ist keine Sprache von FinStats mehr: Ein Browser, der es gewählt hatte, liest Nynorsk, die Schriftsprache, aus der es entstand.

## [2.3.0] - 2026-10-07 to 2026-10-26

Kapazität, ein Live-Kanal, der einfache Modus und Sprachen

### Added
- **Sprachen**: FinStats spricht 23 Sprachen. Britisches und amerikanisches Englisch sowie Norwegisch (Bokmål) und Norwegisch (Nynorsk) werden vom Maintainer geschrieben und geprüft; Dänisch, Deutsch, Spanisch, Finnisch, Französisch, Hindi, Isländisch, Italienisch, Japanisch, Koreanisch, Niederländisch, Polnisch, Portugiesisch (Brasilien), Russisch, Schwedisch, Türkisch, Ukrainisch und Chinesisch (vereinfacht und traditionell) sind Übersetzungen, die hier niemand geprüft hat, und sagen auf Englisch, was ihnen noch fehlt. Jede Seite beider Layouts, das öffentliche Profil, die Versionshinweise und alles, was der Server in Worten antwortet (eine Fehlermeldung, die Karten des Rückblicks, was eine Jellyfin-Aufgabe tut), erscheinen in der Sprache, die unter Einstellungen → Darstellung → Sprache gewählt ist, pro Browser; solange keine gewählt ist, entscheidet die Sprache des Browsers. Zahlen, Datumsangaben, Einheiten und Aufzählungen werden so geschrieben, wie diese Sprache sie schreibt. Ein Skript kann mit `Accept-Language` eine bestimmte anfordern.
- **Einfacher Modus**: ein zweites Layout für alle, die von Plex kommen, aufgebaut wie ein klassischer Statistikserver für Plex. Eine Leiste oben statt des Menüs und nur die Statistikseiten: Startseite, Mediatheken, Benutzer, Verlauf, Diagramme und Einstellungen, dazu ein Zahnrad für dein Profil, die Einstellungen, die Versionshinweise, das Design und das Abmelden. Wird pro Browser unter Einstellungen → Darstellung → Layout gewählt. Er lädt nur seine eigene Hälfte von FinStats und startet deshalb schneller; Rückblick, Gemeinsam, Pipeline, Sicherheit und die Serverseiten bleiben im FinStats-Layout. Er trägt die eigenen Farben von FinStats, hell und dunkel, und nichts darin bewegt sich.
- **Die Startseite im einfachen Modus**: was gerade läuft, eine Karte pro Stream (wie er abgespielt wird, von zu Hause oder unterwegs, wann er endet), die Wiedergabestatistik als Ranglisten-Karten nach Anzahl der Wiedergaben oder Dauer über die letzten N Tage, was jede Mediathek enthält und was zuletzt hinzukam.
- **Die Diagramme im einfachen Modus**: Wiedergaben nach Medienart (Tag für Tag, nach Wochentag und Stunde, die zehn meistgenutzten Apps und Personen), nach Art der Wiedergabe (Tag für Tag, die meisten gleichzeitigen Streams jedes Tages, die Auflösung der Datei und die gesendete, Apps und Personen) und Monat für Monat, nach Anzahl der Wiedergaben oder Dauer, für alle oder für die gewählten Personen.
- **Mediatheken, Benutzer und Titel im einfachen Modus**: Tabellen aller Mediatheken und Personen mit dem, was zuletzt abgespielt wurde, worauf und von wo (die Adresse nur mit *Netzwerkdetails sehen*), und eine Seite für jede Mediathek, jede Person und jeden Titel mit ihren Zahlen und ihrem eigenen Verlauf.
- Für Skripte: `/api/stats/split` (Wiedergaben nach App, Person, Auflösung, Tag oder Monat, aufgeteilt nach Medienart oder Art der Wiedergabe); Toplisten nach Anzahl verschiedener Zuschauer (`sort=users`); Aktivität eingegrenzt auf geschaute Wiedergaben (`watched=1`); Auswertungen nur für die angefragten Teile (`parts=`); die Spitzenlast aufgeteilt in Transkodierung, direktes Streaming und direkte Wiedergabe, auch pro Tag; was jede Mediathek enthält und zuletzt abspielte; kürzlich Hinzugefügtes nach Art (`kind=`); die Heatmap nach Medienart; die letzte Wiedergabe einer Person; ob ein Stream von zu Hause läuft.
- **Ein Live-Kanal**: Die Übersicht, die Downloads in der Pipeline und die Statusleiste halten sich über eine WebSocket-Verbindung pro Tab (`/api/live`) aktuell, sodass sich das, was läuft und was heruntergeladen wird, auf dem Bildschirm ändert, sobald es sich ändert, und nicht erst beim nächsten Nachsehen. Hinter einem Reverse-Proxy, der WebSocket-Upgrades nicht weiterreicht, oder bei einer Verbindung, die wortlos abgebrochen ist, fragen die Seiten wie bisher alle paar Sekunden nach. `/api/now-playing`, `/api/downloads` und `/api/summary` bleiben, wie sie sind, für Skripte.
- **Server → Kapazität**: Kommt dieser Server zurecht, gemessen an dem, was FinStats gesehen hat? Die geschäftigsten Momente insgesamt und in den letzten 90 Tagen: gleichzeitige Streams und wie viele davon ihr Bild mit Hardware transkodiert bekamen, ohne Hardware oder ohne Angabe (ein aus Tautulli importierter Verlauf stammt von Plex und bleibt außen vor). Jeder Datenträger, auf dem die Mediatheken liegen, und wie schnell das, was Jellyfin in sechs Monaten hinzugefügt hat, ihn füllt (ein erst kürzlich befüllter Datenträger wird ab einem Monat nach seiner ersten Datei gemessen, denn sein erster Monat ist das Befüllen selbst) und, wo Jellyfin 10.11 oder neuer den freien Platz meldet, wie lange dieser bei diesem Tempo reicht. FinStats sagt nie, wie viele Streams der Server verkraften könnte.
- **.ics exportieren** unter Pipeline → Demnächst: die Liste so, wie du sie gefiltert hast, heruntergeladen als Kalenderdatei zum einmaligen Importieren. Sie aktualisiert sich nicht selbst; ein Kalenderschlüssel aus Einstellungen → API-Schlüssel liefert weiterhin einen Kalender, der das tut.
- **Einen Blick wert**: Kapazität weist darauf hin, wenn jede Transkodierung, über die Jellyfin in 90 Tagen berichtet hat, ohne Hardwarebeschleunigung lief, obwohl häufig transkodiert wird, wenn der Mediathek-Scan von Jellyfin ein Viertel der Zeit oder mehr läuft und wenn ein Datenträger für weniger als sechs Monate Platz hat. Jeder Hinweis kommt mit seinen Zahlen und lässt sich beiseitelegen wie die Befunde im Mediathek-Zustand.
- **Woraus deine Mediathek besteht** lässt sich auf eine oder mehrere Mediatheken eingrenzen, unter Mediatheken und auf der Seite jeder Mediathek (`library_id=a,b`).
- **Datums- und Zeitformate** unter Einstellungen → Darstellung: ein Datum als 31.01.2026, 31/01/2026, 31-01-2026, 01/31/2026, 2026-01-31, 2026.01.31 oder 2026/01/31 und eine Uhrzeit mit 24 oder 12 Stunden, mit oder ohne führende Null, oder jeweils so, wie die Sprache sie schreibt. Gespeichert in diesem Browser.
- **Eine Begrüßung**: Wenn sich jemand zum ersten Mal anmeldet, in welchem Browser auch immer, und am Ende der Einrichtung, fragt FinStats nach Sprache, Datums- und Zeitformat und Layout, mit einem Bild von jedem. Überspringen behält britisches Englisch, 31/01/2026, 13:05 und FinStats.

### Changed
- Das Logo innerhalb einer Seite ist das bloße Zeichen neben dem Namen, gesetzt in Inter, egal welche Schrift eine Voreinstellung wählt; wo es allein steht (das Favicon), behält es seine abgerundete Kachel.
- Eine in Software transkodierte Wiedergabe wird als solche erfasst, wo Jellyfin das angibt; vorher sah sie genauso aus wie eine von einem Server, der es nicht angibt. Die Seiten nennen beide weiterhin Software.
- Ein Update führt jetzt neben seinen Migrationen einmalig und älteste Version zuerst aus, was eine Version an einer Datenbank erledigen muss, die ein älteres FinStats geschrieben hat. Für 2.3.0 heißt das: Die Serverdetails werden beim ersten Start gelesen, sodass Kapazität sofort da ist.

### Performance
- **Eine offene Seite fragt nichts mehr ab**: Die Übersicht stellte 30 Anfragen pro Minute, um aktuell zu bleiben, die Downloads in der Pipeline 18; mit dem Live-Kanal stellen beide keine. Egal wie viele Tabs offen sind, FinStats fragt Jellyfin, Sonarr und Radarr nicht mehr als vorher, und bei einem verborgenen Tab wird die Download-Warteschlange weiterhin seltener gelesen.
- Die Seite einer Person liest nur die Wiedergaben dieser Person: Summen und letzte Wiedergabe wurden über alle Wiedergaben des Servers berechnet und erst dann auf sie eingegrenzt, etwa vier Sekunden bei einer Million Wiedergaben.
- Die Seite einer Mediathek liest nur deren Titel und Wiedergaben, wo sie vorher alle Mediatheken zählte und eine davon behielt: ein Drittel der Zeit bei einer Million Wiedergaben.
- Ging die Uhr von Jellyfin der von FinStats voraus, las *Nach dem Mediathek-Scan von Jellyfin* jede Minute die ganze Mediathek neu, bis der Unterschied verstrichen war; jetzt folgt auf jeden Scan genau ein Lesen.
- Eine GeoIP-Datei, die sich nicht öffnen ließ, wurde jede Viertelstunde erneut kopiert, hundert Megabyte und mehr, und jede Adresse neu verortet; jetzt wird sie erst wieder versucht, wenn sie sich ändert.
- Der Mediathek-Zustand öffnet sofort, nachdem der Zeiger auf seinem Link geruht hat: Was vorab geholt wurde, lag dort, wo die Seite nie nachsah.

### Stability
- Ein Update, das nach einer bereits gelaufenen Migration scheiterte und von Docker sofort neu gestartet wurde, legte bei jedem Start eine neue Kopie der halb aktualisierten Datenbank an, und drei Starts räumten die einzige Kopie weg, die die vorherige Version öffnen konnte. Die Kopie von vor dem Update bleibt jetzt erhalten.
- Ein Titel, der in Jellyfin gelöscht wurde, während die Mediathek gelesen wurde (eine von Radarr ersetzte Datei), konnte dazu führen, dass das Lesen einen anderen, noch vorhandenen Titel übersah und ihn als entfernt markierte; seine Wiedergaben konnten dann auf eine andere Kopie davon wandern. Was ein Lesevorgang nicht gesehen hat, wird jetzt zuerst per ID nachgefragt.
- Das Wiederherstellen einer Sicherung von vor dem Umbenennen oder dem Upgrade einer Datei stellte jede seitdem auf die neue Datei verschobene Wiedergabe ein zweites Mal her, dauerhaft verdoppelt.
- Eine Anfrage in der Antwort von Seerr, die FinStats nicht lesen konnte (ein Feld, das in einem Seerr-Update umbenannt wurde), wurde beim täglichen vollständigen Lesen als entfernt markiert.
- Eine Aufgabe, die auf einen Fehler stieß, blieb für immer auf „läuft“, und kein Auslöser startete sie erneut, bis FinStats neu gestartet wurde; jetzt endet sie als fehlgeschlagen.
- Ein beschädigtes Tautulli-ZIP brach das Hochladen ab, ohne die Datei zu entfernen, die die Tokens aller Plex-Benutzer enthält; sie blieb bis zur Bereinigung nach sechs Stunden liegen.
- Ein Import oder eine Wiederherstellung, die abgebrochen wurde, während ihre Datei ankam, ließ diese Datei für immer im Datenordner zurück, samt Adressen; sie wird jetzt beim nächsten Start entfernt.
- Ein Bild konnte halb geschrieben ausgeliefert werden, während es zwischengespeichert wurde, und der Browser behielt es dann so.
- Widerrufene API-Schlüssel blieben für immer gespeichert, egal wie viele erstellt und widerrufen wurden.
- Ein Lesen der Gesehen- und Favoritenmarken einer Person, das leer oder unlesbar zurückkam, löschte sie alle bis zum nächsten Lesen; es wird nun abgewiesen, wie ein leeres Lesen einer Mediathek.
- Eine Folge mit einer Endnummer, die keine Folge haben kann (ein dort eingetragenes Datum), konnte den Mediathek-Zustand nach einem Lesen der Mediathek ohne Ende beschäftigen.

### Fixed
- Ein Versionshinweis mit Code in Fettschrift (die von 2.0.2 und 2.0.3) zeigte seine Backticks, statt den Code darzustellen.
- Ein Diagramm mit Anzahlen konnte seine Achse mit 0, 3, 5, 8, 10 oder sogar 0, 0, 1, 1, 1 beschriften: Schritte von 2,5, gerundet geschrieben. Anzahlen steigen jetzt in ganzen Schritten.
- Ein Kalenderschlüssel, den jemand ohne das Recht zum Verwalten erstellte, kam mit einem Link auf die Adresse des Browsers statt auf die öffentliche Adresse von FinStats und funktionierte deshalb nur von innen; die Seite fragte außerdem Einstellungen ab, die ihr verweigert wurden.
- Die letzte Wiedergabe einer Person konnte ein paar Sekunden zu kurz sein, um anderswo als Wiedergabe zu zählen, und nannte dann einen Titel, den ihr Verlauf nicht zeigt.
- Eine fehlgeschlagene Wiederherstellung aus „Kürzlich gelöscht“ sagte nichts und ließ ihre Schaltfläche weiterdrehen; ebenso ein fehlgeschlagenes *Zurückholen* im Mediathek-Zustand und ein fehlgeschlagenes *Wieder öffnen* oder *Alle erledigen* unter Sicherheit.
- Der Kalender-Feed heißt in einer Kalender-App FinStats, nicht finstats.
- Die Download-Warteschlange verriet jedem, der sie sehen durfte, wer welchen Download angefragt hatte; die Anfrage einer anderen Person erfordert jetzt *Aktivität aller sehen*, wie in der Pipeline.
- Wurde jemandem *Benachrichtigungen erhalten* entzogen, gingen weiterhin Nachrichten an die Ziele dieser Person, und sie konnte sie nicht mehr abschalten.
- Eine Wiedergabe, die innerhalb des Zusammenführungsfensters pausiert und fortgesetzt wurde, wurde einmal als beendet gemeldet, ihre Fortsetzung und ihr echtes Ende nie.
- Die Wiedergaben eines Tracks konnten auf einen gleichnamigen Track eines anderen Künstlers („Intro“) wandern, wenn sein Album die Mediathek verließ.
- Ein Benachrichtigungsziel, das als IPv4-Adresse innerhalb von IPv6 geschrieben war (NAT64, 6to4), galt als öffentlich, obwohl es das lokale Netzwerk erreichte.
- Eine Sonarr- oder Radarr-Verbindung, die auf einen anderen Server umgestellt wurde, zeigte eine Woche lang weiter die Poster des alten Servers.
- Die Warnungen unter Sicherheit meldeten, nichts müsse angesehen werden, wenn zwei oder mehr Personen ausgewählt waren, und eine Seite, die durch das Erledigen ihrer letzten Warnung leer wurde, bot keinen Weg zurück.
- Jeder API-Schlüssel, der noch nicht abgelaufen war, behauptete, er sei „gerade eben“ abgelaufen.
- Ein Dialog blieb nach Zurück über der Seite stehen, und die Seite ließ sich nicht scrollen, bis er geschlossen wurde.
- Das Audit-Protokoll zeigte acht Arten von Einträgen (eine zurückgeholte Wiedergabe oder Sicherung, den geleerten Papierkorb, öffentliche Profile, eine geänderte Darstellung) nur mit ihren IDs und ohne etwas unter Details.
- Einen Titel über sein Kontextmenü auf die Merkliste zu setzen, scheiterte wortlos, wenn der Server ablehnte.
- *Gemeinsam geschaut* auf der Übersicht behandelte mehrere gleichzeitig ausgewählte Personen als eine.
- Der Rückblick schrieb Stunden und Tage unter zehn mit dem Dezimalkomma des Browsers neben seinen eigenen englischen Tausendertrennzeichen.
- Kapazität sagte, eine Festplatte werde "im Tempo der letzten 0 Monate" voll.
- Auf der Seite Sicherheit zählten die Adressen ohne Ort die Wiedergaben aller seit jeher, ganz gleich, auf welche Person und welchen Zeitraum die Seite eingeschränkt war.
- Wer *FinStats verwalten* durfte, aber nicht *Aktivität aller sehen*, konnte die Wiedergabe jedes anderen in den Papierkorb legen.
- Eine Karte aus dem Rückblick oder einem öffentlichen Profil konnte aus dem Cache des Browsers in der zuvor gewählten Sprache kommen.
- Die Downloads, die der Live-Kanal schickte, waren auf britischem Englisch, gleich in welcher Sprache die Seite war.
- Eine Anmeldung, die änderte, was jemand sehen darf, erreichte die anderen offenen Tabs der Person erst bei der nächsten Prüfung nach einer halben Minute.
- Ein Tab, der seine Live-Verbindung an einen neueren abgab, holte sich auf der nächsten Seite eine zurück, sodass die Tabs einer Person einander immer wieder schlossen.
- Tabellen sortierten Größen von 1.000 bis 1.023 Einheiten als Text in Sprachen, die Ziffern mit einem Leerzeichen gruppieren, und türkische Prozentangaben ebenso.
- Ein Titel mit einem Platzhalter im Namen, etwa "Big {device} Bunny", wurde von den Meldungen des Servers ausgefüllt.
- Tabellen sortierten Dauern und Anteile an einer Gesamtzahl (5 von 26) in jeder Sprache außer Englisch und den nordischen als Text.
- Ein Tab, der noch in einer Sprache offen war, hörte den Server in einer anderen, sobald ein anderer Tab sie gewählt hatte.
- Ein Wort neben einer Zahl stand auf Polnisch, Russisch und Ukrainisch in der falschen Form ("5 wieczory", "2 потоков").
- Kapazität zählte die erste Befüllung einer Mediathek als Wachstum und teilte, was jede Mediathek hinzugefügt hatte, durch sechs Monate, gleich über welchen Zeitraum gemessen; eine Datei, die den Platz einer anderen einnahm (ein Upgrade aus Radarr oder Sonarr oder eine Datei, der Jellyfin eine neue ID gab), zählte als ganz neu, obwohl nur der Unterschied neu war.
- Wer mit SyncPlay gemeinsam schaute, war keine Gruppe, wenn einer Minuten nach dem anderen dazukam; Wiedergaben, die gleichzeitig an derselben Stelle enden, gehören jetzt zusammen, und der Verlauf wird einmal neu gruppiert.
- Wer *Aktivität aller sehen* nicht hat, konnte an *Nie geschaut* einer Mediathek ablesen, dass jemand anderes einen Titel abgespielt hatte; jetzt zählt dort nur das eigene Schauen.

## [2.2.0] - 2026-10-01 to 2026-10-07

Mediathek-Zustand und FinUI

### Added
- **Ein eigenes Menü am Computer**: Einstellungen → Darstellung bietet acht an: die Seitenleiste wie bisher, eine nach dem Zweck der Seiten gruppierte, eine schmale Leiste aus Symbolen, die sich unter dem Zeiger zur Seitenleiste aufklappt, eine Seitenleiste, die mit der Suche beginnt, ein Dock, eine Befehlsleiste, Seiten, die du oben anheftest, und farbige Kacheln. Die Seitenleisten können rechts stehen. Das Dock gleitet weg, während du nach unten scrollst, und kommt zurück, wenn du nach oben scrollst oder auf den unteren Fensterrand zeigst; mit ihm wandert die Statusleiste nach oben.
- **Ein eigenes Menü am Handy**: eine Tableiste mit Mehr (der neue Standard), ein Blatt, das sich zur Hälfte öffnet, der ganze Bildschirm, die Seiten im Bogen um den Daumen gefächert oder eine Adressleiste, die zum Menü wächst. Jedes durchsucht seine Seiten und reicht alle anderen Wörter an die Suche weiter.
- Beide merkt sich der Browser, in dem sie gewählt wurden, sodass Handy und Computer jeweils ihr eigenes behalten; beide sind FinUI-Komponenten, `desktop-nav` und `mobile-nav`.
- **Ein Kontextmenü**: Klicke irgendwo in FinStats mit der rechten Maustaste auf einen Titel, eine Person oder eine Mediathek (ein Poster auf der Übersicht, eine Listenzeile, einen Namen in einer Tabelle, einen Film unter Demnächst, der noch nicht in der Mediathek ist), halte den Finger darauf oder drücke die Menütaste, und ein Menü wächst aus der gedrückten Stelle: öffnen, in einem neuen Tab öffnen, einen Film oder eine Serie auf deine Merkliste setzen, in Jellyfin oder bei TMDB und TVDB öffnen, den Link kopieren. Umschalt mit Rechtsklick öffnet weiterhin das Menü des Browsers. Die FinUI-Komponente `context-menu`.
- **Suche ohne Dialog**: Die Suche wächst aus deinem Menü (eine Seitenleiste verbreitert sich zu ihr, das Dock steigt zu ihr auf, der Name der Befehlsleiste wird zu ihr, das Menü eines Handys nimmt sie auf) oder nimmt, in Einstellungen → Darstellung gewählt, den Platz der Seite ein, während das Menü bleibt. Strg+Leertaste öffnet sie wie bisher; das Fenster über allem ist weg. Die FinUI-Komponente `search`.
- **Alles in einer Mediathek**: Die Karte Kürzlich hinzugefügt einer Mediathek bietet *Alles darin* an: jeden Film, jede Serie und jedes Album, das sie enthält, als Poster, nach Name, Jahr, Hinzufügedatum oder Größe, beim Tippen eingegrenzt. Weitere kommen beim Scrollen hinzu.
- **Server → Mediathek-Zustand**: FinStats vergleicht jede Datei mit ihren Nachbarn und listet auf, was heraussticht, jeweils mit den Zahlen dahinter. Es berichtet nur; in Jellyfin wird nichts geändert, gelöscht oder neu gescannt.
- **Lücken**: Episoden, die zwischen zwei Dateien einer Staffel fehlen („Staffel 2: Episoden 4 und 5 fehlen zwischen 3 und 6“). Eine Datei mit zwei Episoden zählt als beide; Specials und das Ende einer Staffel bleiben unberücksichtigt.
- **Abweichende Staffeln**, die sich in Auflösung, HDR oder Codec vom Rest ihrer Serie unterscheiden, und **Ausreißer-Episoden**: eine Datei, die nicht zu ihrer Staffel passt, etwa eine SD-Episode in einer 1080p-Staffel. Die Auflösung richtet sich nach Breite oder Höhe, sodass ein beschnittener Film mit 1920×800 als 1080p gilt.
- **Kopien und Versionen**: derselbe Film oder dieselbe Episode doppelt, erkannt an den Anbieter-IDs, nach denen sich Merkliste und Pipeline schon richten. Zwei Kopien in derselben Auflösung zeigen den doppelt belegten Platz; eine 4K-Kopie neben einer 1080p-Kopie wird als zwei Versionen geführt und nie als Verschwendung gezählt.
- **Dünne Dateien**: eine Bitrate weit unter dem, was Auflösung und Codec üblicherweise brauchen („4K HEVC mit 1,1 Mbps“); die dünnen Episoden einer Staffel sind ein einziger Befund.
- **Synchronfassungen, die abbrechen**: eine Audiosprache, die manche Staffeln auf jeder Episode haben und andere auf keiner.
- **Nie identifiziert**: Filme und Serien, die Jellyfin ohne TMDB-, TVDB- oder IMDb-ID zugeordnet hat.
- **Ausblenden** lässt sich ein Befund, der absichtlich so ist, mit einer Notiz. Er kommt von selbst zurück, wenn sich seine Dateien ändern. Ausblenden darf, wer FinStats verwalten und den Server sehen darf; Ausblendungen stehen im Audit-Protokoll und sind Teil einer Sicherung.
- Die Seite einer Mediathek sagt, wie viel es dort anzusehen gibt, und führt dorthin.
- **FinUI**, die Komponenten, aus denen FinStats gebaut ist, ist ein eigenes Projekt (`github.com/finstats/finui`): jede Schaltfläche, Karte und jedes Bedienelement in jedem Zustand und beiden Designs unter `finui.finstats.no`, dazu **FinUI create**, um Farben, Radius, Dichte und mehr live zu ändern und mit einem Befehl mitzunehmen. **Einstellungen → System** führt dorthin.
- **Einstellungen → Darstellung**: Alle wählen selbst, wie FinStats für sie aussieht, über den Code einer FinUI-Voreinstellung: Grundfarbe, Akzent, Diagrammfarben, Kontrast, Radius, Dichte, Rahmen, Karten, Schaltflächen, Felder, Tabellen, Hervorhebung, Bewegung, Symbole, das Menü, die Seite hinter allem, der Fokusring, die Gestaltung der Überschriften und die Schriften für Text, Überschriften und Zahlen. Deine Wahl gilt nur für dich; die Seite zeigt, was der Code bedeutet, und führt zu FinUI create, um ihn zu ändern.
- **Stile**: elf vollständige Looks als Ausgangspunkt, mit einer einzigen Wahl in FinUI create, dessen Code Einstellungen → Darstellung übernimmt. Es sind Washi (FinStats im Auslieferungszustand), Studio, Noir, Matinee, Console, Pastille, Ledger, Gazette, Arcade, Canopy und Harbour. FinStats selbst bringt einen Look mit und bietet keine Liste von Stilen an.
- **Symbole, die sich bewegen**: Jedes Symbol in einem Link, einer Schaltfläche oder einem Tab macht beim Daraufzeigen einmal seine eigene Bewegung (Aktualisieren dreht sich, der Pfeil von Herunterladen fällt in seine Ablage, der Papierkorb hebt den Deckel), und ein gedrücktes Aktualisieren dreht sich weiter, bis die Antwort da ist. Still bei reduzierter Bewegung und bei Bewegung: Aus.
- **Bewegung, mit FinMotion**: Die Komponenten von FinUI bewegen sich jetzt: Der Knopf eines Schalters wird hinübergeworfen, die Unterstreichung eines Tabs rückt zum nächsten Tab, ein abgelehntes Feld schüttelt den Kopf, ein Kopieren wird mit einem Strich abgehakt, Hinweise stapeln sich wie ein Blatt Spielkarten und fächern sich beim Daraufzeigen auf, das Blatt eines Handys gibt beim Ziehen nach wie ein Gummiband, die Zeilen einer sortierten Tabelle wandern an ihren Platz, eine sich ändernde Zahl rollt wie ein Kilometerzähler, und Diagramme zeichnen sich beim ersten Erscheinen selbst. Alles folgt der Wahl unter Bewegung in Einstellungen → Darstellung und bleibt bei reduzierter Bewegung still. FinMotion ist ein eigenes Projekt (`github.com/finstats/finmotion`), das auf FinUI aufsetzt.
- **Mehr Symbole in FinUI**: jetzt 239 statt 71, jedes mit einer eigenen Bewegung (lauter und leiser, ein Headset und ein stummgeschaltetes, ein Stift, der schreibt, eine Glocke, die läutet, ein Telefon, das vibriert), und jedes lässt sich aus der Galerie von FinUI als SVG herunterladen, still oder animiert.
- **Schriften**: vierunddreißig frei lizenzierte Schriftfamilien zur Wahl (Geist, Roboto, Manrope, IBM Plex Sans, Montserrat, Lora, Playfair Display, Fira Code und mehr), mitgeliefert wie Inter, nie aus dem Netz nachgeladen und nur geladen, wenn sie gewählt sind.
- **Mehr Grundfarben und Akzente**: Nord, Solarized, Gruvbox, Catppuccin, Rosé Pine, Tokyo Night, Dracula, Everforest und Ocean; sechzehn weitere Akzente, zehn davon tagsüber eine Farbe und nachts eine andere. Jede Grundfarbe hat jetzt tagsüber ein eigenes Papier: Im hellen Design sahen die Grautöne alle gleich aus.
- **Löschen lässt sich 30 Tage lang rückgängig machen**: Eine gelöschte Wiedergabe oder Sicherung wartet im Papierkorb, und **Rückgängig** im folgenden Hinweis holt sie zurück. Aktivität und Einstellungen → Sicherungen listen jeweils, was kürzlich gelöscht wurde und wann es endgültig verschwindet, sofern es etwas gibt. Eine Wiedergabe im Papierkorb fehlt in allen Summen, Diagrammen, Rückblicken und Profilen, und ein Import oder eine Wiederherstellung bringt sie nicht als neu zurück; wer sie mit jemandem geschaut hat, zählt als allein, bis sie zurückkehrt.

### Changed
- Dark Reader und ähnliche Erweiterungen lassen die Seiten von FinStats in Ruhe (`darkreader-lock`): FinStats zeichnet beide Designs selbst, und eine umgefärbte Seite kam ihnen in die Quere.
- Auf dem Handy gleitet das Menü nicht mehr von links herein: Die Tableiste sitzt unten, sofern kein anderes gewählt ist.
- FinStats liest deine Mediathek nach diesem Update einmal, um zu erfahren, wo Dateien mit mehreren Episoden enden.
- Auf der Seite eines Titels ist **Zur Merkliste hinzufügen** so hoch wie **In Jellyfin öffnen** daneben; vorher war es eine Größe kleiner.
- Die Zeitraum- und Personenfilter bleiben oben am Bildschirm, während eine Seite scrollt, sodass aus 7 T. auch vom Ende einer langen Seite aus Alle wird; ebenso die Filter für Anfragen und Demnächst in der Pipeline.
- Die Schaltfläche „Nach oben“ hält Abstand zu dem Menü, das gerade den unteren Fensterrand belegt: tief in der Ecke neben dem Dock, knapp über der Tableiste, dem Daumenknopf oder der Adressleiste eines Handys und sonst über der Statusleiste. Es ist die FinUI-Komponente `to-top`.
- Das Löschen einer Wiedergabe oder Sicherung verschiebt sie in den Papierkorb, statt sie sofort zu entfernen (`PUT` mit `{"deleted": true}`; `DELETE` ist aus der API verschwunden). Sicherungen über der Anzahl, die behalten wird, werden weiterhin endgültig entfernt.
- Die **Versionshinweise** sind ein Blatt statt eines Kartenstapels: Jeder Hinweis ist eine eigene Zeile, mit seiner Art (Neu, Geändert, Behoben und so weiter) und deren Symbol in einer Spalte davor, unter einer Überschrift pro Reihe. Die Versionsliste bleibt rechts.

### Performance
- Beim Öffnen lädt FinStats nur, was sich geändert hat: Ein Stylesheet, Skript oder eine Schrift, die der Browser schon hat, wird mit wenigen Bytes bestätigt statt erneut gesendet. Bisher holte jeder Besuch alle Stylesheets und Module von FinStats vollständig, obwohl jedes eine Kennung trug, an der man sie unterscheiden konnte.

### Stability
- Das Wiederherstellen einer Sicherung liest die Einstellungen nur aus der Einstellungszeile: Bei einer Datei aus einem neueren FinStats, das dort mehr speichert, gingen die Einstellungen wortlos verloren.
- Ein offener Tab wächst nicht mehr mit jeder besuchten Seite: Jede verlassene Seite blieb hinter der angezeigten im Speicher (rund 2.600 Elemente pro Besuch), festgehalten von der Liste dessen, was FinMotion bewegt, und vom Scroll-Listener der Filterleiste, bis der Tab geschlossen wurde.
- Ein vertippter Befehl (`finstats bakcup`) oder `restore` ohne Datei legt keine Datenbank mehr an und aktualisiert auch keine, bevor er sagt, dass er nicht weiß, was zu tun ist.

### Fixed
- Das kleine Diagramm einer Kachel läuft nicht mehr über die Kachel hinaus, wenn die Zahl daneben breit ist („56 T. 23 h“) oder wenn sich vier Kacheln eine schmalere Zeile teilen: Das Diagramm weicht, und die Zahl behält ihre Breite.
- Im hellen Design wurde eine Hauptschaltfläche unter dem Zeiger violett; jetzt behält sie die Farbe des Siegels.
- In den Einstellungen sitzt ein Schalter auf einer Höhe mit Name und Hilfetext seiner Einstellung; er hing oben in der Zeile.
- In einer Rangliste ohne Text unter einem Namen, etwa bei den Top-Benutzern der Übersicht, saß der Name oben in seiner Zeile neben einem zentrierten Bild; jetzt ist er zentriert.
- Eine Reihe Statistikkacheln in einer Breite zwischen Handy und Desktop (vier Kacheln, drei nebeneinander) ließ die letzte allein in einer eigenen Zeile; vier Kacheln stehen jetzt zu viert, zu zweit oder einzeln nebeneinander, drei zu dritt oder einzeln.
- Das Lesen des Servers zählte ein Gerät mit, an dem sich niemand angemeldet hatte und das es nicht speichert: Die Aufgabe nannte drei Geräte, wo die Serverseite zwei aufführte.
- Eine Sekunde, eine Minute, eine Stunde: Warnungen sagten „1 Sekunden auseinander“ und „in 1 Minuten“, und eine Jellyfin-Aufgabe mit einem Intervall von einer Minute lief „alle 1 Minuten“.
- `finstats --help` nennt jede Variable, die FinStats liest; `FINSTATS_PUBLIC_IP_URL`, `FINSTATS_ALLOW_LIBRARY_SHRINK` und `FINSTATS_SKIP_PREUPDATE_BACKUP` fehlten.
- Der Einrichtungsassistent meldete bei einer Adresse, die gar keine war, „Keine Verbindung“, in einem Satz, der in den nächsten überging; jetzt sagt er, dass das keine Adresse ist.
- In Einstellungen → Sicherungen verschwand eine Rückfrage, die gestellt wurde, während eine Sicherung geschrieben wurde, sobald die Sicherung fertig war.
- Beim Menü als aufklappende Leiste zeigte das Schließen der Suche die offene Suche und die geschlossene Leiste gleichzeitig und ließ die Leiste dann offen, bis der Zeiger kam und wieder ging. Esc gibt den Tastaturfokus jetzt dorthin zurück, wo er beim Öffnen der Suche war, und lässt den Fokus eines Klicks los.
- Auf dem Handy blieb die Seite beim Schließen der Suche abgedunkelt, bis die Suche verschwunden war, und wurde dann von einem Bild aufs nächste hell; die Abdunklung blendet jetzt mit aus.
- Die Schritte des Tautulli-Imports sagten, die Sicherung werde nach **Backup Database** heruntergeladen. Tautulli speichert sie auf seinem eigenen Rechner: Die Schritte sagen jetzt, wo (sein Ordner `backups`, `/config` mit Docker), welche Datei zu nehmen ist und dass sie herüberkopiert werden muss.
- Eine Suche, die geschlossen wurde, bevor sie fertig aufgegangen war, ließ die aufklappende Leiste ohne ihre Animation zurück: Von da an ging sie schlagartig auf und zu.
- Die aufklappende Leiste blieb nach der Wahl einer Seite offen, bis irgendwo anders geklickt wurde, und zeichnete in manchen Browsern oder beim Zoomen eine Bildlaufleiste unter ihre Symbole. Jetzt klappt sie nur für den Zeiger und die Tastatur auf und zeichnet im geschlossenen Zustand keine Bildlaufleiste.
- Auf der Seite Sicherheit zählt eine Adresse, die die Datenbank einem Land, aber keinem Punkt auf der Karte zuordnet, zu diesem Land statt zu den Adressen ohne bekannten Ort.

## [2.1.2] - 2026-10-01

### Added
- **Import aus Tautulli**: Hol deinen Plex-Verlauf herüber. Lade die Datenbanksicherung von Tautulli (die `.db` oder die `.zip`, die sie enthält) unter **Einstellungen → Import** hoch; ihre Wiedergaben haben im Filter *Erfasst von* der Aktivität einen eigenen Eintrag.
- **Ein Steckbrett, um festzulegen, wer wer ist**: Plex-Benutzer auf der einen Seite, Jellyfin-Benutzer auf der anderen. Zieh ein Kabel von jedem Plex-Benutzer zu der Person, die er heute ist, oder klicke erst den einen und dann den anderen an, oder nimm die Tastatur. Wer ohne Kabel bleibt, wird nicht importiert, und zwei Plex-Konten können in eine Person einfließen.
- Filme und Episoden werden deiner Mediathek über den Namen zugeordnet (ein Film über Titel und Jahr, eine Episode über Serie und Nummer), und was noch nicht auf deinem Server ist, wird aufbewahrt und zugeordnet, sobald es ankommt. Eine fortgesetzte Sitzung ist eine Wiedergabe; Musik wird nicht importiert.
- Die Sicherung enthält die Zugriffstokens von Plex: FinStats liest sie nie und entfernt die Datei, sobald du importierst oder neu anfängst.
- **Nicht zugeordnete Medien** unter **Einstellungen → Daten**: jeder Titel aus Tautulli, Jellystat oder Streamystats, der zu nichts in deiner Mediathek passt, mit seiner Herkunft und dem Titel, der ihm am wahrscheinlichsten entspricht. **Zuordnen** verknüpft ihn über die besten Vorschläge oder eine Suche (ein Film lässt sich als Special einer Serie zuordnen), und FinStats merkt sich die Wahl, sodass derselbe Verlauf sich bei einem erneuten Import selbst verknüpft. Die Zuordnungen sind Teil einer Sicherung.

### Performance
- **Der Rückblick öffnet drei- bis fünfmal schneller**: Die meistgeschauten Darsteller und Regisseure des Jahres wurden berechnet, indem das Jahr für jede darin vorkommende Person einmal neu durchsucht wurde. Gemessen an einem echten Verlauf ging ein Jahr von 0,29 s auf 0,09 s und die letzten 12 Monate von 0,53 s auf 0,12 s.

### Fixed
- **Importierter Verlauf findet Titel, die anders heißen**: Eine Wiedergabe passt auch zum Originaltitel eines Titels, sodass eine Serie, die Plex oder Tautulli als 오징어 게임 kannten, zu Squid Game findet und *Im Westen nichts Neues* zu *All Quiet on the Western Front*. Die Originaltitel deiner Mediathek werden beim nächsten Scan gelesen.
- Namen werden so verglichen, wie die Suche sie vergleicht (Groß- und Kleinschreibung, Akzente, Satzzeichen, eine in den Namen geschriebene Jahreszahl, ein vorangestelltes „The“), und ein Erscheinungsjahr darf um eins abweichen, sodass „Star Wars: Episode I - The Phantom Menace“ „… – The Phantom Menace“ findet und ein Kingsman von 2014 den von 2015. Ein Titel, der ganz anders heißt, wird weiterhin nie erraten.
- Ein fehlgeschlagener Streamystats-Import oder eine fehlgeschlagene Wiederherstellung wurde in Benachrichtigungen mit ihrem internen Namen angekündigt.
- Die Schaltfläche *Das Jahr als Karten teilen* im Rückblick und der Zuschauerrang daneben hatten keinen Abstand zueinander.

## [2.1.1] - 2026-10-01

### Changed
- **Die Merkliste wird überall angeboten, wo ein Titel ist**: auf den Karten unter Demnächst auf der Übersicht, wie in Pipeline → Demnächst, und auf der Seite einer Episode oder Staffel, die dann ihre Serie auf die Liste setzt.
- Auf einem Poster der Übersicht erscheint die Merklisten-Schaltfläche, während du auf die Karte zeigst oder sie mit der Tastatur erreichst, statt die Ecke jedes Posters zu verdecken; auf einem Touchscreen ist sie immer da.

### Stability
- Das Speichern einer Sonarr-, Radarr- oder Seerr-Verbindung, während jemand **Einstellungen → Ausgehende Verbindungen** geöffnet hatte, konnte beide für immer aufeinander warten lassen.
- **Einstellungen, die sich nicht lesen lassen, halten FinStats beim Start an** und nennen die betroffene Einstellung, statt still jede Einstellung auf ihren Standard zurückzusetzen.
- **Zwei gleichzeitige Änderungen an den Einstellungen** (etwa die Einstellungsseite und der Zeitplan einer Aufgabe) heben sich nicht mehr gegenseitig auf.
- Eine Pause, ein Sprung oder ein Spurwechsel, der gesehen wurde, während die Datenbank beschäftigt war, wird beim nächsten Durchlauf geschrieben, statt verloren zu gehen.
- Das Lesen einer Mediathek scheitert nicht mehr an einer Merkliste mit mehreren Einträgen, die sich als ein und derselbe Titel herausstellen.

### Fixed
- **Eine ersetzte Datei ist nicht „jetzt auf dem Server“**: Ein Upgrade der Datei eines Films auf jemandes Merkliste meldet nicht mehr, er sei gerade angekommen.
- Ein einzelnes gescheitertes Lesen der Plugins, geplanten Aufgaben oder Speicherdaten von Jellyfin leert diese Liste auf der Serverseite nicht mehr bis zum nächsten Lesen.
- **Eine Streamystats-Wiedergabe endet auf jeder Seite dort, wo sie endete**: Die Abbruchkurve eines Titels und das „beendet“ einer Episode richteten sich stattdessen nach ihrer Länge und zählten sie als geschätzt.

## [2.1.0] - 2026-09-29 to 2026-09-30

Merkliste

### Added
- **Eine eigene Merkliste**: Setz einen Film oder eine Serie von seiner Seite, aus Demnächst, aus Kürzlich hinzugefügt oder aus der Suche (Strg+Eingabe) darauf, und finde sie unter **Merkliste** in deinem eigenen Profil.
- **Jeder Eintrag sagt, wo er gerade steht**: auf dem Server, „5 von 26 Episoden“, angefragt, „Demnächst am Freitag“, nicht auf dem Server, hat die Mediathek verlassen oder geschaut. Das wird bei jedem Ansehen neu ermittelt, mit derselben Bedeutung von „gesehen“ wie bei deinem Serienfortschritt. Geschaute Titel bleiben in einer eigenen Gruppe, bis du sie entfernst.
- **Ein Titel, der noch nicht auf deinem Server ist**, kann direkt aus dem, worauf Sonarr oder Radarr wartet, auf die Liste. Er wird von selbst dem Titel zugeordnet, wenn er ankommt, und eine umbenannte Datei nimmt ihren Eintrag mit.
- **„Auf deiner Merkliste: jetzt verfügbar“**: eine Benachrichtigung an deine eigenen Ziele, wenn etwas ankommt, auf das du gewartet hast. Aus, bis du sie ankreuzt.
- **Niemand sonst sieht deine Merkliste**, auch kein Administrator, und ihre Benachrichtigung geht nur an deine Ziele. Nichts wird in Jellyfin, Seerr, Sonarr oder Radarr geschrieben.
- Merklisten sind Teil einer Sicherung, und eine Wiederherstellung führt sie zusammen, ohne etwas zu verdoppeln.

### Changed
- Einträge von `/api/upcoming` tragen die TVDB- und TMDB-IDs ihres Titels, und `/api/auth/me` sagt, ob der Server Merklisten führt.

## [2.0.4] - 2026-09-29

### Added
- **Aufgabenplanung wie in Jellyfin**: Jede Aufgabe, die FinStats von selbst erledigt, hat Auslöser (täglich zu einer Uhrzeit, wöchentlich an einem Tag, in einem Intervall, beim Start oder nach dem Mediathek-Scan von Jellyfin), jeweils mit optionaler Zeitbegrenzung. Klicke auf eine Aufgabe, um ihren Zeitplan zu sehen und zu ändern; *Auslöser hinzufügen* öffnet denselben Dialog wie in Jellyfin.
- **Aufgaben ist ein eigener Bereich** in den Einstellungen, mit dem letzten Lauf jeder Aufgabe in Worten („Zuletzt vor 38 Minuten gelaufen, Dauer 6 Minuten“) und einer Schaltfläche zum Ausführen; vorher stand das oben unter System.
- **Metadatenänderungen**: Jede Stunde liest FinStats, was Jellyfin seit dem letzten Mal gespeichert hat (Namen, Beschreibungen, Genres, Bewertungen, Besetzung und Crew, Dateidetails und Bilder), sodass eine Bearbeitung in Jellyfin erscheint, ohne auf einen Mediathek-Scan zu warten.
- Sicherung und Geolokalisierungsdatenbank lassen sich ebenfalls unter Aufgaben ausführen; Esc führt vom Zeitplan einer Aufgabe zurück zur Liste.

### Changed
- **Das Design ist ein Schalter mit drei Stufen**, Hell · Gerät · Dunkel, wo vorher eine Schaltfläche durch sie wechselte: Klicke auf ein Drittel davon, zieh den Knopf oder nimm die Pfeiltasten. Der Knopf trägt das Symbol der Wahl, das sich beim Wechsel hineinbewegt.
- **Abmelden steht unter Einstellungen → Konto**, wo auch steht, wer hier angemeldet ist; die Seitenleiste behält den Designschalter.
- **Die alten Zeitplan-Einstellungen sind jetzt Auslöser**: *Dem Mediathek-Scan von Jellyfin folgen* mit seinem Ersatzintervall, *Alle N Tage sichern* und der GeoIP-Schalter *Datenbank aktuell halten* wurden zu den passenden Auslösern ihrer Aufgaben, mit den Werten, die du hattest. Erfassung, Sicherungen und Sicherheit verlinken stattdessen auf den Zeitplan.
- Ein Start führt nicht mehr jede Aufgabe erneut aus: Wann jede zuletzt fertig wurde, wird gespeichert.
- Eine Jellyfin-Aufgabe ohne bisherige Schätzung zeigt „noch etwa …“, mit wechselnden Punkten, wo die Zeit stehen wird, statt „ETA…“.

### Performance
- **Die Karte unter Sicherheit ruckelt nicht mehr**: Ziehen und Zoomen verschieben die bereits gezeichnete Karte und zeichnen sie einmal neu, wenn die Geste zur Ruhe kommt, und das Pulsieren der laufenden Wiedergaben lässt nicht mehr die ganze Karte 120-mal pro Sekunde neu zeichnen, während niemand sie berührt. Ein Ziehen ging auf einem langsamen Computer von 34 auf 60 Bilder pro Sekunde.

### Fixed
- **Ein in Jellyfin ersetztes Poster oder Porträt erscheint in FinStats**: Bilder wurden eine Woche auf dem Server und eine weitere Woche im Browser zwischengespeichert, egal was Jellyfin inzwischen hatte. Jetzt werden sie unter dem Bild-Tag von Jellyfin zwischengespeichert, sodass ein neues Bild eine neue Datei ist, und der Browser prüft bei jedem Anzeigen, ob seine Kopie aktuell ist. Ein neues Porträt speichert nur die Person neu, deshalb liest Metadatenänderungen auch die Personen.
- **Server → Jobs zeigt eine Schätzung, sobald die Seite öffnet**: Die Restzeit wurde nur aus Messungen ermittelt, die die geöffnete Seite selbst machte, sodass jeder Besuch mit „ETA…“ begann, bis die Seite den Prozentwert hatte wandern sehen. FinStats misst eine laufende Jellyfin-Aufgabe jetzt selbst, alle 10 Sekunden, solange eine läuft, und die Seite bekommt die Schätzung geliefert. Eine Aufgabe, die Jellyfin die ganze Zeit mit 0 % meldet (die erste Phase eines Mediathek-Scans), behält ihre Punkte, bis sie sich bewegt: Nichts wird geraten.
- **Tooltips auf der Karte erscheinen am Punkt**: In Firefox konnte der Tooltip des Punkts einer laufenden Wiedergabe weit entfernt landen, und wo Punkte sich überlagern (jemand schaut zu Hause), hing es an einem Pixel, welcher Tooltip erschien. Jetzt nennt ein einziger Tooltip alles unter dem Zeiger.
- **Die Statusleiste hat eine einzige Hover-Form**: Jeder Link (die Streams, Repo, die Version) leuchtet als dasselbe kleine Kästchen auf, und die Punkte zwischen den Einträgen sitzen mittig in den Lücken. Wer auf die Version zeigte, ließ vorher auch den Punkt und die Lücke davor aufleuchten.
- **Ein Dateipfad nimmt die ganze Breite seiner Karte ein**: Auf der Karte Datei eines Titels und bei den Ordnern von Jellyfin auf der Serverseite wurde ein Pfad in eine schmale Zelle gequetscht und lief neben leerem Platz neun Zeilen hinunter.
- Ein Dialog, der gerade ausblendet, schluckt keinen Klick auf die Seite darunter mehr.

## [2.0.3] - 2026-09-28

### Security
- Die Serverseite gab jedem mit *Serverdetails sehen* die ID jedes Geräts; IDs erfordern wie überall sonst auch *Netzwerkdetails sehen*.
- Zwei Berechtigungsschalter, die umgelegt wurden, bevor der erste gespeichert war, sandten zweimal dieselbe Ausgangsliste, und der zweite machte den ersten rückgängig: Eine entzogene Berechtigung blieb erteilt, ein abgeschalteter Abschnitt des öffentlichen Profils blieb veröffentlicht, während beide Schalter das Gewählte zeigten.

### Stability
- **Nur ein Import oder eine Wiederherstellung zur Zeit**: Zwei gleichzeitig gesendete Uploads bestanden beide die Prüfung, und eine Wiederherstellung achtete nie auf Importe; der zweite wartete vergeblich auf die Datenbank und scheiterte nach Minuten des Hochladens.
- **Eine Adresse mit einem kaputten `%`-Escape öffnet die Seite „Nicht gefunden“**, statt für immer einen Ladekreisel zu zeigen.
- **Eine große Seitenzahl in einer Liste ist eine leere Seite**: Sie lief über und lieferte wieder die erste Seite.

### Fixed
- **Tabellen sortieren Zahlen in einem deutschen oder norwegischen Browser richtig**: „1,204“ wurde als Kommazahl 1,204 gelesen statt als 1204, und alles über einer Million als Text.
- **Die Tabellenansicht eines Diagramms sortiert Datumsangaben, Wochentage und Positionen zeitlich**, nicht nach ihren Buchstaben.
- **Der Download-Verlauf nennt, woher Importe kamen**: Sonarr und Radarr nennen Indexer und Protokoll nur beim Grab, deshalb waren diese Aufschlüsselungen immer leer. Ein gescheitertes Staffelpaket ist ein Fehlschlag, nicht einer pro Episode.
- **Das Löschen einer gemeinsam geschauten Wiedergabe** ermittelt die Gruppen dieses Titels neu, statt eine „Gruppe“ aus einer Person zurückzulassen.
- **Gemeinsame Zeit ist für eine Person nie mehr, als sie geblieben ist**: Die Seite Gemeinsam schrieb ihr die des ganzen Abends gut.
- **Ein Abend in Gesellschaft über Mitternacht an Silvester** (oder über den Rand von „letzte N Tage“) gehört ganz zu dem Zeitraum, in dem er begann, statt zu zwei Abenden allein zu werden.
- **Importierte Episodenwiedergaben tragen den Namen der Episode**, wo Jellystat nur den der Serie aufbewahrt hatte.
- **Die Geolokalisierung findet die Städtedatenbank neben den Country- und ASN-Datenbanken**, die `geoipupdate` schreibt, statt sich selbst abzuschalten.
- **Der Hotspot des häufigsten Zurückspulens folgt den Filtern der Seite**, wie seine Zahlen.
- **„Dein Jahr ist fertig“** geht nur an Personen, deren Rückblick etwas enthält.
- **Wird eine stummgeschaltete Warnung erneut erledigt, bleibt ihr Ortspaar stummgeschaltet**; nur Wiedereröffnen hebt eine Stummschaltung auf.
- **Ein abgelaufener API-Schlüssel zählt nicht mehr zu den zwanzig.**
- **Das Sitzungslimit lässt die älteste Sitzung gehen**, auch wenn viele Anmeldungen in dieselbe Sekunde fallen.
- **Eine von der Abfrage gefundene Heimadresse bleibt**, wenn dieselbe Adresse aus der eingetippten Liste entfernt wird.
- **Bilder im Audit-Protokoll und in der Schlüsselliste**, wo nur Initialen standen.
- **Die Benachrichtigungsliste eines Administrators sagt, wem ein persönliches Ziel gehört**, statt bei allen „Deins“; **Kürzlich gesendet** zeigt jedes Ziel seines letzten Ereignisses.
- **Esc auf einem Titel oder einer Person verlässt FinStats nie** und schließt nur den obersten Dialog.
- **Eine Seite wird auch gezeichnet, wenn ein Speichern das Vorladen abbricht, auf das sie wartete.**
- **„Läuft gerade“ zeigt einen pausierten Zuschauer, der spult,** an der Stelle, an der er jetzt ist, nicht erst, wenn er auf Wiedergabe drückt.
- **Das Entfernen einer Verbindung aktualisiert, welche Seiten es gibt**, wie es das Speichern einer Verbindung tut.
- **Das Erstellungsdatum der Geolokalisierungsdatenbank behält sein Jahr** in US-Englisch.

## [2.0.2] - 2026-09-28

### Security
- Hinter einem Proxy zählte das Anmeldelimit die erste `X-Forwarded-For`-Adresse, also die, die der Client sendet: eine neue erfundene Adresse pro Versuch, und das Limit griff nie. Jetzt ist es die letzte, die der Proxy geschrieben hat.
- *IP-Adressen und Orte einschließen* sandte Adressen an ein persönliches Ziel, dessen Inhaber kein *Netzwerkdetails sehen* hat und dem keine Seite sie zeigen würde. Fehlgeschlagene Anmeldungen anderer erfordern jetzt auch *Serverdetails sehen*, wie auf der Seite Sicherheit.

### Stability
- **Ein Anhalten geht nie verloren**: Wurde das Lesen einer Mediathek in den ersten Augenblicken nach dem Start verweigert, bevor der Server darauf wartete, wurde die Verweigerung notiert, und FinStats lief weiter, statt anzuhalten.
- **Ein Jellyfin, das in einer Form antwortet, die FinStats nicht lesen kann, wird abgelehnt**, statt ihm zu glauben: Jeder Benutzer oder jeder Titel einer Mediathek, der unter unbekannten Schlüsseln ankam (`id` für `Id`), passierte die Schutzprüfung mit voller Anzahl und wurde dann als entfernt markiert.
- **Das Ende einer Wiedergabe geht nie verloren**: War die Datenbank länger beschäftigt, als ihre Wartezeit reicht (weil ein Import sie hielt), blieb eine Wiedergabe, die in diesem Moment endete, bis zum nächsten Neustart als laufend markiert. Jetzt wird sie beim nächsten Durchlauf geschrieben.
- **Einstellungen, die eine Wiederherstellung nicht lesen kann, bleiben unangetastet**: Eine Sicherung, deren Einstellungen nicht mehr zu dieser Version passten, setzte bei der Wiederherstellung jede Einstellung zurück: Zusammenführungsfenster, Berechtigungen, Heimadressen, Zeitplan.
- **Ein Steuerzeichen in einem Namen oder Titel macht die Karten nicht mehr kaputt**: Die Profilkarte, jede Story-Karte mit diesem Titel und das ZIP davon antworteten mit einem Fehler.

### Fixed
- **Benachrichtigungen enden für jemanden, der in Jellyfin deaktiviert oder gelöscht wurde**: Seine persönlichen Ziele bekamen weiter alles, was seine alten Berechtigungen erlaubten.
- **Eine Benachrichtigung in der Warteschlange wird vor dem Senden erneut geprüft**: Ein abgeschaltetes Ziel oder ein Inhaber, der eine Berechtigung verloren hatte, bekam weiterhin, was auf einen neuen Versuch wartete.
- **Ein Ziel, das immer mit `Retry-After` antwortet, wird aufgegeben**, nach fünf Versuchen wie jedes andere, statt dreißig Tage lang erneut versucht zu werden.
- **Geräte-IDs auf der Seite einer Person** werden nur mit *Netzwerkdetails sehen* gezeigt, wie überall sonst.
- **Ein veröffentlichtes Jahr wartet bei allem einen Tag**: Seine beendeten und abgebrochenen Serien folgten jeder Wiedergabe, sodass ein Druck auf Wiedergabe bei einer abgebrochenen Serie die öffentliche Seite binnen Sekunden änderte.
- **Eine angefragte Serie zählt als geschaut**, wenn eine ihrer Episoden es ist: Jede angefragte Serie galt als nie geschaut, im Kapitel Anfragen und auf der Karte Gewünscht.
- **Das Kapitel Gemeinsam im Rückblick lässt Live-TV aus**, wie der Rest des Jahres: Ein gemeinsam laufen gelassener Sender konnte zum Toptitel werden und der Anteil 100 % übersteigen.
- **Die Karte Tage von „den letzten 12 Monaten“ endet am neuesten Tag**: Das Raster ließ die jüngsten drei oder vier Wochen weg.
- **Neue Episoden einer Serie werden einmal pro Abend angekündigt**: Episoden beiderseits von Mitternacht UTC wurden auf einem Server östlich oder westlich davon doppelt angekündigt.
- **Das Verbinden von Seerr kündigt alte Anfragen nicht als schaubereit an**: Eine Anfrage, die beim ersten Lesen schon verfügbar war und sonst keine Angabe zum Zeitpunkt hatte, galt als in diesem Moment angekommen.
- **Die Titel einer in Jellyfin gelöschten Mediathek verschwinden mit ihr**: Sie blieben in der Suche, im Regal Kürzlich hinzugefügt und in den Summen.
- **Ein Remux bleibt direktes Streaming**, wenn eine Messung kurz ihre Transkodierungsdetails verliert (beim Spulen), statt für den Rest der Wiedergabe zur Transkodierung zu werden.
- **„Jellyfin ist nicht erreichbar“ bedeutet, dass mehrere Lesevorgänge nacheinander scheiterten**: Vereinzelte Zeitüberschreitungen, Tage auseinander, auf einem Server, der ruhig am Socket hing, summierten sich dazu.
- **Ein Streamystats-Import zeigt seinen Fortschritt auf seiner eigenen Karte**, nicht über dem letzten Ergebnis des Jellystat-Imports.
- **Eine Adresse, deren Host mit „web“ beginnt** (`http://web:8096`), wird als Host gelesen und nicht als Pfad des Jellyfin-Webclients abgeschnitten.
- **Ein Fehler in der Warteschlange blendet die gesamten Zugangsdaten** einer Adresse aus, auch wenn das Passwort einen Schrägstrich enthält.

## [2.0.1] - 2026-09-28

### Changed
- **Die Sprachen eines Titels sind nicht mehr im Weg**: Der Kopfbereich nennt die ersten drei Audio- und Untertitelsprachen, je eine Zeile, und eine neue Karte **Sprachen** listet alle auf: die vollständigen zusammen, die teilweisen mit der Anzahl der Episoden, die sie haben. Die zwei Dutzend Untertitelsprachen einer langen Serie schoben das Artwork früher einen Bildschirm nach unten.

### Fixed
- **Das Poster eines Titels bleibt neben seinem Namen**: Lief der Text daneben länger als das Poster (eine lange Inhaltsangabe, viele Studios), sank das Poster an den unteren Rand des Kopfbereichs, unter leeres Artwork.
- **Eine Version, die FinStats nicht lesen kann, heißt auch so**: Ein von Hand zu etwas Unlesbarem geändertes `app_version` ließ FinStats den Start als „das ältere FinStats 2.0.1“ verweigern, was es nicht war. Es verweigert weiterhin, sagt aber jetzt, welchen Wert es nicht lesen konnte und wie er zu schreiben ist.

## [2.0.0] - 2026-09-25 to 2026-09-28

Rückblick 2026, öffentliche Profile und gemeinsames Schauen

### Added
- **Mit wem du geschaut hast**, im Rückblick: Abende in Gesellschaft, die darin verbrachte Zeit, der Titel, der am häufigsten Leute zusammenbrachte, und bis zu drei Begleiter, in der App mit Namen und mit ihren Profilen verlinkt.
- **Bis zum Ende gesehene Serien**, im Rückblick: die in diesem Jahr gesehene letzte Episode jeder Serie, nach derselben Bedeutung von „gesehen“ wie bei den Fortschrittsbalken im Profil, und auch die angefangenen und für später liegen gelassenen: weniger als die Hälfte gesehen und zwei Monate lang nichts abgespielt.
- **Was du dir gewünscht hast**, im Rückblick: im Jahr gestellte Seerr-Anfragen, wie viele davon ankamen und wie viele du dann geschaut hast. Nur, wo es Anfragen gibt.
- **Das Jahr im Vergleich zum Vorjahr**: Stunden, Wiedergaben und Tage, und wie viel mehr oder weniger.
- **Das Jahr als Story**: jedes Kapitel als Karte in 1080×1920, dem Format von Handy-Storys (das Jahr, die Zahlen, Top-Serien, -Filme und -Musik, Genres, die Persona, Stunden und Tage, jeder Tag des Jahres, Rekorde, Gesellschaft, beendete Serien, Anfragen und das Vorjahr). Speichere eine oder alle als ZIP über „Das Jahr als Karten teilen“.
- **Eine Karte nennt nie eine andere Person**: Karten werden aus einer Kopie des Jahres gezeichnet, die kein Feld für eine andere Person, einen Rang oder eine App hat. In der App trägt eine Karte überhaupt keinen Namen.
- **Das Jahr des Servers**, für Jellyfin-Administratoren: alle Titel und Summen und die Persona des Hauses, ohne dass jemand eingestuft oder genannt wird. Ein Schalter auf der Seite des Rückblicks, `scope=server` in der API.
- **„Dein Jahresrückblick ist fertig“**: eine Benachrichtigung im Dezember, einmal an jede Person, die in diesem Jahr geschaut hat.
- **Öffentliche Profile**: Eine Person kann einen Teil ihres Profils unter einem Link veröffentlichen, den jeder ohne Konto öffnen kann (Summen und Toptitel, Tage in Folge und wann geschaut wird, das ganze Jahr als Story aus Karten, die letzten zehn Wiedergaben), jeder Teil aus, bis die Person ihn unter Einstellungen → Öffentliches Profil einschaltet. Geräte, Adressen, Apps, Wiedergabemethoden, Dateipfade und andere Personen werden nie veröffentlicht, egal was eingeschaltet ist, und der angezeigte Name ist einer, den die Person selbst eintippt, nie ihr Jellyfin-Login.
- **Nichts auf einer veröffentlichten Seite ist neuer als einen Tag**, die letzten Wiedergaben genauso wie jede Summe, sodass eine Seite, die jemand im Auge behält, nie verrät, wer gerade schaut.
- **Teilbare Karten**: Zu jedem veröffentlichten Profil gehört ein Bild in 1200×630, vom Profil und vom Jahr, das Discord, Signal, Mastodon und die anderen zeigen, wenn der Link eingefügt wird, und das sich zum Posten herunterladen lässt. Auf dem Server gezeichnet aus dem, was die Seite zeigt, und aus nichts weiter.
- **Ein Link, den du zurücknehmen kannst**: 22 zufällige Zeichen, mit einem Klick zurückgesetzt; der alte Link antwortet dann genau wie einer, den es nie gab.
- **Öffentliche Profile sind aus, bis ein Administrator sie erlaubt**: ein Schalter für den ganzen Server, eine Liste, wer was veröffentlicht, und eine Möglichkeit, ein Profil vom Netz zu nehmen. Veröffentlichungen, Änderungen, Zurücksetzungen und Sperrungen stehen im Audit-Protokoll, und ein Schlüssel kann nichts öffentlich machen.
- **Gemeinsam**, eine eigene Seite: wer mit wem schaut, wie viel und wie sich das verändert hat. Gemeinsame Zeit, Abende und der Anteil der Wiedergabezeit in Gesellschaft, jeweils gegenüber dem Zeitraum davor; Stunden in Gesellschaft gegenüber Stunden allein pro Tag oder Woche; jedes Paar von Personen mit seinen Abenden, der gemeinsamen Zeit, wann es zuletzt geschaut hat und was es am meisten schaut; die am häufigsten gemeinsam geschauten Titel; die letzten Abende; und, für alle, die alle sehen dürfen, die Zeit jeder Person allein gegenüber ihrer Zeit in Gesellschaft.
- Jedes Paar zählt: Ein Abend zu dritt zählt für jedes seiner drei Paare, wo die Karte früher nur genau dieselbe Personengruppe zählte. Die gemeinsame Zeit eines Paars ist der kürzere der beiden Aufenthalte.
- Die Karte **Gemeinsam geschaut** auf der Übersicht und in einem Profil verlinkt auf die Seite, und die Karte eines Profils sagt, welcher Anteil der Wiedergabezeit dieser Person in Gesellschaft war.
- `GET /api/stats/groups` bekommt `pairs`, `series`, `people`, `previous`, `bucket` sowie `watch_s` und `share` in seinen Summen; die bisherigen Schlüssel bleiben unverändert.
- **Wo Leute aufhören**, auf jeder Film- und Episodenseite: der Anteil der Wiedergaben, die zu jeder Minute noch laufen, von allen am Anfang bis zu denen, die das Ende erreicht haben, mit dem Zurückspulen und dem Einschalten von Untertiteln dieser Wiedergaben auf derselben Achse markiert. Die Karte sagt, wie viele Ausstiege gemessen (von FinStats aufgezeichnete Wiedergaben) und wie viele geschätzt sind (importierte Wiedergaben, nach ihrer Laufzeit), und braucht drei Wiedergaben, bevor sie etwas zeichnet.
- **Wer dranbleibt**, auf einer Serienseite: wie viele Personen jede Episode angefangen haben, der Reihe nach, und eine Spalte **Beendet** pro Episode. „Alle hören bei Episode drei auf“ heißt: Leute, die bei Episode vier nie auf Wiedergabe drücken.
- **Dateien, die nie abspielen**, unter Wiedergabe: ein Film oder eine Episode, dreimal oder öfter gestartet und nie über dreißig Sekunden hinausgekommen, samt denen, die es versucht haben, und ihren Apps. Das Protokoll von Jellyfin enthält keine Wiedergabefehler, also sind die Wiedergaben die Zeugen; die Mindestdauer einer Wiedergabe gilt hier nicht, denn genau die erreichen diese Wiedergaben nie.
- **Am häufigsten zurückgespult** und **Untertitel eingeschaltet**, unter Wiedergabe: Titel nach Rücksprüngen pro Wiedergabe gereiht, mit der Minute, in der sie sich häufen, und nach dem Anteil der Wiedergaben, bei denen Untertitel in den ersten zehn Minuten eingeschaltet wurden. Alle drei Listen erfordern *Aktivität aller sehen*. Sprünge unter 20 Sekunden wurden nie aufgezeichnet, ein kurzes Wiederholen einer einzelnen Zeile ist also absichtlich unsichtbar.
- Ein Sprung behält, woher er kam, als Zahl neben seiner Beschriftung (`from_s` in der Zeitleiste einer Wiedergabe). Jeder bereits aufgezeichnete Sprung bekommt seine Zahl aus der Beschriftung, ebenso eine wiederhergestellte Sicherung von davor.
- `GET /api/stats/files` für die drei Listen; `GET /api/items/{id}` bekommt `insights`, und die Episoden einer Serie bekommen `users` und `finished`.
- **API-Schlüssel**: Alle Angemeldeten können sich unter Einstellungen → API-Schlüssel eigene Schlüssel erstellen, für ein Skript, ein Dashboard oder ein Handy. Ein Schlüssel wird als `Authorization: Bearer fs_…` gesendet, nur einmal angezeigt, gehasht gespeichert, kann ein Ablaufdatum tragen und wird mit einem Klick widerrufen; ein Administrator sieht alle und kann jeden widerrufen. Er steht für dieselbe Person mit denselben Berechtigungen wie eine Anmeldung, bei jeder Anfrage live gelesen, sodass der Verlust des Anmelderechts sofort jeden Schlüssel stoppt. Ein Schlüssel kann keine Schlüssel erstellen oder widerrufen.
- **Ein Kalender-Feed**: `GET /api/calendar.ics?key=…` liefert, was bei Sonarr und Radarr ansteht, als iCalendar, das ein Handy abonnieren kann. Ein Schlüssel mit dem Bereich *Kalender* öffnet den Feed und sonst nichts; der Feed liest den Schlüssel aus der Adresse, weil ein abonnierter Kalender keinen Header senden kann, nie aus dem Cookie, und nennt niemanden.
- **Ein Audit-Protokoll** unter Server → Audit, für Jellyfin-Administratoren: jede Anmeldung und jeder Fehlversuch mit Adresse, jede geänderte Einstellung oder Berechtigung und der neue Wert, jeder erstellte, benutzte oder widerrufene Schlüssel, jede Verbindung, jedes Ziel, jede Sicherung und jeder Import, jede gelöschte Wiedergabe und jede erledigte Warnung, jeweils mit Person, Herkunft, verwendetem Schlüssel und ob es geklappt hat. Ein Jahr lang aufbewahrt, Teil von Sicherungen. `GET /api/audit` blättert durch.
- **Import aus Streamystats**, ein eigener Bereich in den Einstellungen neben Jellystat. Nimm die Datei aus dessen **Settings → Backup & Import → Download Backup** und zieh sie hinein: Jahre an Verlauf kommen in ein paar Sekunden an, und nichts, was du schon hast, wird erneut gezählt. `finstats import-streamystats <file>` erledigt das ohne Oberfläche, `POST /api/import/streamystats` über die API; es läuft immer nur ein Import gleichzeitig, egal welcher Art.
- **Beide Tracker genutzt? Importiere beide Dateien.** Eine Wiedergabe wird als eine erkannt, die FinStats schon hat, an der eigenen ID des Trackers und, falls das nicht greift, daran, dass dieselbe Person dasselbe Element schaut und eines der beiden Enden der Wiedergabe nah an einer im Verlauf liegt. So lassen sich ein Jellystat-Export und ein Streamystats-Export derselben Abende beide importieren, in beliebiger Reihenfolge, und ein Abend, den FinStats selbst aufgezeichnet hat, wird von keinem überschrieben. „Nah“ ist das Zusammenführungsfenster unter Einstellungen → Erfassung.
- Importierte Zeilen sagen, was sie nicht sagen können. Eine Wiedergabe, die Streamystats selbst aufgezeichnet hat, enthält nichts über die Datei (keinen Codec, keine Auflösung, keine Bitrate, keinen Container), daher bleiben diese Felder leer, statt mit einer Vermutung gefüllt zu werden, und die Karten, die sie zeigen würden, blenden sich aus. Eine Wiedergabe, die es von Jellystat übernommen hatte, trägt die ganze Sitzung, und die wird stattdessen gelesen: datiert auf den Moment, in dem sie endete, den einzigen, den einer der beiden Tracker behalten hat, und nur dann als Transkodierung beschrieben, wenn die Sitzung wirklich eine war.
- Zeilen, die Streamystats schrieb, weil Jellyfin ein Element als gesehen meldete, für eine Wiedergabe, die es nie gesehen hat, werden gezählt und übersprungen: Sie sind so lang wie der ganze Film und wären Wiedergabezeit an einem Abend, an dem niemand geschaut hat. Das Ergebnis sagt, wie viele es waren.
- Wie Streamystats-Daten Feld für Feld gedeutet werden, gelernt an einem echten Export statt aus einer Dokumentation: `docs/streamystats-import.md`.
- **Von welchem Tracker jede Wiedergabe stammt**, als Filter in der Aktivität: was FinStats selbst aufgezeichnet hat, was von Jellystat kam, was von Streamystats kam. Angeboten wird nur, was dein Verlauf wirklich enthält, und der Filter ist eingegrenzt wie jede andere Liste, sodass jemand, der nur seine eigenen Wiedergaben sehen darf, auch nur erfährt, woher sein eigener Verlauf kam.
- **Die Filter in der Aktivität sind Auswahllisten zum Ankreuzen**, sodass du Filme *und* Episoden ohne Musik, zwei Wiedergabemethoden oder zwei Personen auf einmal abfragen kannst, statt je eines oder alle. Der Medientyp hat **Sonstige** dazubekommen, jetzt, wo er sich kombinieren lässt.
- Mehrere Personen auf einmal, überall, wo der Benutzerfilter erscheint (die Übersicht, Wiedergabe, Mediatheken, Sicherheit und der Rest), und die Karte sowie jede Statistik folgen. Ein Benutzerfilter grenzt weiterhin nur ein, was jemand sehen darf: Ohne *Aktivität aller sehen* ist eine Anfrage auf die anfragende Person festgelegt, egal wen und wie viele die Adresse nennt.
- **Ein helles Design**: Washi-Papier, mit Tintenschrift und einem zinnoberroten Akzent. FinStats folgt der Hell- oder Dunkel-Einstellung des Geräts, bis du eines wählst.
- **Eine Design-Schaltfläche neben Abmelden** wechselt zwischen Gerät, Hell und Dunkel. Die Wahl wird in diesem Browser gespeichert und angewendet, bevor die Seite gezeichnet wird, sodass sie nie im anderen Design aufblitzt.
- **In Jellyfin öffnen**: Jede Film-, Serien- und Episodenseite hat eine Schaltfläche, die die Seite dieses Titels in Jellyfin in einem neuen Tab öffnet.
- **Jellyfin-Adresse für Personen** unter Einstellungen → Jellyfin: wohin diese Schaltflächen zeigen, für den Fall, dass die Adresse, mit der FinStats sich verbindet, keine ist, die ein Browser erreichen kann. Leer behält die Adresse, mit der FinStats sich verbindet; nur ein Jellyfin-Administrator kann sie setzen.

### Changed
- **FinStats hat ein neues Zuhause**: `github.com/finstats/finstats`, und das Image ist `ghcr.io/finstats/finstats`. Ändere den Image-Namen in deinem `docker run` oder deiner Compose-Datei, um weiter Updates zu bekommen; der Datenordner bleibt, wie er ist.
- **Die Einstellungen zeigen jeweils einen Bereich.** Die Seite bestand aus vierzehn Karten in einer Spalte und sechshundert Wörtern Hilfe. Jetzt hat jeder Bereich seine eigene Adresse (`/settings/collection`), eine Liste links wechselt zwischen ihnen, und auf dem Handy wird die Liste zu Chips. Links auf die alten Anker landen weiterhin richtig.
- **Eine Einstellung ist eine Zeile**: was sie ist, eine Zeile mit der Regel, das Bedienelement rechts. Zahlen in einem Bereich werden gemeinsam mit einer Schaltfläche unten gespeichert; Schalter speichern sich weiterhin selbst. Die Statuskarten (Aufgaben, Ausgehende Verbindungen, Datenbank) und der Link zu den Lizenzen stehen unter **System**, und die beiden Importe haben einen eigenen Bereich.
- **Eine Einstellung finden** über das Feld in der Kopfzeile (`/` setzt den Fokus dorthin), mit jedem Wort aus ihrem Namen; es sagt, in welchem Bereich die Einstellung steht, und Eingabe springt zur Zeile.
- **Die Einstellungen sind für alle Angemeldeten da**: Die Bereiche, die den Server verwalten, bleiben bei denen, die ihn verwalten dürfen, und wer nur Benachrichtigungen erhalten durfte, kommt endlich an seine eigenen Ziele.
- **Die Serverseite zeigt jeweils einen Bereich**, wie die Einstellungen: Überblick (System und Speicher), Jobs, Geräte, Plugins, Protokoll und Audit, jeder unter eigener Adresse. Links auf `/server#jobs` landen weiterhin richtig.
- **Das Serverprotokoll ist ein Bereich der Serverseite** (`/server/log`) statt einer eigenen Seite im Menü; `/events` leitet samt Filtern dorthin weiter.
- Eine Benachrichtigung über eine gescheiterte Aufgabe, eine gescheiterte Sicherung oder eine Verbindung, die nicht mehr antwortet, verlinkt auf den Einstellungsbereich, zu dem sie gehört, statt an den Anfang der Seite.
- Was deine Mediathek über eine importierte Wiedergabe sagen kann (welche Art von Medium es war, welche Episode, ihre Laufzeit, zu welcher Mediathek sie gehört), wird nach **jedem** Lesen der Mediathek ergänzt, nicht nur im Moment eines Imports. Verlauf wird meist importiert, bevor FinStats die Mediathek je gelesen hat, daher blieben diese Wiedergaben bisher dauerhaft geraten oder unbekannt.
- Das Wiederherstellen einer FinStats-Sicherung fügt keine Wiedergabe mehr hinzu, die von einem Tracker unter einer anderen ID kam, und behält trotzdem zwei echte Wiedergaben desselben Titels, die Minuten auseinanderliegen.
- Die Details einer Wiedergabe nennen den Tracker, der sie gebracht hat, statt alles Importierte eine Jellystat-Wiedergabe zu nennen.

### Performance
- **Ein Neustart geht bei jedem Verlauf fast sofort.** Jeder Start ermittelte über den ganzen Verlauf, wer gemeinsam geschaut hat, und entschied neu, welche Wiedergaben lokal waren, beides, ohne etwas zu ändern: 24 s und 5,5 s bei zehn Millionen Wiedergaben. Ein Start erledigt jetzt nur, was der letzte Lauf übrig gelassen hat, und alles erst wieder nach einem Update oder wenn sich die zugehörige Einstellung geändert hat.
- **Viele Wiedergaben, die auf einmal beginnen, werden in Sekunden erfasst.** Jede neue Wiedergabe, jedes Speichern des Fortschritts und jedes Gerät wurde einzeln geschrieben, und jede beendete Wiedergabe wurde mit einem eigenen Blick auf den ganzen Verlauf des Titels abgeschlossen; ein Durchlauf schreibt jetzt jede Art auf einmal. 100.000 gleichzeitig startende Wiedergaben brauchen 3 s, wo 25.000 gleichzeitig endende früher über vier Minuten brauchten.
- **„Läuft gerade“ bleibt auch bei Tausenden Zuschauern schnell.** Um herauszufinden, wer gemeinsam schaut, wurde jeder Stream mit jedem anderen verglichen und jeder Begleiter namentlich aufgeführt: vierzig Sekunden bei 10.000 Streams und darüber hinaus Gigabytes an Speicher. Streams werden jetzt innerhalb eines Titels verglichen, und jeder zeigt ein paar Namen und wie viele weitere.
- Die Ermittlung, wer gemeinsam geschaut hat, schrieb jede gruppierte Wiedergabe des Verlaufs neu, sobald FinStats startete, importierte oder eine Wiedergabe enden sah; jetzt schreibt sie nur, was sich geändert hat. Das Finden beendeter Wiedergaben wird nicht mehr mit dem Quadrat der laufenden langsamer.
- **Das Erkennen einer Wiedergabe, die der Verlauf schon hat, läuft über einen Index**, sodass ein Import von 150.000 Wiedergaben Sekunden dauert und die Neuzuordnung nach dem Lesen einer Mediathek Titel für Titel vorgeht, statt jede Wiedergabe zu lesen, während sie die Datenbank blockiert.
- **Wiedergabe-Auswertungen und die Übersicht sind über den gesamten Zeitraum schneller**: Ihre unabhängigen Teile werden nebeneinander berechnet, das Diagramm der gleichzeitigen Streams ermittelt den lokalen Tag einmal pro Viertelstunde statt einmal pro Wiedergabe, und Genres werden pro Titel gezählt. Bei einer Million Wiedergaben: 4,1 s auf 2,4 s und 1,6 s auf 1,1 s.

### Stability
- **Eine Mediathek mit Fernsehserien gilt nicht mehr als ausgeweidet.** Das Lesen einer Mediathek macht zwei Durchgänge (jedes Element, dann ein kleinerer für Besetzung und Crew, nur von Filmen und Serien), und die Prüfung, ob einem Lesevorgang zugetraut werden kann, das zu entfernen, was er nicht gesehen hat, sah auf den zweiten. In einer Mediathek mit Serien oder Musik liegen die beiden Zahlen weit auseinander, also lehnte die Prüfung jedes Lesen ab und hielt FinStats an, und die Installation startete neu und tat dasselbe wieder. Nichts wurde je fälschlich entfernt, aber kein Lesen der Mediathek konnte fertig werden. Falls deine Serien- oder Musik-Mediathek keine neuen Titel mehr aufgenommen hat, liegt es daran.
- Ein Import konnte mit „database is locked“ scheitern, wenn ein anderer Schreibvorgang (eine Wiedergabe, die aktualisiert wurde) zwischen seinem ersten Lesen und seinem ersten Schreiben landete. Beide Importe holen sich die Schreibsperre gleich zu Beginn, und das Datenbankjournal bleibt nicht mehr so groß wie der größte Import, der je lief.
- **Eine beschädigte, abgeschnittene, fremde oder schreibgeschützte Datenbankdatei wird sofort abgelehnt**, mit dem Namen der Datei und dem Hinweis, dass nichts geändert wurde, statt nach 30 stillen Sekunden; in die Datenbank eines anderen Programms wird nie geschrieben.
- Eine Sicherung oder ein Schnappschuss vor einem Update, der mittendrin unterbrochen wird, hinterlässt keine Datei mehr, die unter den drei aufbewahrten den Platz eines guten Schnappschusses einnehmen konnte.
- **Das Überschreiben der Ortsdatenbank, während FinStats läuft, bringt es nicht mehr zum Absturz**: FinStats liest aus einer privaten Kopie der Datei.
- Eine Jellyfin-App, die jedes Mal eine neue Geräte-ID meldete, konnte den Speicherverbrauch von FinStats wachsen lassen, solange es lief; Geräte, die fünf Minuten lang nicht gesehen wurden, werden jetzt vergessen.
- Eine Person behält höchstens 30 angemeldete Sitzungen; die älteste wird bei der nächsten Anmeldung abgemeldet.

### Fixed
- **Derselbe Abend wird nicht mehr doppelt gezählt, wenn ein Element umbenannt wurde.** Das Neuzuordnen des Verlaufs zu einem in Jellyfin neu hinzugefügten Titel konnte eine von einem Tracker importierte Wiedergabe zum Duplikat einer schon vorhandenen machen, weil die Prüfung, die das hätte abfangen sollen, lief, bevor die ID wechselte. Jetzt läuft sie überall erneut, wo IDs neu zugeordnet werden. Entfernt werden immer nur importierte Wiedergaben, nie eine, die FinStats selbst aufgezeichnet hat, und wie viele es waren, steht im Protokoll. Der erste Start von 2.0 durchsucht den ganzen Verlauf einmal nach den schon vorhandenen.
- **Eine Sitzung folgt der Person so, wie Jellyfin sie jetzt führt.** Ein Administrator, den Jellyfin herabgestuft hatte, behielt für den Rest seiner 30-tägigen Sitzung vollen Zugriff auf FinStats, und wer in Jellyfin deaktiviert oder gelöscht wurde, blieb angemeldet. Jetzt geht der Zugriff verloren, sobald FinStats die Benutzer von Jellyfin liest (alle 15 Minuten oder sofort aus den Einstellungen), wie es bei API-Schlüsseln schon immer war.
- Eine Anmeldung schreibt, was Jellyfin über die Person sagt, in die Benutzerliste von FinStats, sodass eine importierte Tracker-Sicherung niemanden mehr zum Administrator machen kann, bevor das erste Lesen der Benutzer es korrigiert.
- Wer in Jellyfin gelöscht wurde, wird als entfernt markiert, auch wenn zwei Lesevorgänge der Benutzer in dieselbe Sekunde fallen.
- **Download-Fehler zeigen nie das Passwort eines Download-Clients oder den API-Schlüssel eines Indexers**, die Sonarr und Radarr darin zitieren können.
- Wiedergaben, die beim Lesen einer Mediathek einem umbenannten Titel neu zugeordnet wurden, wurden erst beim nächsten Neustart auf gemeinsames Schauen geprüft; jetzt geschieht das sofort.

## [1.6.5] - 2026-09-24

### Added
- **Vor jedem Upgrade wird automatisch eine vollständige Sicherung angelegt.** Wenn ein neueres FinStats deine Datenbank zum ersten Mal öffnet, schreibt es eine komplette Kopie davon (dein ganzer Verlauf, samt Einstellungen und allem) nach `data/pre-update-backups/`, *bevor* es irgendetwas ändert. Sollte ein Upgrade je etwas kaputt machen, sind deine Daten sicher: FinStats stoppen, die Kopie an die Stelle von `finstats.db` legen, die Version starten, die du vorher hattest, und den Fehler melden. Nichts geht verloren. Die neuesten paar Kopien werden aufbewahrt; setze `FINSTATS_SKIP_PREUPDATE_BACKUP=1`, um das abzuschalten, wenn der Speicherplatz knapp ist.

## [1.6.4] - 2026-09-24

### Added
- **Fünf weitere Ziele für Benachrichtigungen**: Telegram, Slack, Pushover, Pushbullet und E-Mail, neben den schon vorhandenen Zielen Webhook, Discord, ntfy und Gotify. Jedes fragt nur nach dem, was es braucht (eine Chat-ID, einen Benutzerschlüssel, ein Postfach), und die drei, die bei einem bestimmten Dienst zu Hause sind (Telegram, Pushover, Pushbullet), lassen sich nur dort erreichen, sodass ein Token nie an einen nachgemachten Host gehen kann. Mail wird entweder vom ersten Byte an über TLS oder per `STARTTLS` verschickt; eine dritte Möglichkeit gibt es nicht.
- **Lizenzen**, unter **Einstellungen**: die eigene GPL von FinStats und jede Drittanbieter-Lizenz, auf der es aufbaut (253 Rust-Crates, die zwei mitgelieferten Schriften, die Kartenumrisse und die Geolokalisierungsdatenbank), jeder Text vollständig, so wie sein Projekt ihn geschrieben hat. Wähle eine Komponente, und ihre Lizenz öffnet sich im Fenster neben der Liste; ein Crate mit mehreren Dateien wechselt an Ort und Stelle zwischen ihnen. Die Liste wird aus den Crates erzeugt, aus denen FinStats tatsächlich gebaut wird, sodass keine Abhängigkeit hinzukommen kann, ohne dass ihre Lizenz erfasst wird.

## [1.6.3] - 2026-09-24

### Fixed
- Die Schutzprüfung greift jetzt auch, wenn Jellyfin *gar keine* Mediatheken oder *gar keine* Benutzer auflistet (ein kaputtes Lesen, nie ein normaler Tag), was die Feinabstimmung in 1.6.2 durchgelassen hatte. Einige Mediatheken zu verlieren gilt weiterhin als normal; alle auf einmal zu verlieren nicht.

## [1.6.2] - 2026-09-24

### Fixed
- Die in 1.6.1 eingeführte Schutzprüfung für die Datenintegrität war zu eifrig: Eine Mediathek, die tatsächlich die meisten ihrer Elemente verlor (etwa eine Handvoll Clips, deren Dateien weg waren), wurde für ein kaputtes Lesen gehalten, sodass FinStats sich weigerte, sie zu aktualisieren, und anhielt. Jetzt greift sie nur bei einem *eindeutig* kaputten Lesen: einer Mediathek, die groß genug ist, um zu zählen, und komplett leer zurückkommt, oder einer großen, die bis auf fast nichts ausgeweidet ist. Alltägliche Änderungen werden wie bisher übernommen, und der Fall „Jellyfin hat nichts geliefert“ wird weiterhin erkannt.

## [1.6.1] - 2026-09-24

### Added
- **FinStats traut keinem Lesen von Jellyfin mehr, das deinen Verlauf auslöschen würde.** Kommt eine Mediathek, für die FinStats Tausende Elemente führt, plötzlich leer oder fast leer zurück (ein Jellyfin-Upgrade, das seine API geändert hat, ein Fehler, der als leeres Ergebnis daherkommt), behält FinStats seine Daten, markiert den Abgleich als fehlgeschlagen (und benachrichtigt) und hält sauber mit einer klaren Meldung an, statt alles als entfernt zu markieren. Starte neu, sobald Jellyfin wieder normal läuft, oder setze `FINSTATS_ALLOW_LIBRARY_SHRINK=1`, wenn du eine Mediathek wirklich geleert hast. Dieselbe Schutzprüfung gilt für die Liste der Mediatheken und die Benutzerliste.

### Security
- Ein von einem Client gesendeter `X-Forwarded-Host`-Header konnte für einen fremden `Origin` bürgen und einen seitenübergreifenden Schreibzugriff an der Same-Origin-Prüfung vorbeibringen. Dieser Header gilt jetzt nur noch hinter einem vertrauenswürdigen Proxy (`FINSTATS_TRUST_PROXY`), nach derselben Regel, die die Ermittlung der Client-Adresse schon nutzt. Das Sitzungscookie mit `SameSite=Lax` war und bleibt die erste Verteidigungslinie.
- Ein persönliches Benachrichtigungsziel konnte auf `0.0.0.0` oder `::` zeigen, die diesen Rechner erreichen. Die Prüfung, die persönliche Ziele auf öffentliche Adressen beschränkt, zählt diese jetzt als lokal, neben Loopback und den privaten Bereichen.

### Fixed
- Die Seite **Wiedergabe** lädt jetzt auch für eine Mediathek, die eine Wiedergabe ohne erfasste Methode enthält (eine Zeile aus einer Sicherung, die älter als die Spalte ist); eine solche Wiedergabe zählt einfach als direkte Wiedergabe.
- Ein sehr langes Wort in einer Suche oder einem Filter ist kein Serverfehler mehr: SQLite lehnt ein `LIKE`-Muster über 50.000 Zeichen ab, daher werden Filterwörter vor der Abfrage auf eine vernünftige Länge gekürzt.
- Die Seite **Einstellungen** scrollt auf dem Handy nicht mehr seitwärts, wenn eine Karte einen ganzen Satz zurückmeldet, etwa das Ergebnis einer Wiederherstellung.

## [1.6.0] - 2026-09-24

Benachrichtigungen

### Added
- Karte **Benachrichtigungen** in den Einstellungen: Ziel hinzufügen, ankreuzen, worüber es informiert werden soll, auf Test drücken. Ohne Ziel wird keinerlei Anfrage gestellt.
- Vier Arten von Zielen: ein einfacher **Webhook** (JSON, dokumentiert in `docs/api.md`, optionaler `Authorization`-Header), **Discord** (ein Embed über die Webhook-URL eines Kanals), **ntfy** (ntfy.sh oder ein eigener Server, an ein Topic) und **Gotify** (dein eigener Server, mit einem Anwendungstoken). Titel stehen in der Nachricht statt in einem Header.
- Elf Arten von Ereignissen, jede zum eigenen Ankreuzen: unmögliche Reise, ein Konto in einem neuen Land, eine Häufung fehlgeschlagener Anmeldungen, eine fehlgeschlagene Aufgabe, eine Sicherung, die nicht geschrieben werden konnte, etwas, das nicht mehr antwortet (Jellyfin, Sonarr, Radarr oder Seerr), und wenn es wieder antwortet, neue Titel in der Mediathek (pro Serie und Tag zusammengefasst, wie im Regal auf der Übersicht), eine Anfrage, die schaubar wurde, und Beginn oder Ende einer Wiedergabe. Die letzten beiden sind standardmäßig aus.
- Eine Anfrage, die schaubar wurde, nennt den Titel, wer sie gestellt hat und wie lange gewartet wurde.
- Persönliche Ziele: Wer **Benachrichtigungen erhalten** bekommt, kann ein eigenes Ziel hinzufügen, an das nur geht, was die Person ohnehin sehen darf: ihre eigenen Anfragen und Warnungen immer, die anderer mit *Aktivität aller sehen*, Adressen mit *Netzwerkdetails sehen*. Ein persönliches Ziel muss auf eine öffentliche Adresse zeigen; nur ein Administrator darf eines ins eigene Netzwerk richten.
- **IP-Adressen und Orte einschließen**, pro Ziel und standardmäßig aus: Eine Nachricht über eine unmögliche Reise nennt sonst nur Orte („Oslo, Norwegen und London, Vereinigtes Königreich“).
- **Kürzlich gesendet**, in derselben Karte: was gesagt wurde, an welches Ziel und was die Gegenseite antwortete, wenn es nicht ankam. Eine gescheiterte Nachricht wird über etwa eine Stunde viermal erneut versucht, und auf ein Ziel, das Anfragen drosselt, wird gewartet.
- **Die Adresse von FinStats**, eine Einstellung für Administratoren, ist das, woraus der Link einer Benachrichtigung gebaut wird. Leer heißt, Nachrichten tragen keinen Link.
- Die eigenen geplanten Aufgaben von Jellyfin auf der Serverseite: was jede tut, in Worten statt mit dem Namen ihres Codes, und ob sie zu den schweren gehört. Eine laufende Aufgabe zeigt einen Fortschrittsbalken, einen Prozentwert und wie viel noch fehlt; die übrigen zeigen, wann sie als Nächstes laufen, wann sie zuletzt liefen und ob das gut ging. Aufgaben, die Jellyfin in seinem eigenen Dashboard verbirgt, sind dabei, und nichts in FinStats kann eine Aufgabe starten oder stoppen.
- Wie viel von einer Aufgabe noch fehlt, wird an der Geschwindigkeit gemessen, mit der sich ihr Prozentwert tatsächlich bewegt hat, gegen die jüngste Messung, die weit genug zurückliegt, um etwas auszusagen, statt am Durchschnitt des ganzen Laufs, sodass eine Aufgabe, die schneller oder langsamer wird, nach ihrem aktuellen Tempo beschrieben wird.
- Die Aufgabenlisten, die FinStats ohnehin aus anderen Gründen liest (die Prüfung auf Mediathek-Scans alle 5 Minuten, die Serverdetails alle 15), fließen in dieselbe Messung ein, sodass eine Aufgabe, die seit einer Stunde läuft, meist schon beim Öffnen der Seite eine Schätzung hat. Dafür wird Jellyfin nichts zusätzlich gefragt.
- Eine laufende Aufgabe ohne bisherige Messung zeigt **ETA** und wechselnde Auslassungspunkte statt einer Zahl, und eine, deren Prozentwert sich eine halbe Minute nicht bewegt hat, zeigt daneben „68 % seit 4 min“. Nichts tritt an die Stelle einer Schätzung, die nicht gemessen wurde.
- Eine Liste aller Versionen neben den Versionshinweisen: Ein Klick führt zu jeder Version, sie markiert die gerade gelesene, und auf einem schmalen Bildschirm wird sie zu einer Zeile von Reihen über den Hinweisen.

### Changed
- Die Versionshinweise klappen nicht mehr zu. Jede Reihe steht offen, und die Versionsliste ist der Weg durch sie, statt dass immer nur eine Gruppe offen ist und jede andere einen Klick braucht.
- **Einstellungen → Ausgehende Verbindungen** listet jedes Benachrichtigungsziel neben Jellyfin und den Dienstverbindungen: was es ist, sein Host, ob es eingeschaltet ist und wann es zuletzt eine Nachricht angenommen hat.
- An der Erfassung, am Verlauf und an jeder Zahl auf dem Bildschirm ändert sich nichts. FinStats *liest* weiterhin nur aus Jellyfin, Sonarr, Radarr und Seerr.

## [1.5.2] - 2026-09-22

### Changed
- Das Lesen der Sitzungsliste von Jellyfin und das Zurücksetzen der Uhr, die danach verlangt hat, sind ein einziger Aufruf, sodass beides nicht mehr auseinanderfallen kann. Genau diese Form hatte der Fehler mit ausufernden Anfragen, der beim Bau von 1.5.0 auftauchte.
- Das nachzuweisen dauert jetzt einen Bruchteil einer Sekunde statt sieben Minuten gegen einen echten Server, deshalb wird es bei jeder Änderung geprüft. Dieselben Anfragen, derselbe Verlauf, dieselben Zahlen auf dem Bildschirm.

## [1.5.1] - 2026-09-22

### Fixed
- Eine Transkodierung wird einmal erfasst, nicht einmal pro Sekunde. Wenn eine Wiedergabe mit Transkodierung beginnt und dann zur direkten Wiedergabe zurückkehrt, lässt Jellyfin die Transkodierungsdetails an der Sitzung stehen; jede Messung wurde mit einem Wert verglichen, den FinStats gerade überschrieben hatte, sodass jede Messung wie eine Änderung aussah und die Seite Aktivität sich mit Zeilen „Transkodierung“ füllte, unter denen „Direkte Wiedergabe: …“ stand.
- Bereits geschriebene Zeilen werden beim Start aufgeräumt: Wo eine Folge von ihnen dasselbe sagt, bleibt die erste. Eine Wiedergabe, die transkodiert werden musste, zählt weiterhin als Transkodierung.

## [1.5.0] - 2026-09-22

Sitzungserfassung per WebSocket

### Changed
- FinStats hält vom Start an eine Verbindung zu Jellyfin offen, und darüber erfährt es von einer Wiedergabe. 1.4.0 lieferte das als Einstellung aus, standardmäßig aus, damit es sich an echten Servern bewähren konnte; jetzt ist es immer an. Dieselben Wiedergaben, Pausen und Sprünge, ermittelt vom selben Code.
- Nichts läuft: FinStats fragt Jellyfin überhaupt nichts. Ein Abend, an dem niemand schaut, kostet keine Anfragen.
- Etwas läuft: Die Sitzungsliste wird einmal pro Sekunde gelesen. Das macht Pausen, Sprünge und Spurwechsel sekundengenau, und so endet eine Wiedergabe, deren Client ohne Abschied verschwunden ist.
- Alles Geladene ist drei Messungen hintereinander pausiert: Das Fragen hört auf, und die Verbindung übernimmt, wo vorher ein pausierter Film jede Sekunde dieselbe Frage kostete, rund 16 MB pro Stunde. Ein Fortsetzen oder jemand anderes, der etwas startet, wird innerhalb etwa einer Sekunde bemerkt, und pausierte Zeit zählt weiterhin nie als Wiedergabezeit.
- Jedes ausgehende Lesen fragt nach komprimierten Antworten und entpackt sie (Jellyfin, Sonarr, Radarr und Seerr komprimieren alle, wenn man darum bittet), was bei den beiden größten Lesevorgängen mehr als die Hälfte einspart: der Sitzungsliste während einer Wiedergabe und dem Lesen der Mediathek. Die Geolokalisierungsdatenbank wird weiterhin so heruntergeladen, wie sie gespeichert ist.
- Die Jellyfin-Karte in den Einstellungen sagt, welche der beiden Hälften läuft, und der Live-Punkt folgt dem, ob die Verbindung trägt, nicht der Hälfte, in der du gerade bist.
- Es läuft immer nur eine der beiden: Solange FinStats fragt, bittet es Jellyfin, nichts zu senden, und bittet es, wieder anzufangen, sobald die letzte Wiedergabe endet. Die Verbindung bleibt die ganze Zeit offen.
- Einem Jellyfin, das auf das Abonnement mit nichts antwortet, wird geglaubt, solange es auf Keep-alives antwortet: FinStats sagt das einmal, liest die Sitzungsliste in der Zwischenzeit selbst und bleibt abonniert, bis die erste Push-Nachricht die Sache klärt. „Kann nicht pushen“ und „startete gerade neu, als wir fragten“ sehen gleich aus, also ist Schweigen nie ein Urteil.
- Der Rückfall bleibt unverändert: Eine Verbindung, die sich schließt, verstummt oder sich als ungeeignet erweist, fällt gleich beim nächsten Durchlauf auf zeitgesteuertes Fragen zurück und verbindet sich immer wieder neu, in den Intervallen **Während jemand schaut, prüfen alle** und **Während nichts läuft, prüfen alle**.

### Added
- Was die Erfassung gerade tut, unter `GET /api/status`: in welchem der vier Zustände sie ist (`idle_socket`, `playing_poll`, `paused_socket`, `fallback`), ob die Verbindung offen und abonniert ist, wie oft sie fragt, wie viele Sitzungsabfragen in der letzten Minute hinausgingen und wann sich das zuletzt geändert hat. Aus dem Speicher beantwortet (keine Anfrage, keine Abfrage), und ohne Namen, Titel oder Anzahlen.
- Diese Felder werden bei jedem Durchlauf gegeneinander geprüft, und eine anhaltende Unstimmigkeit wird ins Protokoll geschrieben.
- Ein Begrenzer unter jeder Sitzungsabfrage im Prozess: höchstens 2 pro Sekunde und 70 pro Minute; alles darüber wird verzögert, gezählt und höchstens einmal pro Minute protokolliert. Normales Schauen bleibt weit darunter.

### Fixed
- Live-Aktualisierungen ersetzen das Abfragen jetzt wirklich. Eingeschaltet fragte 1.4.0 trotzdem alle fünf Sekunden nach der Sitzungsliste und öffnete die Verbindung alle zwanzig bis vierzig Sekunden neu, weil es die Verbindung danach beurteilte, wie kürzlich eine Liste angekommen war, und Jellyfin eine sendet, wenn sich etwas *ändert*, und dazwischen nichts. Ob die Verbindung lebt, entscheidet jetzt, ob Jellyfin überhaupt antwortet, geprüft mit einem Keep-alive etwa zweimal pro Minute.
- Eine Sitzungsliste, die ankam, während FinStats zwischen zwei Durchläufen war, wurde verworfen, in der Annahme, die nächste folge anderthalb Sekunden später. Sie wird jetzt behalten und genutzt.
- **Zuletzt geprüft** in den Einstellungen und die Karte Ausgehende Verbindungen veralten nicht mehr, während die Verbindung ruhig ist: Sie zeigen die Antwort von Jellyfin auf ein Keep-alive.

### Removed
- Die Einstellung **Jellyfin melden lassen, was läuft** unter Einstellungen → Erfassung und `live_socket` aus der Einstellungs-API.
- `socket_enabled` aus dem Erfassungsstatus: Ein „ausgeschaltet“ gibt es nicht mehr.
- `last_reconcile_at` aus dem Erfassungsstatus, ersetzt durch `socket_live`.

## [1.4.1] - 2026-09-22

### Changed
- Seerr wird nur noch vollständig gelesen, wenn sich etwas geändert hat: Alle fünf Minuten fragt FinStats nach einer Zeile, der zuletzt geänderten Anfrage, und der Durchlauf endet dort, wenn es sie schon kennt. Das ist etwa ein Kilobyte statt rund sechzig für eine Seite mit fünfzig. Anfragen, die noch unterwegs sind, werden jede Viertelstunde angesehen, alles wird einmal am Tag aufgelistet, und eine neue Anfrage erscheint weiterhin binnen fünf Minuten.
- Die Warteschlangen von Sonarr und Radarr werden alle fünf Sekunden gelesen, während eine Seite sie zeigt, jede Minute, solange etwas in der Warteschlange ist, und alle fünf Minuten, solange sie leer ist, statt rund um die Uhr jede Minute. Das Öffnen der Seite, das Verbinden eines Dienstes oder eine neue Anfrage in Seerr aktualisieren sie sofort.

## [1.4.0] - 2026-09-21

Optionale WebSocket-Sitzungen und alles, was FinStats erreichen kann

### Added
- **Jellyfin melden lassen, was läuft** unter Einstellungen → Erfassung, aus, bis du es einschaltest: Eine Verbindung trägt die Sitzungsliste, und aus rund 17.000 Anfragen am Tag werden fast keine. Wiedergaben, Pausen, Sprünge und Gruppen werden von genau dem Code ermittelt, der sie vorher ermittelt hat. Solange es aktiv ist, stellt FinStats weiterhin eine normale Anfrage pro Minute, *während etwas läuft*, und keine, wenn nichts läuft. Eine Verbindung, die sich schließt, fünfzehn Sekunden verstummt oder sich als ungeeignet erweist, fällt beim nächsten Durchlauf auf den Timer zurück und verbindet sich immer wieder neu.
- **Ausgehende Verbindungen**, eine Karte in den Einstellungen für Jellyfin-Administratoren: jedes Ziel, das FinStats erreichen kann (dein Jellyfin, der Dienst „Wie ist meine IP“, die Datenbank von DB-IP und jedes Sonarr, Radarr oder Seerr), mit seinem Zweck, ob es eingeschaltet ist und wann es zuletzt geantwortet hat. Aus dem gebaut, was FinStats schon weiß, und nur mit Adressen: nie ein Schlüssel, nie ein Basispfad.
- **Jetzt abfragen** unter Einstellungen → Heimnetzwerk, für den Tag, an dem sich deine öffentliche Adresse ändert.

### Changed
- Die Abfrage „Wie ist meine IP“ läuft einmal, wenn FinStats zum ersten Mal eine Adresse braucht, statt alle fünfzehn Minuten.
- Die Jellyfin-Karte in den Einstellungen sagt, wie die Erfassung von Wiedergaben erfährt, und, wenn die Verbindung eingeschaltet ist, aber nichts trägt, warum nicht.

## [1.3.0] - 2026-09-20

Sonarr, Radarr und Seerr auf einer Seite

### Added
- **Verbindungen** in den Einstellungen, für Jellyfin-Administratoren: **Sonarr**, **Radarr** und **Seerr** (oder Jellyseerr/Overseerr), mehrere von einer Art, jede vor dem Speichern getestet. Dein Download-Client braucht keine eigene Einrichtung, weil Sonarr und Radarr schon mit ihm sprechen und FinStats liest, was sie wissen.
- **Anfragen**: wer was angefragt hat, wie lange es bis zur Ankunft dauerte und ob es je geschaut wurde. Kacheln für wartende Titel und die typische Wartezeit, ein Diagramm, wie sich das Monat für Monat verändert hat, und Listen dessen, was vor Wochen ankam und nie abgespielt wurde, und wer am meisten anfragt.
- **Demnächst**: neue Episoden und Filmstarts aus Sonarr und Radarr als Tagesagenda, markiert danach, ob *du* die Serie schaust, und für Personen, die alle sehen dürfen, wer sonst noch. Eine Serie gilt als geschaut, wenn jemand in den letzten vier Monaten eine Episode davon abgespielt hat. Eine Zeile Demnächst auf der Übersicht, eine pro Profil für die Serien, die diese Person schaut, und auf der Seite eines Titels, was für ihn als Nächstes kommt.
- **Downloads**: die Live-Warteschlange jedes Sonarr und Radarr (was es ist, wie weit, wie schnell, was beim Import schiefging), mit der Person daneben, die es angefragt hat. Ein Staffelpaket ist eine Zeile, egal wie viele Episoden es enthält, und was Aufmerksamkeit braucht, steht oben. Darunter, was in einer Woche, einem Monat oder einem Jahr hereinkam, nach Indexer, Qualität und Download-Client, und welche Downloads fehlschlugen. Die Übersicht zeigt dieselbe Liste, gekürzt.
- **Downloads sehen**, eine neue Berechtigung. Ohne sie sehen Personen weiterhin, wie weit ihre *eigene* Anfrage ist und wie lange es noch dauert, und sonst nichts: keine Release-Namen, keine Geschwindigkeiten, keine anderen Downloads. Anfragen anderer erfordern *Aktivität aller sehen*.

### Changed
- FinStats sendet diesen Diensten `GET` und sonst nichts: Es enthält keinen Code, der eine Anfrage genehmigen, eine Suche starten oder einen Download hinzufügen, pausieren oder entfernen könnte.
- Ihre API-Schlüssel werden in der eigenen Datenbank von FinStats gespeichert, nie an den Browser zurückgeschickt, nie in ein Protokoll geschrieben und sind nie Teil einer Sicherung. Keiner Weiterleitung wird gefolgt, sodass ein Schlüssel nirgendwohin gelangen kann, wo du ihn nicht eingegeben hast, und Zertifikate werden geprüft, sofern du das nicht für eine Verbindung abschaltest.

## [1.2.2] - 2026-09-20

### Added
- Audio- und Untertitelsprachen auf jedem Film und jeder Episode („Audio: Japanisch · Englisch“).
- Wie weit eine Synchronfassung reicht: Eine Serie oder Staffel sagt, wie viele ihrer Episoden jede Sprache haben („Englisch: 13 von 26 Episoden“), und die Episodenliste hat eine Spalte **Audio**.
- **Audiosprachen** und **Untertitelsprachen** auf Mediathekseiten: wie viele Dateien sich in jeder Sprache abspielen lassen.
- FinStats liest die Mediathek nach diesem Update einmal neu, um die Sprachen zu erfassen. Es sagt nie „synchronisiert“, weil Jellyfin die Originalsprache eines Titels nicht liefert, und eine Spur ohne Sprachkennzeichnung wird als „Unbekannt“ aufgeführt.

## [1.2.1] - 2026-09-20

### Changed
- Die Bilder in der README sind für diese Version neu aufgenommen, mit einem neuen der Seite **Sicherheit**. In der App ändert sich nichts.

## [1.2.0] - 2026-09-20

Von wo aus geschaut wird, auf einer Karte

### Added
- **Seite Sicherheit**, für Personen, die Netzwerkdetails und die Aktivität aller sehen dürfen: eine Weltkarte mit einem Punkt für jeden Ort, von dem aus deine Benutzer schauen, so groß wie das, was dort passiert. Zu Hause ist grün, Orte unterwegs sind lila, Live-Streams pulsieren, und fehlgeschlagene Anmeldungen von außen sind rot. Ziehen zum Verschieben, Zoomen mit den Schaltflächen, den Tasten oder Strg + Mausrad, und überlappende Punkte verschmelzen, bis du hineinzoomst. Darunter jeder Ort mit seinen Personen, Wiedergaben, Wiedergabezeit und Anmeldungen, und die Länder nach Wiedergaben.
- **Unmögliche Reise**: eine Warnung, wenn ein Konto an zwei Orten gesehen wird, die kein Flug verbindet, oder an zwei weit entfernten Orten gleichzeitig, mit beiden Sichtungen, der Entfernung, der Zeit dazwischen und der Geschwindigkeit, die das erfordert hätte. **Auf Karte zeigen** zeichnet die Reise.
- **Neues Land**: das erste Mal, dass jemand aus einem Land abspielt oder sich anmeldet, in dem er noch nicht gesehen wurde.
- Eine Warnung mit einer Notiz erledigen oder ein Ortspaar für diese Person stummschalten, für ein VPN oder ein Handy, dessen Mobilfunkanbieter in der Hauptstadt sitzt. Warnungen, die in altem Verlauf gefunden werden, nach einem Import oder in einer frischen Datenbank, werden als erledigt abgelegt, statt die Liste zu fluten. Warnungen und was du zu ihnen entschieden hast, sind Teil von Sicherungen.
- Orte stammen aus einer Städtedatenbank, die lokal gelesen wird: Keine Adresse wird je irgendwohin gesendet, und die Karte wird aus mit FinStats mitgelieferten Umrissen gezeichnet statt von einem Kartendienst. Lade die freie Datenbank von DB-IP über die Seite Sicherheit oder **Einstellungen → Sicherheit** herunter, lass FinStats sie monatlich erneuern (aus, bis du es einschaltest), oder leg deine eigene `.mmdb` (DB-IP, MaxMind GeoLite2-City) in `data/geoip/`. `FINSTATS_GEOIP_DB` nennt eine Datei an anderer Stelle.
- **Einstellungen → Sicherheit**: die verwendete Datenbank, die monatliche Aktualisierung und wie schnell (900 km/h) und wie weit voneinander entfernt (500 km) zwei Sichtungen sein müssen, um als unmögliche Reise zu zählen. Städtedatenbanken liegen oft ein paar hundert Kilometer daneben, dafür ist die Entfernung da.

## [1.1.3] - 2026-09-20

### Added
- Strg+Leertaste durchsucht auch Besetzung und Crew von allem in deiner Mediathek, unter **Besetzung und Crew**, mit Foto, ob die Person spielt oder Regie führt und in wie vielen deiner Titel sie vorkommt; ein Klick öffnet ihre Seite. So nachsichtig wie die Titelsuche (Wörter in beliebiger Reihenfolge, Akzente egal, ein Vertipper erlaubt), und wenn mehrere gleich gut passen, kommt zuerst, wer in mehr deiner Titel vorkommt.

## [1.1.2] - 2026-09-20

### Added
- **Kürzlich hinzugefügt** auf der Übersicht, über dem Aktivitätsdiagramm: die 30 neuesten Zugänge als seitlich scrollbare Posterreihe, mit Pfeilen am Computer. Jeder sagt, wann er ankam und was er ist (ein Film, ein Album oder „Staffel 4 · 3 Episoden“), und eine einzelne neue Episode sagt, welche, mit dem eigenen Poster der Staffel, wenn es eines gibt.
- **Umschalt + Mausrad** und die Pfeiltasten bewegen um ein Poster, Bild auf und Bild ab um eine Bildschirmbreite, Pos1 und Ende gehen an das jeweilige Ende, und Wischen funktioniert. Das normale Mausrad scrollt weiterhin die Seite.
- Neue Episoden derselben Serie, die am selben Tag ankommen, teilen sich einen Eintrag, und eine ganze auf einmal hinzugefügte Serie ist ein Eintrag („3 Staffeln · 60 Episoden“), sodass ein großer Import nicht alles andere aus der Liste drängt.
- Die Reihe ignoriert Zeitraum und Person der Übersicht, da es um die Mediathek geht und nicht um Wiedergaben, und blendet sich aus, solange die Mediathek leer ist.

## [1.1.1] - 2026-09-20

### Added
- Seiten öffnen sofort: Einen Moment, nachdem FinStats geladen ist, holt es still, was du wahrscheinlich als Nächstes öffnest (die Übersicht, dein Profil und deine Zeitleiste, Aktivität, Benutzer, Mediatheken und jede Mediathek, Wiedergabe, die Serverseiten, den Rückblick und die aktivsten Personen), sodass eine davon sofort erscheint und sich trotzdem im Hintergrund aktualisiert.
- Filme, Serien und Mitwirkende werden nur bei erkennbarem Interesse geholt, da es Tausende davon gibt: Den Zeiger auf einem Link ruhen lassen, ihn antippen oder mit der Tastatur erreichen holt genau diese eine Seite.
- Es wartet, bis die geöffnete Seite fertig geladen ist, arbeitet eine Seite nach der anderen ab, während der Browser untätig ist, pausiert in einem Hintergrund-Tab, tut nichts im Datensparmodus oder bei sehr langsamer Verbindung und wird beim Abmelden vergessen.
- Ein kleiner Link **Repo** mit dem GitHub-Logo in der Statusleiste, neben der Version.

### Fixed
- Die **Zeitleiste** vergaß, welche Mediatheken ausgeschaltet waren, wenn die Seite neu geladen oder aus einem Lesezeichen geöffnet wurde, obwohl die Wahl in der Adresse stand.

## [1.1.0] - 2026-09-20

Eine Zeitleiste von allem, was du geschaut hast

### Added
- **Zeitleiste**, neben **Überblick** in jedem Profil: dein Schauen als ein Pfad von heute zurück bis zur ersten Wiedergabe, die FinStats kennt, mit einem Abend voller Episoden zu einem einzigen Halt zusammengefasst. Jeder Halt ist ein Poster mit dem, was geschaut wurde und wann („Staffel 2 · Episoden 3-6“, ein Film und ob er mehr als eine Sitzung brauchte, oder ein Album und wie viele Tracks), und öffnet den Titel.
- Der Pfad richtet sich nach der Fensterbreite: drei Halte pro Reihe auf einem breiten Bildschirm, zwei auf einem schmaleren, jede zweite Reihe rückwärts mit einer Kurve, die sie mit der nächsten verbindet, bis hin zu einer einzigen geraden Linie auf dem Handy. Der erste Halt jedes Monats trägt den Monat.
- Ältere Halte laden beim Scrollen von selbst, ganz zurück bis zum Beginn des Verlaufs.
- Schalter für **Mediatheken** über dem Pfad lassen weg, was du nicht sehen willst, etwa Musik, und die Wahl ist Teil der Adresse.
- Was noch läuft, ist mit **Läuft gerade** markiert.
- Du siehst deine eigene Zeitleiste; die anderer erfordern die Berechtigung „alle sehen“.

## [1.0.5] - 2026-09-20

### Added
- Schutz vor Downgrades: Die Datenbank merkt sich die neueste FinStats-Version, die sie geöffnet hat, und eine ältere Version weigert sich, darauf zu starten, statt still mit Daten zu arbeiten, die sie nicht vollständig versteht. Die Meldung sagt, was zu tun ist. Versionen bis 1.0.4 erschienen, bevor es diese Prüfung gab.

## [1.0.4] - 2026-09-20

### Fixed
- In den Details einer Wiedergabe war die Kopierschaltfläche neben einer langen **Geräte-ID** in eine eigene Zeile gerutscht.
- Auf der Seite Wiedergabe war die letzte Spalte von **Welche Clients transkodieren** abgeschnitten („Transk…“); Überschriften aus zwei Wörtern brechen jetzt um.

### Changed
- Die Screenshots in der README sind wieder aktuell; sie zeigten noch Version 0.5.0.

## [1.0.3] - 2026-09-20

### Added
- Eine Episodenseite zeigt das eigene Bild der Episode, das Jellyfin in seiner Episodenliste zeigt, statt des Posters der Serie. Eine Episode, für die Jellyfin kein Bild hat, behält das Poster.

## [1.0.2] - 2026-09-20

### Fixed
- In den **Versionshinweisen** hatten die Gruppen v0.9 und v0.10 keine Schlagzeile neben ihrer Version. Die Schlagzeile einer Gruppe ist jetzt immer der erste Satz dessen, worum es in ihrer ersten Version ging, sodass eine lange Einleitung nicht mitten im Satz abgeschnitten wird.

## [1.0.1] - 2026-09-20

### Fixed
- Das veröffentlichte Image startete auf einem frischen Rechner nicht: „unable to open database file: /data/finstats.db“, nachdem es eine halbe Minute gehangen hatte. Docker legt einen fehlenden Ordner `data` als root an, und FinStats läuft als normaler Benutzer. Der Container macht den Ordner jetzt zu seinem eigenen und wechselt dann zu diesem Benutzer, sodass `docker run` beim ersten Versuch funktioniert. Für Dateien, die jemand anderem als Benutzer 1000 gehören, setze `PUID` und `PGID`; `--user` funktioniert wie bisher.
- Ein Datenordner, der wirklich nicht beschreibbar ist, wird sofort gemeldet, mit dem Befehl, der das behebt, statt 30 Sekunden später als Datenbankfehler.

## [1.0.0] - 2026-09-20

Bereit für alle

### Added
- Ein fertiges Image, jetzt unter `ghcr.io/finstats/finstats`, für 64-Bit-Rechner mit Intel/AMD und ARM (ein Raspberry Pi 4 oder 5 funktioniert): `:latest` ist die neueste Version, ein Versions-Tag bleibt, wo es ist, und `:edge` ist die Entwicklungsversion.
- Jede Version erscheint mit diesen Versionshinweisen auf der Releases-Seite des Projekts auf GitHub.

### Changed
- Versionsnummern bedeuten jetzt, was sie sagen: Updates innerhalb von 1.x machen weder deine Daten noch deine Sicherungen oder Einstellungen kaputt.
- Die Docker-Compose-Datei und alle Anleitungen nutzen das veröffentlichte Image. Bei einem selbst gebauten `finstats:latest` muss nur der Image-Name geändert werden; der Ordner `data` wird unverändert übernommen.

## [0.10.2] - 2026-09-20

### Fixed
- Auf dem Handy konnten die Einstellungen und die Seite einer Serie seitwärts scrollen: Eine Beschriftung nur für Screenreader entkam ihrer Tabelle, und die Episodenlisten und die Besetzungsreihe taten dasselbe. Sie scrollen jetzt in sich selbst.
- Der Rückblick scrollt nicht mehr seitwärts, wenn das Hintergrundwort eines Kapitels breiter als die Seite ist.
- Eine lange Zeile „schaut auf …“ unter **Läuft gerade** endet mit Auslassungspunkten, statt mitten im Buchstaben abgeschnitten zu werden.
- Das Poster vor jeder Serie in einem Profil war für Screenreader ein zweiter, namenloser Link; es ist jetzt vor ihnen verborgen, und der Titel daneben ist der Link.

## [0.10.1] - 2026-09-20

### Fixed
- Das Herunterladen einer Sicherung schlug im Browser fehl („Die Quelldatei konnte nicht gelesen werden“ in Firefox), ohne dass die Seite sagte, warum: FinStats komprimierte eine Sicherung, die schon komprimiert ist, ein zweites Mal, und die doppelt gepackte Übertragung brach vor dem Ende ab. Sicherungen werden so gesendet, wie sie sind, mit ihrer Größe vorab, sodass der Browser einen echten Fortschritt zeigen kann.

## [0.10.0] - 2026-09-20

Sicherungen und der Umzug auf eine neue Installation

### Added
- Eine Sicherung pro Woche, die neuesten fünf werden behalten: jede Wiedergabe mit ihrer Zeitleiste aus Pausen und Sprüngen, Gesehen-Markierungen, Berechtigungen, Heimadressen, das Serverprotokoll und deine Einstellungen, in einer kleinen Datei, ein paar hundert KB für Tausende Wiedergaben. **Einstellungen → Sicherungen** listet sie mit je einer Schaltfläche **Herunterladen**, erstellt auf Wunsch eine und legt fest, wie oft und wie viele. Sie liegen im Ordner `backups` deines Datenverzeichnisses. Eine Sicherung enthält keinen Jellyfin-API-Schlüssel und keine Anmeldesitzung, aber sie ist ein vollständiger Wiedergabeverlauf mit IP-Adressen, also bewahre sie an einem privaten Ort auf.
- Wiederherstellen führt zusammen, statt zu überschreiben: Schon vorhandene Wiedergaben werden übersprungen, also ist es sicher, das zweimal zu tun oder in eine Instanz hinein, die schon eine Weile läuft. Entferne den Haken bei „Auch Einstellungen und Berechtigungen wiederherstellen“, um nur den Verlauf zu holen. Sicherungen funktionieren versionsübergreifend in beide Richtungen. Im Terminal: `finstats backup` und `finstats restore <file>`.

### Changed
- Die Erfassung prüft jede Sekunde, während jemand schaut, und alle 5 Sekunden, während niemand schaut, wo sie vorher durchgehend alle 5 Sekunden prüfte, sodass Pausen, Sprünge und Spurwechsel sekundengenau erfasst werden. Beides steht unter **Einstellungen → Erfassung**; ein geändertes altes Intervall muss neu gesetzt werden.

### Fixed
- Eine IP-Adresse in den Details einer Wiedergabe brach mitten in der Adresse auf zwei Zeilen um. Die Markierung Lokal/Extern rückt darunter, wenn kein Platz ist.

## [0.9.1] - 2026-09-20

### Fixed
- Personen zu Hause wurden als extern angezeigt. Ein Gerät im eigenen Netzwerk, das Jellyfin über dessen öffentlichen Namen erreicht, etwa über einen Reverse-Proxy oder eine Domain, kommt mit der öffentlichen IP des Haushalts an, und nur private Adressen zählten als lokal. FinStats lernt jetzt die öffentliche Adresse und zählt diese Wiedergaben für den ganzen Verlauf als lokal, und es merkt sich frühere Adressen.

### Added
- **Einstellungen → Heimnetzwerk**: die Adressen, die FinStats als Zuhause behandelt, eigene hinzufügen (eine frühere Adresse, ein zweites Zuhause, ein VPN-Ausgang) oder die Abfrage abschalten. Die Abfrage fragt alle 15 Minuten einen einfachen Dienst „Wie ist meine IP“ und probiert mehrere (Amazon, Cloudflare und andere), weil werbeblockierende DNS-Server wie Pi-hole sie oft sperren, und einer davon braucht gar kein DNS. Die Anfrage enthält nichts über dich oder deinen Server, und mit abgeschaltetem Schalter spricht FinStats mit nichts außer Jellyfin. `FINSTATS_PUBLIC_IP_URL` legt einen eigenen Dienst fest.

## [0.9.0] - 2026-09-20

Jede Tabelle in der Reihenfolge, die du willst

### Added
- Jede Tabelle sortiert: Ein Klick auf eine Überschrift sortiert danach, ein zweiter kehrt die Reihenfolge um, ein dritter stellt die ursprüngliche wieder her. Die Bedeutung einer Zelle wird verstanden, sodass „3 T. 2 h“ als Zeit und „1,4 GB“ als Größe sortiert wird, und leere Zellen kommen zuletzt. Das gilt für die Benutzerliste, Zuschauer, Episoden, Geräte, Plugins, geplante Aufgaben, die Tabellenansicht jedes Diagramms, die Berechtigungsmatrix und die Balkenlisten (Codecs, Auflösungen, Clients und der Rest), die eine eigene schlanke Kopfzeile bekommen.
- Aktivität und das Serverprotokoll sortieren über alle ihre Seiten statt nur über die Zeilen auf dem Bildschirm, und die Reihenfolge steht in der Adresse, sodass sie ein Neuladen oder einen geteilten Link übersteht.
- Tabellen mit zehn Zeilen oder mehr bekommen ein kleines Filterfeld darüber.

## [0.8.2] - 2026-09-20

### Changed
- Die Jahresauswahl im Rückblick ist derselbe Schalter wie jede andere Zeitraumauswahl in FinStats (7 T. · 30 T. · 90 T. …).

## [0.8.1] - 2026-09-19

### Added
- Seiten für Darsteller und Regisseure, aus „Meistgeschaute Personen“ im Rückblick oder aus der neuen Reihe **Besetzung und Crew** auf jeder Film-, Serien- oder Episodenseite: alles auf deinem Server, worin sie mitwirken, wie viel davon von wem geschaut wurde und was noch wartet. Sie folgen dem gewählten Zeitraum, wer nur die eigene Statistik sehen darf, sieht dort nur das eigene Schauen, und Esc bringt dich zurück.

## [0.8.0] - 2026-09-19

Der Rückblick hat ein neues Gesicht und mehr zu erzählen

### Added
- **Meistgeschaute Personen**: die Darsteller, mit denen du die meiste Zeit verbracht hast, und die Regisseure hinter dem, was du geschaut hast, als Reihe von Porträts. Besetzung und Crew werden mit der Mediathek gelesen und erscheinen nach dem ersten Lesen der Mediathek nach diesem Update.
- **Dein Jahr in Tagen**: jeder Tag des Jahres als ein Quadrat, umso heller, je mehr du geschaut hast, mit deiner längsten Folge von Tagen am Stück und deinem größten Tag darüber. Zeig auf einen Tag oder geh mit den Pfeiltasten durch den Kalender, um zu sehen, was er enthielt.
- **In Zahlen**: Wiedergaben, Wiedergabezeit, verschiedene Titel, Tage mit Schauen, die längste Folge von Tagen und wie viel davon **erneut geschaut** war, dazu, wie sich das Jahr auf Episoden, Filme und Musik verteilt.
- Das Genre-Kapitel sagt, wie viele Genres du gestreift hast und wie groß der Anteil des ersten war.

### Changed
- Ein neues Design: Es öffnet mit deinem Jahr „noch einmal abgespielt“, der Schlagzeile neben den Postern, die es gefüllt haben, über einer Wellenform des Jahres mit einem Balken pro Woche, die lautesten hervorgehoben, und das Jahr wählst du direkt darüber. Jedes Kapitel hat eine Schlagzeile mit seinem Schlüsselwort in Violett, einen Satz, der die Zahlen trägt, und das Wort des Kapitels groß und blass dahinter. Es endet, wie Filme enden, mit dem Abspann: in den Hauptrollen, Regie, gezeigt auf, Laufzeit.
- Stunden, Wochentage und Monate teilen sich ein Kapitel **Aktivitätsmuster** mit einem Schalter dazwischen, und die Schlagzeile folgt dem, was du ansiehst („Der Freitag holte sich die Krone“).

### Fixed
- **Läuft gerade** auf der Übersicht blieb leer, wenn niemand schaute, sodass die Überschrift zu den Filtern darunter zu gehören schien. Das Feld „Gerade läuft nichts“ ist zurück.

## [0.7.10] - 2026-09-19

### Added
- Mediathek-Artwork: Jede Mediathekkarte und die eigene Seite der Mediathek zeigen das Bild, das Jellyfin dafür verwendet, über den Bild-Proxy von FinStats wie die Poster. Eine Mediathek ohne Bild behält ihr Symbol.

## [0.7.9] - 2026-09-19

### Changed
- Die Karte Genres öffnet wieder als **Liste**, mit dem Radar als zweiter Option am Schalter. Eine vorhandene Wahl bleibt erhalten.

## [0.7.8] - 2026-09-19

### Added
- Eine Radaransicht für die Karte Genres, auf der Übersicht und in Profilen: eine Speiche pro Genre, umgeschaltet zwischen **Radar** und **Liste** in der Ecke der Karte, und die Wahl wird gespeichert. Das Zeigen auf einen Punkt oder die Pfeiltasten zeigen Wiedergabezeit und Anteil dieses Genres.

## [0.7.7] - 2026-09-19

### Fixed
- Filme, die zu einer Sammlung gehören, wurden als „Nicht mehr in der Mediathek“ angezeigt. Mit eingeschaltetem *Filme in Sammlungen gruppieren* gibt Jellyfin die Sammlung statt der darin enthaltenen Filme heraus, sodass FinStats sie nie sah; auf einem Server verbarg das 556 von 1.238 Elementen der Mediathek Filme. FinStats fragt jetzt nach den Filmen selbst, und das Lesen, das dieses Update auslöst, bringt sie zurück und ordnet ihre Wiedergaben wieder zu.

## [0.7.6] - 2026-09-19

### Changed
- In Profilen ist die Karte **Serien** nach unten gerückt, direkt über Genres, sodass eine lange Serienliste Wiedergabezeit, Aktivitätsdiagramm und Toptitel nicht mehr aus dem Blick schiebt. Die Folgen von Tagen am Stück bleiben oben, und eine geöffnete Serie bleibt offen, wenn du den Zeitraum änderst.

## [0.7.5] - 2026-09-19

### Changed
- Die Suche vergleicht Wort für Wort, in beliebiger Reihenfolge, mit beliebigem Text dazwischen, statt die genaue Wortfolge zu verlangen, sodass „alya hides“ *Alya Sometimes Hides Her Feelings in Russian* findet. Groß- und Kleinschreibung, Akzente und Satzzeichen spielen keine Rolle, ein vorangestelltes „The“ wird ignoriert, und ein Vertipper wird verziehen („mentalsit“ findet *The Mentalist*); Wörter mit drei Buchstaben oder weniger müssen weiterhin richtig getippt werden. Bessere Treffer stehen weiter oben, und Musik wird auch über ihren Künstler gefunden.
- Die Suchfelder in Aktivität und im Serverprotokoll vergleichen Wort für Wort über alle ihre Spalten: „alya opera“ findet Wiedergaben dieser Serie in Opera.

## [0.7.4] - 2026-09-19

### Changed
- Die Versionshinweise sind nach Reihen gruppiert: Jede `0.x`-Reihe ist ein Klappbereich mit ihren Versionen, mit dem, worum es in der Reihe ging, wie viele es sind und wann. Nur die neueste beginnt geöffnet, und ein geschlossener Bereich zeigt trotzdem an, welche Version du gerade nutzt.

## [0.7.3] - 2026-09-19

### Fixed
- Die Uhr unter „Läuft gerade“ stockte alle paar Sekunden: Die Aktualisierung alle 5 Sekunden zeichnete sie ebenfalls neu, sodass die Zahl, sobald der Server einen Bruchteil voraus war, zu früh sprang und dann eine Sekunde stehen blieb. Jetzt bewegt nur der Takt einmal pro Sekunde sie, um genau eine Sekunde (gemessen 999 bis 1001 ms auseinander), und die Aktualisierung korrigiert weiterhin nach einer Pause, einem Sprung oder echtem Auseinanderlaufen und landet dabei auf dem Takt.

## [0.7.2] - 2026-09-19

### Added
- Gemeinsames Schauen, live: Jede Karte unter „Läuft gerade“ sagt „Mit maria“, während Leute dasselbe zusammen schauen, wo eine Gruppe früher erst erkannt wurde, wenn die Wiedergaben vorbei waren. Live gelten Leute als zusammen, wenn sie beim selben Titel sind und entweder innerhalb des Gruppenfensters gestartet haben oder an fast derselben Stelle sind, was auch funktioniert, wenn FinStats mitten im Stream neu gestartet wurde.

## [0.7.1] - 2026-09-19

### Changed
- „Läuft gerade“ zählt jede Sekunde: Die Uhr tickt, und der Balken gleitet dazwischen, statt bei jeder Anfrage an den Server zu springen. Der Server wird weiterhin alle 5 Sekunden gefragt und bleibt die maßgebliche Quelle: Die Anzeige gleicht sich bei einer Pause, einem Sprung oder wenn der Server voraus ist neu ab, aber nicht für eine Position, die nur ein paar Sekunden alt ist, da Clients nur etwa alle zehn Sekunden melden. Pausierte Streams stehen still.

## [0.7.0] - 2026-09-19

Wer gemeinsam schaut

### Added
- Gemeinsames Schauen: Wenn verschiedene Personen denselben Titel zur selben Zeit starten, zählt FinStats das als gemeinsames Schauen. Eine Karte **Gemeinsam geschaut** auf der Übersicht zeigt die Gruppen, die Titel, die sie teilen, und ihre gemeinsamen Stunden, jedes Profil zeigt, mit wem diese Person am meisten schaut, geteilte Wiedergaben tragen in der Aktivität ein kleines Personen-Symbol, und die Wiedergabedetails sagen, mit wem geschaut wurde. Es funktioniert über deinen ganzen Verlauf, importierte Wiedergaben eingeschlossen.
- Eine Einstellung, wie nah beieinander die Starts liegen müssen, standardmäßig 60 Sekunden: Rund ein Drittel echter Gruppensitzungen beginnt 6 bis 60 Sekunden auseinander. Die Leute müssen außerdem ein paar Minuten nebeneinander weiterschauen, damit zwei Personen, die zufällig dieselbe Episode öffnen, keine Gruppe sind.

### Changed
- Die Suche öffnet sich mit **Strg + Leertaste** statt mit Strg/⌘ + K.

## [0.6.1] - 2026-09-19

### Added
- Administratoren können den Rückblick einer anderen Person öffnen, oben auf der Seite Rückblick oder mit **Rückblick öffnen** in ihrem Profil. Er spricht dann namentlich über sie statt mit „du“.

### Changed
- Alle anderen sehen weiterhin nur ihren eigenen Rückblick, egal welche Berechtigungen sie haben, und es gibt weiterhin keinen Rückblick des ganzen Servers.

## [0.6.0] - 2026-09-19

Dein Profil und wo du in jeder Serie stehst

### Added
- Serienfortschritt in Profilen: jede Serie, die du angefangen hast, als Balken mit einem Segment pro Episode (gesehen, angefangen oder noch nicht), Staffel für Staffel aufklappbar, sortiert danach, wie nah du am Ende bist, mit eigenen Ansichten für beendete Serien und für alles.
- **Als gesehen markieren** in deinem eigenen Profil, pro Episode, Staffel oder Serie, für das, was du geschaut hast, während nichts aufzeichnete. Die eigenen Gesehen-Markierungen von Jellyfin werden ebenfalls genutzt. Markierungen bleiben in FinStats und ändern nie etwas in Jellyfin; eine aufgezeichnete Wiedergabe lässt sich nicht entmarkieren, deine eigenen Markierungen schon.
- Tage am Stück in Profilen: deine längste Folge von Tagen hintereinander mit einer Wiedergabe, deine aktuelle Folge und an wie vielen Tagen du etwas geschaut hast.
- **Mein Profil** in der Seitenleiste.
- Die Aktivität zeigt unter dem Fortschrittsbalken, wo die Wiedergabe aufhörte (zum Beispiel `44:15 / 45:00`), und Datum und Uhrzeit unter „vor 3 Std.“. Aus Jellystat importierte Wiedergaben haben keine Stoppposition, weil Jellystat nie eine aufgezeichnet hat.

### Changed
- Episoden, die fehlen oder noch nicht ausgestrahlt wurden, gelten nicht mehr als Teil deiner Mediathek. Jellyfin führt sie als Platzhalter ohne Datei, und sie zählten bisher zu den Episodensummen.

## [0.5.1] - 2026-09-19

### Added
- Ein Logo: eine Play-Taste, in drei Diagrammbalken geschnitten, in der Seitenleiste und auf den Anmeldebildschirmen, als scharfes SVG mit einem `favicon.ico` in mehreren Größen für Browser und Lesezeichenleisten, die eines wollen, dazu ein Startbildschirm-Symbol für Handys und Tablets.

## [0.5.0] - 2026-09-19

Du entscheidest, wer was sieht

### Added
- Berechtigungen unter Einstellungen → Zugriff, für alle auf einmal oder Person für Person: **Aktivität aller sehen** (Statistik und Verlauf anderer, die Seite Benutzer, jeder Live-Stream), **Netzwerkdetails sehen** (IP-Adressen, Geräte-IDs, lokal oder extern), **Serverdetails sehen** (die Serverseite, das Serverprotokoll, fehlgeschlagene Anmeldungen, Dateipfade) und **FinStats verwalten** (Einstellungen, Aufgaben, Import, Wiedergaben löschen).
- Anmeldung pro Person, um ein paar Leute hereinzulassen, ohne FinStats für jeden Jellyfin-Benutzer zu öffnen.
- Änderungen gelten sofort, auch das Entziehen von Zugriff, und alles wird vom Server durchgesetzt, nicht durch das Verstecken von Schaltflächen.

### Changed
- Jellyfin-Administratoren haben weiterhin alles und sind die Einzigen, die Berechtigungen ändern können; wer FinStats verwalten darf, kann nichts erteilen, weder sich selbst noch anderen.
- Der Jahresrückblick bleibt persönlich, egal was erteilt ist.
- Für bestehende Installationen ändert sich nichts, bis du etwas erteilst.

## [0.4.5] - 2026-09-19

### Changed
- FinStats steht unter der **GNU General Public License v3.0**. Eine frühere Version wurde nie veröffentlicht, also erschien nichts unter anderen Bedingungen. Die Lizenzen der mitgelieferten Schriften liegen ihnen bei.

## [0.4.4] - 2026-09-19

### Added
- Esc geht einen Schritt zurück: Auf einer geöffneten Mediathek, einem Benutzer oder einem Titel bringt es dich dorthin zurück, woher du kamst, mit der Liste genau so, wie du sie verlassen hast. Alles Geöffnete schließt sich zuerst (Wiedergabedetails, Suche, ein Dropdown, das Handymenü), und Esc bleibt unbeachtet, während du tippst oder ein Diagramm mit der Tastatur bedienst.

## [0.4.3] - 2026-09-19

### Changed
- FinStats merkt sich, was jede Seite zuletzt gezeigt hat: Die Rückkehr zu einem Profil, einer Mediathek oder einem Zeitraum, den du schon angesehen hast, zeigt ihn sofort, samt Profilbild, während frische Zahlen dahinter laden, und die Seite zeichnet sich nur neu, wenn sich etwas geändert hat. Das Gemerkte lebt im Browser-Tab und wird verworfen, wenn du dich abmeldest oder etwas änderst.
- Ladeplatzhalter erscheinen nur noch, wenn das Laden wirklich langsam ist. Vorher zeigte jede Seite sie mindestens eine Drittelsekunde lang.

### Fixed
- Mediatheken, die in Jellyfin gelöscht wurden und nie eine Wiedergabe hatten, werden nicht mehr aufgeführt. Eine gelöschte Mediathek mit Verlauf wird weiterhin gezeigt, als entfernt markiert.

## [0.4.2] - 2026-09-19

### Fixed
- Auf der Seite Benutzer stand das Etikett **Admin** in einer eigenen Zeile und machte diese Zeile höher als die anderen. Etiketten stehen jetzt neben dem Namen.

## [0.4.1] - 2026-09-19

### Fixed
- Das Wort „null“ erscheint außerhalb von Dezember und Januar, wenn es kein Rückblick-Banner zu zeigen gibt, nicht mehr unter dem Titel der Übersicht.

## [0.4.0] - 2026-09-19

Sehen, was sich geändert hat, ohne die App zu verlassen

### Added
- Tab **Versionshinweise** mit jeder Version, die gerade laufende markiert.
- Die Version in der Statusleiste verlinkt dorthin, und der Tab zeigt nach einem Update einen Punkt, bis du nachgesehen hast.

### Changed
- Das Neuzuordnen des Verlaufs zu umbenannten Elementen ist weit schneller: 18 Sekunden auf eine Drittelsekunde bei 666 verwaisten Titeln, sodass der Start auch bei wachsendem Verlauf sofort bleibt.

## [0.3.3] - 2026-09-19

### Changed
- Die Mediathek wird direkt nach dem Ende der Jellyfin-Aufgabe **Scan Media Library** neu gelesen, nie nach einem eigenen Timer und nie während eines Scans. FinStats hat gegenüber Jellyfin immer nur gelesen, und es startet dort nie einen Scan.
- Ein wöchentliches Lesen als Sicherheitsnetz deckt Server ab, die auf Echtzeitüberwachung setzen, wo die Scan-Aufgabe vielleicht nie läuft.
- Das alte Intervall ist jetzt nur noch ein Rückfall, wenn das Folgen abgeschaltet ist oder der Server keine Scan-Aufgabe meldet.

### Added
- Schalter **Dem Mediathek-Scan von Jellyfin folgen** unter Einstellungen → Erfassung, standardmäßig an.

## [0.3.2] - 2026-09-19

### Fixed
- Der Verlauf übersteht Umbenennungen. Jellyfin gibt einer umbenannten Datei eine neue ID, was ihre früheren Wiedergaben unter dem rohen Ordnernamen (`Title (2010) [tmdbid-…] [imdbid-…]`) ohne Poster zurückließ. Diese Wiedergaben werden dem aktuellen Mediathekeintrag wieder zugeordnet: über die TMDB-, IMDb- oder TVDB-ID, wenn der alte Name eine trägt, sonst über bereinigten Titel und Jahr, und Episoden über Serie plus Staffel- und Episodennummer. Eine Zuordnung muss eindeutig sein, damit wirklich gelöschte Titel bleiben, wie sie sind.
- Aus Jellystat importierte Live-TV-Sender wurden als Filme gezählt, weil Jellystat keinen Elementtyp aufzeichnet. Sie werden jetzt als Live-TV erkannt, und neue Live-TV-Wiedergaben werden mit dem eigenen Typ von Jellyfin erfasst.

### Changed
- Live-TV bleibt komplett aus dem Rückblick heraus: Toplisten, Summen, Persona, Rekorde und Rang.

### Added
- `finstats relink`, um den Verlauf auf Wunsch neu zuzuordnen. Es läuft auch beim Start und nach jedem Lesen der Mediathek.

## [0.3.1] - 2026-09-19

### Changed
- Der Rückblick ist strikt persönlich: Alle, Administratoren eingeschlossen, sehen nur ihren eigenen, und die Ansicht „Alle“, die Benutzerauswahl und das Serverkapitel sind weg.
- Der Rückblick öffnet mit dem Jahr, das „fertig“ ist: im Dezember das laufende Jahr, sonst das vorherige. Andere Jahre und „Letzte 12 Monate“ bleiben einen Klick entfernt.

## [0.3.0] - 2026-09-19

Dein Jahr im Rückblick

### Added
- Tab **Rückblick**, eine scrollende Geschichte eines Jahres: geschaute Stunden, Top-Serien, Filme, Tracks und Genres und eine Zuschauer-Persona (Nachteule, Frühaufsteher, Wochenendkrieger, Binge-Watcher, Filmfan, Musikliebhaber, Gewohnheitstier) mit Diagrammen nach Tageszeit und Wochentag.
- Das Jahr Monat für Monat, mit dem Titel, der jeden Monat geprägt hat.
- Rekorde: größter Tag, größter Binge, längste Folge von Tagen, längste einzelne Wiedergabe, am häufigsten erneut geschauter Titel, erste Wiedergabe des Jahres und der älteste geschaute Titel.
- Entdeckungen: neu angefangene Serien, beendete Filme und Episoden und die Serien, die genau eine Episode bekamen.
- Ein einzeiliges Banner auf der Übersicht im Dezember und Januar, wenn ein Rückblick fertig ist.

## [0.2.0] - 2026-09-19

Mehr, als Jellystat je aufgezeichnet hat

### Added
- Eine Zeitleiste für jede Wiedergabe: Start, Pause, Fortsetzen, Sprung, Audio- und Untertitelwechsel, direkte Wiedergabe, die zur Transkodierung wird, und Stopp. Dazu die Zahl der Pausen und Sprünge, wo die Wiedergabe fortgesetzt wurde und ob der Zuschauer im lokalen Netzwerk war.
- Tab **Server**: Jellyfin-Version, ausstehende Updates und Neustarts, Speicherbelegung pro Mediathek (Jellyfin 10.11+), Plugins, Ergebnisse geplanter Aufgaben und jedes registrierte Gerät.
- Auswertungen unter **Wiedergabe**: gleichzeitige Streams über die Zeit, welche Clients Transkodierungen erzwingen, wie weit Leute kommen, bevor sie aufhören, lokale gegenüber externen Wiedergaben und Sehverhalten.
- Woraus deine Mediathek besteht: Auflösungen, Codecs, Dynamikumfang, Container, Titel pro Jahrzehnt, Zugänge pro Monat, die größten Titel und die Titel, die nie jemand geschaut hat, abgeglichen mit dem Verlauf von FinStats und den eigenen Gesehen-Markierungen von Jellyfin.
- Übersicht: Spitzenwert gleichzeitiger Streams, geschätzte gestreamte Datenmenge, Anteil externer Wiedergaben, Genres nach Wiedergabezeit und fehlgeschlagene Anmeldungen.
- Titelseiten: Links zu IMDb und TMDB, Studios, Farbtiefe und Bildrate und wer den Titel in Jellyfin als gesehen markiert hat.
- Benutzerseiten: Genres sowie die Filme, Episoden und Favoriten, die Jellyfin verzeichnet hat.

### Fixed
- Die Zeitangabe auf einer Karte unter „Läuft gerade“ läuft nicht mehr aus der Karte heraus.
- Lange Beschriftungen auf der Seite Wiedergabe wurden zu früh abgeschnitten.
- Eine Serie, die die Mediathek verlassen hat, zeigt weiterhin ihre Staffel- und Episodenliste.

## [0.1.0] - 2026-09-19

Erste Version

### Added
- Live-Erfassung aus den Sitzungen von Jellyfin: Benutzer, Titel, Client, Gerät, IP-Adresse, Wiedergabemethode, Gründe für die Transkodierung und Hardwarebeschleunigung, Video-, Audio- und Untertiteldetails sowie geschaute gegenüber pausierter Zeit.
- Anmeldung mit deinem Jellyfin-Konto. Eine Einrichtung in zwei Schritten erstellt einen eigenen API-Schlüssel für FinStats, und Passwörter werden nie gespeichert. Benutzer ohne Administratorrechte können sich optional anmelden, um nur ihre eigene Statistik zu sehen.
- Seiten für Übersicht, Aktivität, Benutzer, Mediatheken, Wiedergabe und Serverprotokoll, eine Befehlspalette (Strg/⌘ K) und eine Live-Statusleiste.
- Jellystat-Import: Große Sicherungen werden auf die Festplatte gestreamt und in Sekunden importiert, und dieselbe Datei zweimal zu importieren ist sicher.
- Eine einzige kleine Binärdatei mit eingebetteter Weboberfläche und einer SQLite-Datei, dazu ein Docker-Image.
