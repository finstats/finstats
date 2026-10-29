# Lista zmian

Wszystko, co zmieniło się w FinStats, od najnowszych. Ten plik jest wkompilowany w program
i to właśnie jego treść pokazuje karta **Lista zmian** w aplikacji.

Format: `## [version] - date` albo `## [version] - first day to release day` dla wydania przygotowywanego
przez kilka dni, a pod nim `### Added`, `### Changed`, `### Security`, `### Performance`, `### Stability`, `### Fixed`
lub `### Removed` (Dodano, Zmieniono, Bezpieczeństwo, Wydajność, Stabilność, Naprawiono, Usunięto), po jednym punkcie
na zmianę. Bezpieczeństwo to coś, co ktoś mógł wykorzystać w złej wierze, Wydajność to coś, co działa szybciej lub lżej,
Stabilność to coś, co nie może już ulec awarii, zatrzymać się, wyciekać ani tracić danych, a Naprawiono to coś, co dawało
błędną odpowiedź. Wydanie `x.y.0` ma pod nagłówkiem krótki wiersz tytułowy: aplikacja używa go jako nagłówka całej serii.

## [2.3.1] - 2026-10-26 to 2026-10-28

### Security
- **Konfiguracja prosi o jednorazowy kod**: dopóki FinStats nie jest skonfigurowany, przy starcie zapisuje kod w swoim dzienniku (`docker logs finstats`), a kreator prosi o niego, zanim sprawdzi adres lub cokolwiek ustawi. W przeciwnym razie ten, kto pierwszy dotarł do portu, mógł skierować FinStats na własny Jellyfin i zostać jego administratorem albo użyć testu, by rozejrzeć się po sieci.
- **Dostęp do bibliotek idzie za Jellyfin**: każdy widzi tylko biblioteki, które Jellyfin mu otwiera, a w nich tylko tytuły, które Jellyfin mu pokazuje w ramach jego limitu kategorii wiekowej, tagów i reguł dla pozycji bez kategorii: na stronach bibliotek, w wyszukiwaniu, obsadzie, obrazkach, Ostatnio dodanych, liście Do obejrzenia, Wkrótce, kanale kalendarza i powiadomieniach o nowych tytułach. Każda zalogowana osoba mogła przeglądać każdą bibliotekę.
- Link mógł zaraz po zalogowaniu przenieść kogoś z powitania na inną stronę. Teraz otwierane są tylko strony samego FinStats.
- Dziennik serwera i nieudane logowania w statystykach pokazywały adresy logowania i aktywność wszystkich każdemu, kto *Widzi serwer*. Cudze wiersze wymagają teraz uprawnienia *Widzi aktywność wszystkich*, a adres *Widzi szczegóły sieci*, jak wszędzie indziej.
- Osobisty cel powiadomień mógł sięgnąć do sieci lokalnej przez nazwę, która przy drugim sprawdzeniu dawała inną odpowiedź. Połączenie idzie teraz na sprawdzony adres, więcej zakresów, które nigdy nie prowadzą do internetu, liczy się jako lokalne, własny adres publiczny domu też jest odrzucany, a nazwa, której nie da się sprawdzić, jest odrzucana tymi samymi słowami co lokalna.
- Za proxy, które dodaje własny wiersz `X-Forwarded-For`, albo z sieci IPv6 każda próba logowania mogła przychodzić z nowego adresu, więc limit nigdy nie działał. Sieć IPv6 /48 ma teraz 50 prób na pięć minut, pełny limit odrzuca nowy adres zamiast zapominać stary, cały serwer zatrzymuje się przy 60 nieudanych próbach na minutę, a nieudane logowanie trafia do dziennika audytu raz na adres i okno.
- `FINSTATS_TRUST_PROXY` wskazuje proxy (adresy lub zakresy), których nagłówkom `X-Forwarded-For`, `X-Forwarded-Host` i `X-Forwarded-Proto` się wierzy, a `1` lub `true` wierzy teraz tylko rozmówcy z pętli zwrotnej lub sieci prywatnej. Po włączeniu każdy rozmówca mógł wybrać sobie adres; wyłączone za proxy sprawiało, że dziesięć błędnych prób blokowało wszystkich.
- Gdy użytkownicy zostali już wczytani, logowanie na nazwę nieznaną, wyłączoną, usuniętą lub niewpuszczaną do FinStats dostaje odpowiedź jak przy błędnym haśle bez pytania Jellyfin i nie wcześniej, niż odpowiedziałby Jellyfin. Takie nazwy mogły blokować konta Jellyfin, obchodzić regułę Jellyfin o dostępie zdalnym albo chować się za adresem FinStats.
- Ktoś, kogo Jellyfin trzyma w sieci domowej (wyłączone *Zezwalaj na połączenia zdalne*), loguje się do FinStats tylko z niej, bo Jellyfin oceniał to ustawienie po adresie samego FinStats, a wczytanie użytkowników, które nie mówi, czy ktoś jest wyłączony, nie wpuszcza go.
- Ciasteczko sesji jest oznaczane jako Secure, gdy tylko strona logowania działa przez https, także za proxy, które tego nie zgłasza.
- Udane logowanie nie wybacza już wcześniejszych prób zgadywania. Przez https ciasteczko sesji jest ciasteczkiem `__Host-`, którego strona pod sąsiednią nazwą nie może podmienić i które nie jest przyjmowane pod żadną inną nazwą, a wylogowanie kończy każdą sesję, którą przesłała przeglądarka.
- Tylko klucz kalendarza otwiera kanał kalendarza z jego adresu; każdy inny klucz idzie w nagłówku.
- Klucz API nie może już nadawać uprawnień, zmieniać tego, kto wchodzi i co jest publiczne, przywracać kopii zapasowej ani dodawać, zmieniać czy testować usługi lub celu powiadomień: każde z nich trwa dłużej niż klucz, więc wymaga zalogowanej przeglądarki.
- Każda odpowiedź niesie `Cross-Origin-Opener-Policy: same-origin` i `Permissions-Policy`, odpowiedzi API niosą `Cross-Origin-Resource-Policy: same-origin` i są `no-store`, chyba że mówią inaczej, a CSP nie dopuszcza już obrazków `blob:`. Strona, która otworzyła FinStats, mogła później zamienić jego kartę we własną stronę logowania.
- Tam, gdzie FinStats zna schemat, przez który jest osiągany (`X-Forwarded-Proto` zaufanego proxy albo adres publiczny), zapis z innego schematu jest odrzucany jako międzywitrynowy.
- `/auth/me`, strona tytułu i Stan biblioteki linkują do Jellyfin pod jego adresem publicznym; adres, pod którym sam FinStats czyta Jellyfin, trafia tylko do administratorów.
- Obrazek jest pobierany z Jellyfin tylko dla czegoś, co FinStats ma, także plakaty i karty opublikowanego profilu, więc nie da się poprosić o obrazki innej biblioteki po identyfikatorze.
- Klucze prywatnego trackera są usuwane także z błędów kolejki pobierania, zarówno po kształcie, jak i po nazwie, a ktoś, kto ma tylko *Widzi pobierane pliki*, dowiaduje się, która usługa zawiodła i jak, ale nigdy jej wewnętrznego adresu ani tego, dokąd przekierowuje.
- Strona Serwer podaje ostatniego użytkownika urządzenia tylko z uprawnieniem *Widzi aktywność wszystkich*, a Niepowiązane multimedia wymagają go obok uprawnienia *Zarządza FinStats*. Bez niego sekcja Zadania liczy tylko własne odtworzenia i strumienie, usunięta biblioteka jest wymieniana według własnych odtworzeń, a czekającej tablicy Tautulli nie da się anulować.
- Ustawienia pokazują usługę adresu publicznego tylko po nazwie hosta, a adres Jellyfin i folder danych tylko administratorom, a tytuł, który zniknął z biblioteki, jest opisywany wyłącznie na podstawie odtworzeń, które czytający może zobaczyć.
- Kopia zapasowa Tautulli, której tabele nie były zwykłymi tabelami, mogła wstrzymać każdy inny zapis aż do ponownego uruchomienia FinStats. Taki plik jest odrzucany, a SQLite czyta kopię w trybie obronnym.
- Przesłanie większe niż 4 GB jest odrzucane, a spakowana kopia zapasowa Tautulli jest rozpakowywana najwyżej do rozmiaru, który deklaruje; jedno i drugie mogło zapełnić dysk, na którym leży baza danych FinStats.
- Odpowiedź od Jellyfin, Sonarr, Radarr, Seerr, sprawdzania adresu publicznego lub pobierania GeoIP jest czytana najwyżej do dopuszczalnego rozmiaru, liczonego po rozpakowaniu.
- Każdy użytkownik Jellyfin mógł sprawić, że FinStats przestawał zapisywać odtworzenia wszystkich, zgłaszając tysiące krótkich odtworzeń jednego tytułu. Ustalanie, kto oglądał razem, zajmuje teraz jedno przejście i nie trzyma przy tym blokady.
- Odtworzenia zapisane jedno po drugim albo jeden tytuł odtwarzany na zmianę na kolejnych urządzeniach nie łączą już obcych osób w jedną grupę, a ktoś, kto widzi tylko siebie, dowiaduje się tylko o osobach, obok których szło jego własne odtworzenie, i tylko na ten czas.
- Link wciąż mógł po zalogowaniu przenieść kogoś z powitania na inną stronę, przez ścieżkę taką jak `/.//`.
- Menedżer nie może już zmieniać, ile kopii zapasowych jest przechowywanych ani jak często są zapisywane, okna scalania (teraz najwyżej godzina) ani tego, kiedy czytani są użytkownicy, dziennik aktywności i kopie zapasowe, a kopie menedżera nigdy nie wypychają kopii administratora; adresy domowe i reguły podróży wymagają tego, czego wymagają alerty, a import historii wymaga także *Widzi aktywność wszystkich*.
- Zapisany klucz, token lub hasło trafia tylko do serwera, dla którego go zapisano, a cel powiadomień jest tworzony lub zmieniany tylko po zalogowaniu, nigdy kluczem API. Administrator może wyłączyć lub usunąć cudzy osobisty cel, ale nie może go przenieść, wyłączyć w nim sprawdzania certyfikatów ani odczytać jego tematu ntfy czy klucza użytkownika Pushover.
- Webhooki Discord i Slack muszą używać `https`, bo ich adres jest ich tokenem.
- Zalew nieudanych logowań nie może już ukryć prawdziwego ani zasypać alertów: nazwy, które nie są kontami, dają jedno powiadomienie na kwadrans, które nikogo nie wymienia, alerty wychodzą pierwsze, a Slack i Discord pokazują treść wiadomości jako tekst i nikogo nie pingują.
- Miejsce w Podsumowaniu, Wydajność, powiadomienie i opublikowany profil nie mówią więcej niż strony: żadnego miejsca ani udziału w czasie wszystkich dla kogoś, kto widzi tylko siebie, najbardziej obciążone chwile bez *Widzi aktywność wszystkich* tylko jako dni z trzema strumieniami lub więcej, żadnego nieudanego zadania ani połączenia dla kogoś, kto nie może zarządzać FinStats, a profil publiczny zmienia się raz dziennie.
- Kopia zapasowa Tautulli z kolumną generowaną jest odrzucana, a obrazek dla zaimportowanego odtworzenia jest pobierany dopiero, gdy biblioteka ma jego tytuł.
- Przywracanie nigdy nie otwiera FinStats szerzej, niż jest otwarty: ustawienia wracają tylko na życzenie, logowanie, uprawnienia i profile publiczne stają się tym, na co pozwalają zarówno ta instalacja, jak i kopia, a adresy publiczne pozostają adresami tej instalacji. `finstats restore <file>` przywraca tylko historię, tak jak strona, a z `--with-settings` także ustawienia.
- Przywrócenie samej historii nie przywraca już adresów domowych, wyciszonych alertów, wpisów audytu udających wpisy tej instalacji ani wskazanych tytułów, których biblioteka nie ma, a żadne przywracanie nie przywraca odtworzeń, wierszy dziennika ani urządzeń z datą sprzed 1990 roku lub późniejszą niż jutro. Przywrócone odtworzenie liczy się jako zaimportowane bez względu na to, z jakiego źródła rzekomo pochodzi, więc ponowne uruchomienie nigdy go nie kontynuuje.
- Folder danych należy wyłącznie do użytkownika samego FinStats (0700, jego pliki 0600, `umask 077` w obrazie). FinStats odmawia działania jako root na folderze danych należącym do innego użytkownika i podaje, jakiego `docker exec -u` użyć, a entrypoint zmienia właściciela tylko folderu i własnych wpisów FinStats.
- Nazwa celu nie może zawierać znaków sterujących, a nazwa urządzenia, osoby lub tytułu nie może już podrobić wiersza w dzienniku.
- Parametr trasy w linku nie może już sięgnąć innej ścieżki API niż ścieżka jego strony, a konstruktor elementów FinUI ustawia adres tylko wtedy, gdy jest http(s) lub względny, nigdy `srcdoc`.
- Wyszukiwanie, opublikowane profile i praca uruchamiana ręcznie nie mogą już zająć FinStats: wyszukiwanie kończy się na 200 znakach, profil jest przechowywany osobno dla każdego języka, karty czekają w krótkiej kolejce, a zadanie, sprawdzanie adresu, pobieranie bazy lokalizacji i wiadomość testowa są uruchamiane tylko co jakiś czas, bez względu na to, jak się o nie prosi, przy czym wiadomości testowe liczone są dla każdego nadawcy. Pobieranie bazy lokalizacji wymaga też tego, czego wymaga czytanie strony Bezpieczeństwo.

### Performance
- Opublikowany profil jest obliczany najwyżej raz na minutę, a nie od nowa dla każdego obrazka i karty na nim.
- Historia osoby jest przeszukiwana pod kątem alertów bezpieczeństwa jeden raz naraz, bez względu na to, ile odtworzeń się zaczyna.
- Zalogowanie lub wylogowanie, unieważnienie klucza albo zmiana uprawnień jednej osoby każe sprawdzić się ponownie tylko tym połączeniom na żywo, których to dotyczy.
- Tablica połączeń Tautulli czyta historię raz, a nie raz dla każdego użytkownika Plex.
- Obrazek, którego brakuje w pamięci podręcznej, jest pobierany jednym żądaniem, a pozostałe czekają, a o taki, którego źródło nie wydało, nie pyta się ponownie przez 30 sekund.
- Pliki interfejsu WWW są kompresowane gzipem raz i przechowywane, a czcionki nie są kompresowane ponownie.

### Stability
- Kopia zapasowa w starszym formacie Jellystat jest czytana wiersz po wierszu zamiast w całości, a wiersz dłuższy niż w jakiejkolwiek kopii jest odrzucany.
- Dwa jednoczesne przesłania z Tautulli nie zapisują już do jednego pliku.
- Numer strony daleko za końcem tytułów biblioteki lub Stanu biblioteki daje pustą stronę, a nie błąd.
- Import lub przywracanie, którego przesyłanie zostało przerwane, nie blokuje już wszystkich innych aż do ponownego uruchomienia, i ani ono, ani import porzucony przed rozpoczęciem nie zostawia po sobie pliku.
- Jeden wiersz kopii zapasowej jest czytany najwyżej do 8 MB, a kopie zapasowe Tautulli przychodzą po jednej.
- Odtworzenie przechowuje najwyżej 2000 zdarzeń, a import pomija odtworzenie, którego początek i następujący po nim koniec nie mieszczą się między 1990 rokiem a jutrem.
- Jeden numer odcinka daleko poniżej zera nie zawiesza już każdego wczytywania biblioteki: Stan biblioteki (także jego podsumowanie), Wydajność, ponowne łączenie i wskazywanie dodają i porównują liczby bez przepełnienia, a kompilacja wydania sprawdza przepełnienie, więc to, co umknie, zatrzymuje jedno zadanie, zamiast się zapętlić.
- Numer sezonu, odcinka lub końcowy jest zachowywany tylko tam, gdzie Jellyfin mógłby go przechować (od 0 do 2 147 483 647), czy pochodzi z Jellyfin, Jellystat, czy z Tautulli.
- Kopia zapasowa Tautulli z powtarzającymi się identyfikatorami jest odrzucana, bo kilka MB wierszy pod jednym identyfikatorem wyczerpywało pamięć procesu, a kopia jest otwierana jako niezmienna i nie zostawia po sobie `-wal` ani `-shm`.
- Połączenie musi wysłać żądanie w ciągu 15 sekund, a naraz obsługiwanych jest najwyżej 512: kilkaset na wpół otwartych gniazd wyłączało FinStats. Ramka w kanale na żywo jest ograniczona do 4 KB.
- Kilka kilobajtów z Sonarr, Radarr lub Seerr nie może już zamienić się w gigabajty pamięci: lista nie przechowuje więcej, niż FinStats z niej czyta.
- Cel powiadomień, który nigdy nie odpowiada, wstrzymuje tylko siebie: do celów wysyła się równolegle, a cel, który ciągle przekracza czas, jest pomijany przez dziesięć minut.
- Pozostały czas i łączna prędkość kolejki, liczby przywróconego odtworzenia i dzień, w którym odtworzenie opuszcza kosz, nie mogą już się przepełnić.
- Przywracanie czyta cały plik, zanim weźmie blokadę zapisu, odrzuca plik, który rozpakowuje się do rozmiaru dużo większego niż własny, i zapisuje partiami, więc kolektor w tym czasie dalej rejestruje.
- Kopie zapasowe są nazywane w UTC i nigdy nie nadpisują istniejącego pliku, a przycinane są tylko własne kopie FinStats sprzed aktualizacji.
- Wiersz dziennika serwera z datą w przyszłości nie wstrzymuje już czytania dziennika aktywności: jest on czytany od najnowszego wiersza, który nie jest w przyszłości.
- Żądania przychodzące jednocześnie na opublikowany link obliczają jego odpowiedź raz, a jego karty są rysowane na osobnym torze, więc posiadacz linku nie może już sprawić, że karty Podsumowania właściciela odpowiadają 503.

### Fixed
- Nieudane logowania na konto to jedno ostrzeżenie na okno, które przy dwudziestu zamienia się w alert, a nazwa liczy się przeciw kontu tylko wtedy, gdy wpisano ją w całości ("x alice" nie liczy się już przeciw alice).
- Wiadomości Discord i Slack są najpierw escapowane, a dopiero potem przycinane: nazwa urządzenia pełna Markdownu sprawiała, że Discord odrzucał każdy alert, który ją wymieniał.
- Publiczny adres IPv6 jest akceptowany dla osobistego celu.
- Za proxy, które samo loguje ludzi (nginx `auth_basic`), ciasteczko FinStats nie jest już odrzucane.
- Nazwa wyświetlana zawierająca `{{url}}` jest pokazywana tak, jak ją zapisano, w podglądzie opublikowanej strony.
- Podsumowanie dla roku 9999 pokazuje rok domyślny zamiast błędu.
- Strona na żywo, której uprawnienie odebrano i przywrócono, gdy była otwarta, znów jest aktualizowana bez przeładowania.

### Removed
- **Synnmørsk** nie jest już jednym z języków FinStats: przeglądarka, która go wybrała, wyświetla nynorsk, język pisany, z którego powstał.

## [2.3.0] - 2026-10-07 to 2026-10-26

Wydajność, kanał na żywo, Tryb prosty i języki

### Added
- **Języki**: FinStats mówi w 23 językach. Angielski w odmianie brytyjskiej i amerykańskiej oraz norweski w odmianach bokmål i nynorsk pisze i sprawdza opiekun projektu; duński, niemiecki, hiszpański, fiński, francuski, hindi, islandzki, włoski, japoński, koreański, niderlandzki, polski, portugalski (Brazylia), rosyjski, szwedzki, turecki, ukraiński i chiński (uproszczony i tradycyjny) to tłumaczenia, których nikt tutaj nie sprawdził, a to, czego jeszcze w nich brakuje, mówią po angielsku. Każda strona obu układów, profil publiczny, Lista zmian i to, co serwer odpowiada słowami (komunikat o błędzie, karty Podsumowania, opis tego, co robi zadanie Jellyfin), są w języku wybranym w Ustawieniach → Wygląd → Język, osobno w każdej przeglądarce; dopóki żaden nie zostanie wybrany, decyduje język przeglądarki. Liczby, daty, jednostki i wyliczenia są zapisywane tak, jak zapisuje je dany język. Skrypt może poprosić o konkretny język nagłówkiem `Accept-Language`.
- **Tryb prosty**: drugi układ dla osób przesiadających się z Plex, zbudowany tak jak klasyczny serwer statystyk dla Plex. Pasek u góry zamiast menu i same strony ze statystykami: Start, Biblioteki, Użytkownicy, Historia, Wykresy i Ustawienia, a do tego koło zębate z profilem, Ustawieniami, Listą zmian, motywem i wylogowaniem. Wybiera się go osobno w każdej przeglądarce w Ustawieniach → Wygląd → Układ. Wczytuje tylko swoją połowę FinStats, więc uruchamia się szybciej; Podsumowanie, „Razem”, „W drodze”, Bezpieczeństwo i strony serwera zostają w układzie FinStats. Używa kolorów samego FinStats, jasnych i ciemnych, i nic się w nim nie porusza.
- **Start w Trybie prostym**: to, co jest odtwarzane, z kartą dla każdego strumienia (jak jest odtwarzany, z domu czy spoza niego, kiedy się skończy), statystyki oglądania jako karty rankingowe według liczby odtworzeń lub czasu trwania z ostatnich N dni, zawartość każdej biblioteki i to, co ostatnio dotarło.
- **Wykresy w Trybie prostym**: odtworzenia według rodzaju multimediów (dzień po dniu, według dnia tygodnia i godziny, dziesięć najczęściej używanych aplikacji i osób), według sposobu odtwarzania (dzień po dniu, największa liczba jednoczesnych strumieni każdego dnia, rozdzielczość pliku i rozdzielczość wysyłana, aplikacje i osoby) oraz miesiąc po miesiącu, według liczby odtworzeń lub czasu trwania, dla wszystkich albo dla wybranych osób.
- **Biblioteki, Użytkownicy i tytuły w Trybie prostym**: tabele wszystkich bibliotek i osób z tym, co odtworzono ostatnio, na czym i skąd (adres tylko z uprawnieniem *Widzi szczegóły sieci*), oraz strona każdej biblioteki, osoby i tytułu z jej liczbami i własną historią.
- Dla skryptów: `/api/stats/split` (odtworzenia według aplikacji, osoby, rozdzielczości, dnia lub miesiąca, z podziałem na rodzaj multimediów lub sposób odtwarzania); rankingi według liczby różnych widzów (`sort=users`); aktywność zawężona do obejrzanych odtworzeń (`watched=1`); analizy tylko dla wskazanych części (`parts=`); szczyt rozbity na transkodowania, strumieniowania bezpośrednie i odtwarzania bezpośrednie, także dla każdego dnia; zawartość każdej biblioteki i ostatnie odtworzenie w niej; ostatnio dodane według rodzaju (`kind=`); mapa cieplna według rodzaju multimediów; ostatnie odtworzenie danej osoby; informacja, czy strumień jest odtwarzany z domu.
- **Kanał na żywo**: Pulpit, pobieranie na stronie „W drodze” i pasek stanu same pozostają aktualne dzięki jednemu połączeniu WebSocket na kartę (`/api/live`), więc to, co jest odtwarzane i pobierane, zmienia się na ekranie w chwili, gdy się zmienia, a nie przy następnym sprawdzeniu. Za odwrotnym proxy, które nie przepuszcza przejścia na WebSocket, albo na połączeniu, które zerwało się bez słowa, strony jak dotąd pytają co kilka sekund. `/api/now-playing`, `/api/downloads` i `/api/summary` pozostają bez zmian, dla skryptów.
- **Serwer → Wydajność**: czy ten serwer da sobie radę, na podstawie tego, co widział FinStats. Najbardziej obciążone chwile w całej historii i w ostatnich 90 dniach: liczba jednoczesnych strumieni oraz ile z nich miało obraz transkodowany sprzętowo, ile bez akceleracji sprzętowej, a ile bez żadnej informacji (historia zaimportowana z Tautulli pochodzi z Plex i jest pomijana). Każdy dysk, na którym leżą biblioteki, i jak szybko zapełnia go to, co Jellyfin dodał w ciągu sześciu miesięcy (dysk zapełniony niedawno jest mierzony od miesiąca po pierwszym pliku, bo pierwszy miesiąc to samo zapełnianie), a tam, gdzie Jellyfin 10.11 lub nowszy podaje wolne miejsce, na jak długo go wystarczy w tym tempie. FinStats nigdy nie mówi, ile strumieni serwer mógłby obsłużyć.
- **Eksportuj .ics** w zakładce „W drodze” → Nadchodzące: lista z nałożonymi filtrami, pobrana jako plik kalendarza do jednorazowego zaimportowania. Nie aktualizuje się sama; klucz kalendarza z Ustawień → Klucze API nadal daje kalendarz, który to robi.
- **Warto sprawdzić**: Wydajność zwraca uwagę, gdy każde transkodowanie zgłoszone przez Jellyfin w ciągu 90 dni odbyło się bez akceleracji sprzętowej, choć transkodowanie jest częste, gdy skanowanie biblioteki w Jellyfin zajmuje jedną czwartą czasu lub więcej oraz gdy na dysku zostało miejsca na mniej niż sześć miesięcy. Każda uwaga ma swoje liczby i można ją odłożyć na bok, tak jak wyniki Stanu biblioteki.
- **Z czego składa się twoja biblioteka** można zawęzić do jednej biblioteki lub kilku, w Bibliotekach i na stronie każdej biblioteki (`library_id=a,b`).
- **Formaty daty i godziny** w Ustawienia → Wygląd: data jako 31.01.2026, 31/01/2026, 31-01-2026, 01/31/2026, 2026-01-31, 2026.01.31 lub 2026/01/31, a godzina w systemie 24- lub 12-godzinnym, z zerem na początku lub bez, albo każda tak, jak pisze ją język. Zapamiętywane w tej przeglądarce.
- **Powitanie**: gdy ktoś pierwszy raz się loguje, w dowolnej przeglądarce, i na końcu konfiguracji, FinStats pyta o język, formaty daty i godziny oraz układ, z obrazkiem każdego. Pomiń zostawia brytyjski angielski, 31/01/2026, 13:05 i FinStats.

### Changed
- Logo wewnątrz strony to sam znak obok nazwy, złożonej krojem Inter bez względu na czcionkę wybraną w presecie; tam, gdzie stoi samodzielnie (favicon), zachowuje zaokrąglony kafelek.
- Odtworzenie transkodowane programowo jest zapisywane jako takie, jeśli Jellyfin to podaje; wcześniej wyglądało tak samo jak odtworzenie z serwera, który tego nie podaje. Strony nadal nazywają oba przypadki „Programowe”.
- Aktualizacja wykonuje teraz, jednorazowo i od najstarszej wersji, to, co dana wersja musi zrobić z bazą danych zapisaną przez starszy FinStats, obok swoich migracji. W wersji 2.3.0 jest to odczyt szczegółów serwera przy pierwszym uruchomieniu, więc Wydajność jest dostępna od razu.

### Performance
- **Otwarta strona o nic nie pyta**: Pulpit wysyłał 30 żądań na minutę, by pozostać aktualnym, a pobieranie na stronie „W drodze” 18; z kanałem na żywo oba nie wysyłają żadnych. Bez względu na liczbę otwartych kart FinStats nie pyta Jellyfin, Sonarr ani Radarr o nic więcej niż wcześniej, a ukryta karta nadal pozwala kolejce pobierania zwolnić.
- Strona osoby odczytuje tylko odtworzenia tej osoby: jej sumy i ostatnie odtworzenie były wyliczane ze wszystkich odtworzeń na serwerze, a dopiero potem zawężane do niej, co przy milionie odtworzeń trwało około czterech sekund.
- Strona biblioteki odczytuje tylko tytuły i odtworzenia tej biblioteki, zamiast liczyć wszystkie biblioteki i zostawiać jedną: przy milionie odtworzeń to jedna trzecia czasu.
- Gdy zegar Jellyfin spieszył się względem zegara FinStats, wyzwalacz *Po skanowaniu biblioteki przez Jellyfin* odczytywał całą bibliotekę ponownie co minutę, dopóki różnica nie minęła; teraz każde skanowanie jest obsługiwane raz.
- Plik GeoIP, którego nie dało się otworzyć, był co kwadrans kopiowany od nowa (sto megabajtów i więcej), a każdy adres był ponownie lokalizowany; teraz kolejna próba następuje dopiero po zmianie pliku.
- Stan biblioteki otwiera się natychmiast, gdy wskaźnik zatrzymał się wcześniej na łączu: to, co pobrano z wyprzedzeniem, trafiało tam, gdzie strona nigdy nie zaglądała.

### Stability
- Aktualizacja, która nie powiodła się po wykonaniu migracji i była od razu restartowana przez Docker, przy każdym starcie tworzyła nową kopię częściowo zaktualizowanej bazy, a trzy starty usuwały jedyną kopię, którą mogła otworzyć poprzednia wersja. Teraz kopia sprzed aktualizacji jest zachowywana.
- Tytuł usunięty w Jellyfin podczas odczytu biblioteki (plik podmieniony przez Radarr) mógł sprawić, że odczyt pominął inny, wciąż istniejący tytuł i oznaczył go jako usunięty, a jego odtworzenia mogły potem zostać przeniesione na inną jego kopię. To, czego odczyt nie zobaczył, jest teraz najpierw sprawdzane po identyfikatorze.
- Przywrócenie kopii zapasowej wykonanej przed zmianą nazwy pliku lub jego wymianą na lepszy przywracało po raz drugi każde odtworzenie przeniesione później na nowy plik, podwajając je na stałe.
- Prośba w odpowiedzi Seerr, której FinStats nie umiał odczytać (pole przemianowane w aktualizacji Seerr), była oznaczana jako usunięta przez codzienny pełny odczyt.
- Zadanie, które natrafiło na błąd, na zawsze pozostawało „uruchomione” i żaden wyzwalacz nie uruchamiał go ponownie aż do restartu FinStats; teraz kończy się niepowodzeniem.
- Uszkodzony plik zip z Tautulli przerywał przesyłanie, zanim plik został usunięty, a zawiera on tokeny wszystkich użytkowników Plex; zostawał tak aż do sześciogodzinnego sprzątania.
- Import lub przywracanie przerwane w trakcie przesyłania pliku zostawiały ten plik w folderze danych na zawsze, razem z adresami; teraz jest usuwany przy następnym starcie.
- Obraz mógł zostać wysłany w połowie zapisany, gdy właśnie trafiał do pamięci podręcznej, a przeglądarka zachowywała go w takim stanie.
- Unieważnione klucze API były przechowywane na zawsze, bez względu na to, ile ich utworzono i unieważniono.
- Odczyt czyichś znaczników obejrzanych i ulubionych, który wrócił pusty lub nieczytelny, czyścił je wszystkie do następnego odczytu; teraz jest odrzucany, tak jak pusty odczyt biblioteki.
- Odcinek z numerem końcowym, jakiego żaden odcinek nie może mieć (wpisana tam data), mógł bez końca zajmować Stan biblioteki po odczycie biblioteki.

### Fixed
- Pozycja listy zmian z kodem wewnątrz pogrubienia (w wersjach 2.0.2 i 2.0.3) pokazywała grawisy zamiast sformatowanego kodu.
- Wykres liczebności mógł opisywać oś 0, 3, 5, 8, 10, a nawet 0, 0, 1, 1, 1: kroki co 2,5 zapisane po zaokrągleniu. Liczebności rosną teraz o całe kroki.
- Klucz kalendarza utworzony przez osobę, która nie jest menedżerem, dostawał link z adresem z przeglądarki zamiast publicznego adresu FinStats, więc działał tylko z wewnątrz sieci; strona pytała też o ustawienia, do których odmawiano jej dostępu.
- Ostatnie odtworzenie osoby mogło być o kilka sekund za krótkie, by gdziekolwiek indziej liczyć się jako odtworzenie, i wskazywać tytuł, którego nie pokazuje jej Historia.
- Nieudane przywrócenie z listy Ostatnio usunięte nic nie mówiło, a jego przycisk dalej się kręcił; to samo dotyczyło nieudanego *Przywróć* w Stanie biblioteki oraz nieudanego *Otwórz ponownie* lub *Rozwiąż wszystkie* w Bezpieczeństwie.
- Kanał kalendarza nazywa się w aplikacji kalendarza FinStats, a nie finstats.
- Kolejka pobierania pokazywała, kto poprosił o każde pobranie, każdemu, kto mógł ją oglądać; cudza prośba wymaga teraz uprawnienia *Widzi aktywność wszystkich*, tak jak na stronie „W drodze”.
- Odebranie uprawnienia *Otrzymuje powiadomienia* nie zatrzymywało tego, co trafiało do miejsc docelowych tej osoby, a ona nie mogła ich już wyłączyć.
- Odtworzenie wstrzymane i wznowione w obrębie okna scalania było raz zgłaszane jako zakończone, a jego wznowienie i faktyczny koniec nigdy.
- Odtworzenia utworu mogły zostać przeniesione na utwór innego wykonawcy o tej samej nazwie („Intro”), gdy jego album zniknął z biblioteki.
- Miejsce docelowe powiadomień zapisane jako adres IPv4 wewnątrz IPv6 (NAT64, 6to4) uchodziło za publiczne, choć prowadziło do sieci lokalnej.
- Połączenie z Sonarr lub Radarr przestawione na inny serwer jeszcze przez tydzień pokazywało plakaty starego serwera.
- Alerty w Bezpieczeństwie twierdziły, że nic nie wymaga uwagi, gdy wybrano co najmniej dwie osoby, a strona opróżniona przez rozwiązanie ostatniego alertu nie miała drogi powrotu.
- Każdy klucz API, który jeszcze nie wygasł, twierdził, że wygasł „przed chwilą”.
- Okno dialogowe zostawało nad stroną po cofnięciu się, a strony nie dało się przewijać, dopóki go nie zamknięto.
- Dziennik audytu pokazywał osiem rodzajów wpisów (przywrócone odtworzenie lub kopia zapasowa, opróżnienie kosza, profile publiczne, zmiana wyglądu) przez ich identyfikatory, bez niczego w kolumnie Szczegóły.
- Dodanie tytułu do listy do obejrzenia z menu kontekstowego kończyło się bez słowa niepowodzeniem, gdy serwer odmówił.
- Karta *Oglądane razem* na Pulpicie traktowała kilka osób wybranych naraz jak jedną.
- Podsumowanie zapisywało godziny i dni poniżej dziesięciu z przecinkiem dziesiętnym przeglądarki obok własnych przecinków oddzielających tysiące.
- Wydajność podawała, że dysk zapełni się "w tempie ostatnich 0 miesięcy".
- Na stronie Bezpieczeństwo adresy bez miejsca liczyły odtworzenia wszystkich z całego czasu, niezależnie od osoby i okresu, do których zawężono stronę.
- Ktoś, kto *Zarządza FinStats*, ale nie *Widzi aktywność wszystkich*, mógł przenieść do kosza czyjekolwiek odtworzenie.
- Karta z Podsumowania lub profilu publicznego mogła przyjść z pamięci podręcznej przeglądarki w wcześniej wybranym języku.
- Pobierania wysyłane przez kanał na żywo były po brytyjsku angielsku, niezależnie od języka strony.
- Logowanie, które zmieniało to, co ktoś może widzieć, docierało do innych otwartych kart tej osoby dopiero przy następnym sprawdzeniu po pół minucie.
- Karta, która oddała połączenie na żywo nowszej, przy następnej stronie brała je z powrotem, więc karty jednej osoby bez końca zamykały sobie nawzajem połączenia.
- Tabele sortowały rozmiary od 1000 do 1023 jednostek jak tekst w językach grupujących cyfry spacją, a tak samo procenty po turecku.
- Tytuł z symbolem zastępczym w nazwie, jak "Big {device} Bunny", był wypełniany w komunikatach serwera.
- Tabele sortowały czasy trwania i liczby z całości (5 z 26) jak tekst we wszystkich językach poza angielskim i nordyckimi.
- Karta wciąż otwarta w jednym języku słyszała serwer w innym, gdy tylko inna karta go wybrała.
- Słowo obok liczby miało złą formę po polsku, rosyjsku i ukraińsku ("5 wieczory", "2 потоков").
- Wydajność liczyła pierwsze zapełnienie biblioteki jako wzrost i dzieliła to, co dodała każda biblioteka, przez sześć miesięcy, niezależnie od okresu pomiaru; plik, który zajął miejsce innego (ulepszenie z Radarr lub Sonarr albo plik, któremu Jellyfin nadał nowy identyfikator), liczył się jako całkiem nowy, choć nowa była tylko różnica.
- Osoby oglądające razem przez SyncPlay nie były grupą, gdy jedna dołączyła kilka minut po drugiej; odtworzenia, które kończą się w tej samej chwili w tym samym miejscu, są teraz razem, a historia zostaje raz pogrupowana od nowa.
- Ktoś bez uprawnienia *Widzi aktywność wszystkich* mógł wyczytać z *Nigdy nieoglądane* biblioteki, że inna osoba odtworzyła tytuł; teraz liczy się tylko jego własne oglądanie.

## [2.2.0] - 2026-10-01 to 2026-10-07

Stan biblioteki i FinUI

### Added
- **Własne menu na komputerze**: sekcja Ustawienia → Wygląd oferuje osiem stylów: pasek boczny jak dotąd, menu pogrupowane według przeznaczenia stron, szynę ikon, która pod wskaźnikiem rozwija się w pasek boczny, pasek boczny zaczynający się od wyszukiwania, dok, pasek poleceń, strony przypięte na górze oraz kolorowe kafelki. Paski boczne mogą stać po prawej. Dok chowa się podczas przewijania w dół i wraca przy przewijaniu w górę albo po wskazaniu dolnej krawędzi okna; przy nim pasek stanu przenosi się na górę.
- **Własne menu na telefonie**: pasek kart z pozycją Więcej (nowe ustawienie domyślne), arkusz otwierający się do połowy, cały ekran, strony rozłożone wachlarzem wokół kciuka albo pasek adresu, który rozrasta się w menu. Każde z nich przeszukuje swoje strony, a pozostałe słowa przekazuje wyszukiwaniu.
- Oba wybory zapamiętuje przeglądarka, w której ich dokonano, więc telefon i komputer mają każdy swój; oba menu to komponenty FinUI, `desktop-nav` i `mobile-nav`.
- **Menu kontekstowe**: kliknij prawym przyciskiem tytuł, osobę lub bibliotekę w dowolnym miejscu FinStats (plakat na Pulpicie, wiersz listy, nazwę w tabeli, film w sekcji Wkrótce, którego nie ma jeszcze w bibliotece), przytrzymaj go palcem albo naciśnij klawisz menu, a menu wyrośnie z miejsca naciśnięcia: otwórz, otwórz w nowej karcie, dodaj film lub serial do listy do obejrzenia, otwórz w Jellyfin albo w TMDB i TVDB, kopiuj link. Shift z prawym kliknięciem nadal otwiera menu przeglądarki. To komponent FinUI `context-menu`.
- **Wyszukiwanie bez okna dialogowego**: wyszukiwanie wyrasta z menu (pasek boczny poszerza się w nie, dok unosi się w nie, nazwa w pasku poleceń zamienia się w pole wyszukiwania, menu telefonu je mieści) albo, jeśli tak wybrano w Ustawieniach → Wygląd, zajmuje miejsce strony, a menu zostaje. Ctrl+Spacja otwiera je jak dotąd; okno nad wszystkim zniknęło. To komponent FinUI `search`.
- **Wszystko w bibliotece**: karta Ostatnio dodane w bibliotece oferuje *Cała zawartość*, czyli każdy film, serial lub album, jaki w niej jest, jako plakaty, według nazwy, roku, daty dodania lub rozmiaru, zawężane w trakcie pisania. Kolejne wczytują się podczas przewijania.
- **Serwer → Stan biblioteki**: FinStats porównuje każdy plik z sąsiednimi i wypisuje to, co odstaje, każdą pozycję z liczbami, które za nią stoją. Tylko zgłasza; nic w Jellyfin nie jest zmieniane, usuwane ani skanowane ponownie.
- **Luki**: brakujące odcinki między dwoma plikami sezonu („Sezon 2: brakuje odcinków 4 i 5 między 3 a 6”). Plik zawierający dwa odcinki liczy się jako oba; odcinki specjalne i koniec sezonu są pomijane.
- **Różniące się sezony**, czyli odmienne od reszty serialu rozdzielczością, HDR lub kodekiem, oraz **nietypowe odcinki**: jeden plik niepodobny do reszty sezonu, na przykład odcinek SD w sezonie 1080p. Rozdzielczość liczy się według szerokości lub wysokości, więc przycięty film 1920×800 to 1080p.
- **Kopie i wersje**: ten sam film lub odcinek dwa razy, według identyfikatorów dostawców, którymi już kierują się lista do obejrzenia i „W drodze”. Dwie kopie w tej samej rozdzielczości pokazują podwójnie zajęte miejsce; kopia 4K obok kopii 1080p to dwie wersje i nigdy nie jest liczona jako zmarnowane miejsce.
- **Słabe pliki**: przepływność daleko poniżej tego, czego zwykle wymaga dana rozdzielczość i kodek („4K HEVC przy 1,1 Mb/s”); słabe odcinki jednego sezonu to jeden wynik.
- **Urwane dubbingi**: język ścieżki dźwiękowej, który niektóre sezony mają w każdym odcinku, a inne w żadnym.
- **Niezidentyfikowane**: filmy i seriale, które Jellyfin dopasował bez identyfikatora TMDB, TVDB ani IMDb.
- **Odrzuć** wynik, który celowo jest taki, jaki jest, i dodaj notatkę. Wraca sam, gdy zmienią się jego pliki. Odrzucać mogą osoby, które mogą zarządzać FinStats i widzą serwer; odrzucenia trafiają do dziennika audytu i są częścią kopii zapasowej.
- Strona biblioteki mówi, ile jest w niej rzeczy do sprawdzenia, i do nich prowadzi.
- **FinUI**, komponenty, z których zbudowany jest FinStats, to osobny projekt (`github.com/finstats/finui`): każdy przycisk, karta i kontrolka w każdym stanie i w obu motywach pod adresem `finui.finstats.no`, a do tego **FinUI create**, w którym można na bieżąco zmieniać kolory, zaokrąglenia, gęstość i nie tylko, a potem zabrać wynik do siebie jednym poleceniem. Prowadzi tam **Ustawienia → System**.
- **Ustawienia → Wygląd**: każdy wybiera, jak FinStats ma dla niego wyglądać, kodem presetu FinUI: kolor bazowy, akcent, kolory wykresów, kontrast, zaokrąglenie, gęstość, obramowania, karty, przyciski, pola, tabele, wyróżnienie, ruch, ikony, menu, tło pod wszystkim, obwódka fokusu, sposób składania nagłówków oraz czcionki tekstu, nagłówków i liczb. Twój wybór należy tylko do ciebie; strona pokazuje, co oznacza kod, i prowadzi do FinUI create, gdzie można go zmienić.
- **Style**: jedenaście kompletnych wyglądów na start, wybieranych jednym kliknięciem w FinUI create, a ich kod przyjmuje sekcja Ustawienia → Wygląd. To Washi (FinStats w wersji domyślnej), Studio, Noir, Matinee, Console, Pastille, Ledger, Gazette, Arcade, Canopy i Harbour. Sam FinStats ma jeden wygląd i nie oferuje listy stylów.
- **Ikony, które się ruszają**: każda ikona w łączu, przycisku lub karcie wykonuje raz swój ruch po wskazaniu (odświeżanie się obraca, strzałka pobierania opada do tacki, kosz unosi pokrywę), a naciśnięte odświeżanie kręci się, dopóki nie nadejdzie odpowiedź. Nieruchome przy ograniczonym ruchu i przy ustawieniu Ruch: Wyłączony.
- **Ruch dzięki FinMotion**: komponenty FinUI teraz się ruszają: gałka przełącznika zostaje przerzucona, podkreślenie karty przesuwa się do sąsiedniej, odrzucone pole kręci głową, kopiowanie zostaje odhaczone jednym pociągnięciem, powiadomienia układają się jak karty w dłoni i rozkładają wachlarzem po wskazaniu, arkusz na telefonie ugina się jak guma, gdy się go ciągnie, wiersze posortowanej tabeli przesuwają się na swoje miejsca, zmieniająca się liczba przewija się jak licznik kilometrów, a wykresy rysują się same, gdy widać je po raz pierwszy. Wszystko to podąża za wyborem Ruch w Ustawieniach → Wygląd i zamiera przy ograniczonym ruchu. FinMotion to osobny projekt (`github.com/finstats/finmotion`), nałożony na FinUI.
- **Więcej ikon w FinUI**: teraz 239 zamiast 71, każda z własnym ruchem (głośniej i ciszej, słuchawki zwykłe i wyciszone, ołówek, który pisze, dzwonek, który dzwoni, telefon, który wibruje), a każdą można pobrać z galerii FinUI jako SVG, nieruchomą lub animowaną.
- **Czcionki**: trzydzieści cztery rodziny na wolnych licencjach do wyboru (Geist, Roboto, Manrope, IBM Plex Sans, Montserrat, Lora, Playfair Display, Fira Code i inne), dołączone tak jak Inter, nigdy nie pobierane z zewnątrz i wczytywane tylko po wybraniu.
- **Więcej baz i akcentów**: Nord, Solarized, Gruvbox, Catppuccin, Rosé Pine, Tokyo Night, Dracula, Everforest i Ocean; szesnaście nowych akcentów, z czego dziesięć ma jeden kolor za dnia, a inny nocą. Każda baza ma teraz za dnia własny papier: w jasnym motywie odcienie szarości wyglądały podobnie.
- **Usunięcie można cofnąć przez 30 dni**: usunięte odtworzenie lub kopia zapasowa czeka w koszu, a **Cofnij** w powiadomieniu, które się wtedy pojawia, przywraca je. Aktywność i Ustawienia → Kopie zapasowe pokazują ostatnio usunięte elementy wraz z terminem ich ostatecznego usunięcia, gdy tylko jakieś są. Odtworzenie w koszu nie wlicza się do żadnej sumy, wykresu, podsumowania ani profilu, a import czy przywracanie nie dodaje go ponownie jako nowego; kto oglądał je z kimś, jest liczony jako oglądający sam, dopóki odtworzenie nie wróci.

### Changed
- Dark Reader i podobne rozszerzenia zostawiają strony FinStats w spokoju (`darkreader-lock`): FinStats sam rysuje oba motywy, a przemalowana strona z nimi walczyła.
- Na telefonie menu nie wysuwa się już z lewej: pasek kart stoi na dole, chyba że wybrano inne menu.
- Po tej aktualizacji FinStats raz odczyta bibliotekę, aby dowiedzieć się, gdzie kończą się pliki zawierające kilka odcinków.
- Na stronie tytułu przycisk **Dodaj do listy do obejrzenia** ma tę samą wysokość co **Otwórz w Jellyfin** obok; był o rozmiar mniejszy.
- Filtry zakresu i osób zostają u góry ekranu podczas przewijania strony, więc z dołu długiej strony da się przełączyć z „7 dni” na „Całość”; tak samo filtry próśb i nadchodzących premier na stronie „W drodze”.
- Przycisk przewijania do góry omija menu, które zajmuje dół okna: stoi nisko w rogu obok doku, tuż nad paskiem kart, przyciskiem pod kciukiem lub paskiem adresu na telefonie, a w pozostałych przypadkach nad paskiem stanu. To komponent FinUI `to-top`.
- Usunięcie odtworzenia lub kopii zapasowej przenosi je do kosza zamiast od razu kasować (`PUT` z `{"deleted": true}`; `DELETE` zniknęło z API). Kopie ponad liczbę zachowywanych są nadal usuwane na stałe.
- **Lista zmian** to jeden arkusz zamiast stosu kart: każda pozycja jest osobnym wierszem z rodzajem (Dodano, Zmieniono, Naprawiono i pozostałe) i ikoną tego rodzaju w kolumnie przed nią, pod nagłówkiem każdej serii. Lista wersji zostaje po prawej.

### Performance
- Otwarcie FinStats pobiera tylko to, co się zmieniło: arkusz stylów, skrypt lub czcionka, które przeglądarka już ma, są potwierdzane kilkoma bajtami zamiast wysyłane ponownie. Wcześniej każda wizyta pobierała w całości wszystkie arkusze stylów i moduły FinStats, choć każdy z nich miał znacznik, który pozwalał je odróżnić.

### Stability
- Przywracanie kopii zapasowej odczytuje ustawienia tylko z wiersza ustawień: plik z nowszego FinStats, który przechowuje tam więcej, tracił ustawienia bez słowa.
- Otwarta karta nie rośnie już z każdą odwiedzoną stroną: każda opuszczona strona zostawała w pamięci za tą na ekranie (około 2600 elementów na wizytę), trzymana przez listę elementów poruszanych przez FinMotion i przez obsługę przewijania paska filtrów, aż do zamknięcia karty.
- Błędnie wpisane polecenie (`finstats bakcup`) albo `restore` bez pliku nie tworzy już ani nie aktualizuje bazy danych, zanim powie, że nie wie, co zrobić.

### Fixed
- Mały wykres na kafelku nie wychodzi już poza kafelek, gdy liczba obok jest szeroka („56 d. 23 godz.”) albo gdy cztery kafelki dzielą węższy rząd: wykres ustępuje, a liczba zachowuje swoją szerokość.
- W jasnym motywie główny przycisk robił się fioletowy pod wskaźnikiem; teraz zachowuje kolor pieczęci.
- W Ustawieniach przełącznik stoi na wysokości nazwy i opisu ustawienia; wcześniej wisiał u góry wiersza.
- Na liście rankingowej bez niczego pod nazwą, takiej jak Najaktywniejsi użytkownicy na Pulpicie, nazwa stała u góry wiersza obok wyśrodkowanego obrazka; teraz jest wyśrodkowana.
- Rząd kafelków statystyk przy szerokości między telefonem a komputerem (cztery kafelki po trzy w rzędzie) zostawiał ostatni sam w osobnym rzędzie; teraz cztery kafelki układają się po cztery, dwa lub jeden w rzędzie, a trzy po trzy lub jeden.
- Odczyt serwera liczył urządzenie, na którym nikt się nie zalogował i którego FinStats nie przechowuje: zadanie mówiło o trzech urządzeniach, gdy strona Serwer pokazywała dwa.
- Jedna sekunda, jedna minuta, jedna godzina: angielskie alerty pisały „1 seconds apart” i „in 1 minutes”, a zadanie Jellyfin z interwałem jednej minuty uruchamiało się „every 1 minutes”.
- `finstats --help` wymienia każdą zmienną, którą odczytuje FinStats; brakowało `FINSTATS_PUBLIC_IP_URL`, `FINSTATS_ALLOW_LIBRARY_SHRINK` i `FINSTATS_SKIP_PREUPDATE_BACKUP`.
- Kreator konfiguracji mówił o czymś, co nie jest adresem, że „nie udało się połączyć”, w zdaniu, które zlewało się z następnym; teraz mówi, że to nie jest adres.
- W Ustawieniach → Kopie zapasowe pytanie zadane podczas zapisywania kopii znikało, gdy kopia była gotowa.
- Przy menu w stylu rozwijanej szyny zamknięcie wyszukiwania pokazywało jednocześnie otwarte wyszukiwanie i zamkniętą szynę, a potem zostawiało szynę otwartą, dopóki wskaźnik nie najechał na nią i nie zjechał. Esc oddaje teraz fokus klawiatury tam, gdzie był w chwili otwarcia wyszukiwania, a fokus po kliknięciu porzuca.
- Na telefonie zamknięcie wyszukiwania zostawiało stronę przyciemnioną, dopóki wyszukiwanie nie zniknęło, a potem w jednej klatce przeskakiwało do pełnej jasności; przyciemnienie znika teraz razem z nim.
- Kroki importu z Tautulli mówiły, że kopia zapasowa pobierze się po kliknięciu **Backup Database**. Tautulli zapisuje ją na własnej maszynie: kroki mówią teraz gdzie (w folderze `backups`, w Dockerze w `/config`), który plik wziąć i że trzeba go przekopiować.
- Wyszukiwanie zamknięte, zanim skończyło się otwierać, odbierało rozwijanej szynie animację: od tej chwili otwierała się i zamykała natychmiast.
- Rozwijana szyna pozostawała otwarta po wybraniu z niej strony, dopóki nie kliknięto gdzie indziej, a w niektórych przeglądarkach lub po powiększeniu rysowała pasek przewijania pod ikonami. Teraz otwiera się tylko dla wskaźnika i klawiatury i nie rysuje paska przewijania, gdy jest zamknięta.
- Na stronie Bezpieczeństwo adres, który baza danych umieszcza w danym kraju, ale nie na mapie, liczy się do tego kraju, a nie do adresów bez znanego miejsca.

## [2.1.2] - 2026-10-01

### Added
- **Import z Tautulli**: przenieś swoją historię z Plex. Prześlij kopię zapasową bazy danych Tautulli (plik `.db` albo zawierający go `.zip`) w **Ustawieniach → Import**; jej odtworzenia mają własną pozycję w filtrze *Zarejestrowane przez* na stronie Aktywność.
- **Tablica połączeń, która mówi, kto jest kim**: użytkownicy Plex po jednej stronie, użytkownicy Jellyfin po drugiej. Przeciągnij przewód od każdego użytkownika Plex do osoby, którą jest teraz, albo kliknij jednego, a potem drugiego, albo użyj klawiatury. Kto zostanie bez przewodu, nie jest importowany, a dwa konta Plex mogą trafić do jednej osoby.
- Filmy i odcinki są dopasowywane do biblioteki po nazwie (film po tytule i roku, odcinek po serialu i numerze), a tytuł, którego jeszcze nie ma na serwerze, jest zachowywany i dołączany, gdy się pojawi. Wznowione oglądanie to jedno odtworzenie; muzyka nie jest importowana.
- Kopia zapasowa zawiera tokeny dostępu Plex: FinStats nigdy ich nie odczytuje i usuwa plik, gdy tylko zaimportujesz dane albo zaczniesz od nowa.
- **Niepowiązane multimedia** w **Ustawieniach → Dane**: każdy tytuł z Tautulli, Jellystat lub Streamystats, który nie pasuje do niczego w bibliotece, z informacją, skąd pochodzi i gdzie najpewniej już jest. **Wskaż** łączy go na podstawie najlepszych propozycji lub wyszukiwania (film można wskazać jako odcinek specjalny serialu), a FinStats zapamiętuje wybór, więc ta sama historia zaimportowana ponownie połączy się sama. Wybory są częścią kopii zapasowej.

### Performance
- **Podsumowanie otwiera się od trzech do pięciu razy szybciej**: najczęściej oglądanych aktorów i reżyserów roku wyliczano, przeglądając cały rok od nowa dla każdej występującej w nim osoby. Na prawdziwej historii rok skrócił się z 0,29 s do 0,09 s, a ostatnie 12 miesięcy z 0,53 s do 0,12 s.

### Fixed
- **Zaimportowana historia znajduje tytuły występujące pod inną nazwą**: odtworzenie pasuje też do tytułu w języku oryginału, więc serial, który Plex lub Tautulli znały jako 오징어 게임, łączy się ze Squid Game, a *Im Westen nichts Neues* z *All Quiet on the Western Front*. Oryginalne tytuły z biblioteki są odczytywane przy jej następnym skanowaniu.
- Nazwy są porównywane tak, jak porównuje je wyszukiwanie (wielkość liter, znaki diakrytyczne, interpunkcja, rok wpisany w nazwę, początkowe „The”), a rok wydania może różnić się o jeden, więc „Star Wars: Episode I - The Phantom Menace” znajduje „… – The Phantom Menace”, a Kingsman z 2014 roku znajduje ten z 2015. Tytuł znany pod zupełnie inną nazwą nadal nigdy nie jest zgadywany.
- Nieudany import ze Streamystats lub nieudane przywracanie były ogłaszane w powiadomieniach pod nazwą wewnętrzną.
- Przycisk *Udostępnij rok jako karty* w podsumowaniu i pozycja widza w rankingu obok niego nie miały między sobą odstępu.

## [2.1.1] - 2026-10-01

### Changed
- **Lista do obejrzenia jest dostępna wszędzie tam, gdzie jest tytuł**: na kartach Wkrótce na Pulpicie, tak jak w zakładce „W drodze” → Nadchodzące, oraz na stronie odcinka lub sezonu, gdzie dodaje do listy cały serial.
- Na plakacie na Pulpicie przycisk listy do obejrzenia pojawia się, gdy wskaźnik jest nad kartą albo gdy dotrze się do niej klawiaturą, zamiast zasłaniać róg każdego plakatu; na ekranie dotykowym jest zawsze widoczny.

### Stability
- Zapisanie połączenia z Sonarr, Radarr lub Seerr, gdy ktoś miał otwarte **Ustawienia → Połączenia wychodzące**, mogło sprawić, że obie operacje na zawsze czekały na siebie nawzajem.
- **Ustawienia, których nie da się odczytać, zatrzymują FinStats przy starcie** ze wskazaniem winnego ustawienia, zamiast po cichu przywracać wszystkie ustawienia do domyślnych.
- **Dwie jednoczesne zmiany ustawień** (na przykład na stronie ustawień i w harmonogramie zadania) nie znoszą się już nawzajem.
- Wstrzymanie, przewinięcie lub zmiana ścieżki zauważone, gdy baza danych była zajęta, są zapisywane w następnym przebiegu, zamiast przepadać.
- Odczyt biblioteki nie kończy się już niepowodzeniem na liście do obejrzenia z kilkoma pozycjami, które okazują się jednym tytułem.

### Fixed
- **Podmieniony plik nie jest „już na serwerze”**: wymiana pliku filmu z czyjejś listy do obejrzenia na lepszy nie powiadamia już tej osoby, że film właśnie się pojawił.
- Jeden nieudany odczyt wtyczek, zaplanowanych zadań lub pamięci masowej Jellyfin nie opróżnia już tej listy na stronie Serwer aż do następnego odczytu.
- **Odtworzenie ze Streamystats kończy się na każdej stronie tam, gdzie naprawdę się skończyło**: krzywa porzuceń tytułu i „ukończone” przy odcinku kierowały się zamiast tego jego długością i liczyły je jako oszacowane.

## [2.1.0] - 2026-09-29 to 2026-09-30

Lista do obejrzenia

### Added
- **Własna lista do obejrzenia**: dodaj do niej film lub serial z jego strony, z Nadchodzących, z Ostatnio dodanych albo z wyszukiwania (Ctrl+Enter), a całą listę znajdziesz w zakładce **Do obejrzenia** na swoim profilu.
- **Każda pozycja mówi, jak wygląda teraz**: na serwerze, „5 z 26 odcinków”, poproszono, „Wkrótce, piątek”, nie ma na serwerze, usunięte z biblioteki albo obejrzane. Jest to wyliczane przy każdym spojrzeniu, z tym samym rozumieniem „obejrzanego” co postęp seriali. Obejrzane tytuły zostają w osobnej grupie, dopóki ich nie usuniesz.
- **Tytuł, którego jeszcze nie ma na serwerze**, można dodać do listy prosto z tego, na co czeka Sonarr lub Radarr. Gdy się pojawi, zostanie sam połączony z tytułem, a plik, któremu zmieniono nazwę, zabiera swoją pozycję ze sobą.
- **„Z listy do obejrzenia: już dostępne”**: powiadomienie do twoich własnych miejsc docelowych, gdy pojawi się coś, na co czekasz. Wyłączone, dopóki go nie zaznaczysz.
- **Nikt inny nie widzi twojej listy do obejrzenia**, nawet administrator, a jej powiadomienie trafia tylko do twoich miejsc docelowych. Nic nie jest zapisywane w Jellyfin, Seerr, Sonarr ani Radarr.
- Listy do obejrzenia są częścią kopii zapasowej, a przywracanie scala je, niczego nie dublując.

### Changed
- Pozycje `/api/upcoming` zawierają identyfikatory TVDB i TMDB swojego tytułu, a `/api/auth/me` mówi, czy serwer prowadzi listy do obejrzenia.

## [2.0.4] - 2026-09-29

### Added
- **Harmonogram zadań na sposób Jellyfin**: każde zadanie, które FinStats wykonuje sam, ma wyzwalacze (codziennie o danej godzinie, co tydzień w danym dniu, w stałych odstępach, przy uruchomieniu albo po skanowaniu biblioteki przez Jellyfin), każdy z opcjonalnym limitem czasu. Kliknij zadanie, by zobaczyć i zmienić jego harmonogram; *Dodaj wyzwalacz* otwiera to samo okno, które ma Jellyfin.
- **Zadania to osobna sekcja** w Ustawieniach, z ostatnim uruchomieniem każdego zadania opisanym słowami („Ostatnio uruchomione 38 minut temu, trwało 6 minut”) i przyciskiem Uruchom; wcześniej zadania były na górze sekcji System.
- **Zmiany metadanych**: co godzinę FinStats odczytuje, co Jellyfin zapisał od ostatniego sprawdzenia (nazwy, opisy, gatunki, oceny, obsadę i twórców, szczegóły plików i obrazy), więc zmiana wprowadzona w Jellyfin pojawia się bez czekania na skanowanie biblioteki.
- Kopię zapasową i pobieranie bazy geolokalizacji też można uruchomić z Zadań; Esc wraca z harmonogramu zadania do listy.

### Changed
- **Motyw to przełącznik z trzema położeniami**, Jasny · Systemowy · Ciemny, w miejscu przycisku, który przełączał je po kolei: kliknij jedną trzecią przełącznika, przeciągnij gałkę albo użyj klawiszy strzałek. Gałka nosi ikonę wybranej opcji, która przy zmianie wsuwa się na miejsce.
- **Wylogowanie jest w Ustawieniach → Konto**, gdzie widać, kto jest tu zalogowany; pasek boczny zachowuje przełącznik motywu.
- **Dawne ustawienia harmonogramu są teraz wyzwalaczami**: *Podążaj za skanowaniem biblioteki Jellyfin* z zapasowym interwałem, *Twórz kopię zapasową co N dni* i przełącznik GeoIP *Aktualizuj bazę danych* stały się odpowiednimi wyzwalaczami swoich zadań, z dotychczasowymi wartościami. Zbieranie danych, Kopie zapasowe i Bezpieczeństwo prowadzą teraz do harmonogramu.
- Start nie uruchamia już wszystkich zadań od nowa: zapamiętywane jest, kiedy każde z nich ostatnio się zakończyło.
- Zadanie Jellyfin, które nie ma jeszcze oszacowania, pokazuje „zostało około …”, z kropkami krążącymi tam, gdzie pojawi się czas, zamiast „ETA…”.

### Performance
- **Mapa w Bezpieczeństwie już się nie zacina**: przeciąganie i powiększanie przesuwają już narysowaną mapę i rysują ją raz, gdy gest się zakończy, a pulsowanie bieżącego odtworzenia nie zmusza już całej mapy do przerysowywania 120 razy na sekundę, gdy nikt jej nie dotyka. Przeciąganie na wolnym komputerze przyspieszyło z 34 do 60 klatek na sekundę.

### Fixed
- **Plakat lub portret podmieniony w Jellyfin pojawia się w FinStats**: obrazy były przechowywane w pamięci podręcznej przez tydzień na serwerze i przez kolejny tydzień w przeglądarce, bez względu na to, co od tego czasu miał Jellyfin. Teraz są przechowywane pod znacznikiem obrazu z Jellyfin, więc nowy obraz to nowy plik, a przeglądarka przy każdym wyświetleniu sprawdza, czy jej kopia jest aktualna. Podmiana portretu zapisuje ponownie tylko osobę, więc Zmiany metadanych odczytują też osoby.
- **Serwer → Zadania pokazuje oszacowanie od chwili otwarcia strony**: pozostały czas był mierzony tylko z odczytów, które strona wykonała sama, będąc otwarta, więc każda wizyta zaczynała się od „ETA…”, dopóki strona nie zobaczyła, jak rośnie procent. FinStats sam mierzy teraz czas trwającego zadania Jellyfin, co 10 sekund, dopóki ono trwa, a strona dostaje gotowe oszacowanie. Zadanie, które Jellyfin przez cały czas raportuje jako 0% (pierwsza faza skanowania biblioteki), zachowuje kropki, dopóki procent się nie ruszy: nic nie jest zgadywane.
- **Dymki na mapie pojawiają się przy kropce**: w przeglądarce Firefox dymek kropki bieżącego odtworzenia mógł wylądować daleko od niej, a tam, gdzie kropki się nakładają (ktoś odtwarza w domu), to, który dymek się pojawiał, zależało od jednego piksela. Teraz jeden dymek wymienia wszystko, co jest pod wskaźnikiem.
- **Pasek stanu ma jeden kształt podświetlenia**: każde łącze (strumienie, Repozytorium, wersja) podświetla się jako ten sam mały prostokąt, a kropki między elementami stoją na środku odstępów. Najechanie na wersję podświetlało też kropkę i odstęp przed nią.
- **Ścieżka pliku zajmuje całą szerokość karty**: na karcie Plik na stronie tytułu oraz przy folderach Jellyfin na stronie Serwer ścieżka była wciśnięta w jedną wąską komórkę i ciągnęła się przez dziewięć wierszy obok pustego miejsca.
- Zanikające okno dialogowe nie przechwytuje już kliknięcia w stronę pod nim.

## [2.0.3] - 2026-09-28

### Security
- Strona Serwer podawała identyfikator każdego urządzenia każdemu z uprawnieniem *Widzi serwer*; identyfikatory wymagają też uprawnienia *Widzi szczegóły sieci*, jak wszędzie indziej.
- Dwa przełączniki uprawnień przestawione, zanim pierwszy się zapisał, wysyłały dwa razy tę samą listę wyjściową, a drugi cofał pierwszy: odebrane uprawnienie pozostawało przyznane, a wyłączona sekcja profilu publicznego pozostawała opublikowana, choć oba przełączniki pokazywały to, co wybrano.

### Stability
- **Tylko jeden import lub przywracanie naraz**: dwa przesłania wysłane jednocześnie przechodziły kontrolę, a przywracanie w ogóle nie sprawdzało importów; drugie czekało, aż baza danych się zwolni, i kończyło się niepowodzeniem po kilku minutach przesyłania.
- **Adres z uszkodzoną sekwencją `%` otwiera stronę „nie znaleziono”**, zamiast zostawiać na zawsze kręcący się wskaźnik.
- **Duży numer strony na liście daje pustą stronę**: wcześniej liczba się przepełniała i wracała pierwsza strona.

### Fixed
- **Tabele sortują liczby poprawnie w przeglądarce niemieckiej lub norweskiej**: „1,204” było odczytywane jako 1.204, a wszystko powyżej miliona jako tekst.
- **Widok tabeli wykresu sortuje daty, dni tygodnia i pozycje chronologicznie**, a nie alfabetycznie.
- **Historia pobierania podaje, skąd pochodzą importy**: Sonarr i Radarr podają indekser i protokół tylko w chwili pobrania wydania (grab), więc te zestawienia były zawsze puste. Nieudana paczka sezonu to jedno niepowodzenie, a nie jedno na każdy odcinek.
- **Usunięcie odtworzenia oglądanego razem** ponownie wyznacza grupy tego tytułu, zamiast zostawiać „grupę” złożoną z jednej osoby.
- **Czas razem dla jednej osoby nigdy nie przekracza czasu, przez który była obecna**: strona „Razem” przypisywała jej czas całego wieczoru.
- **Wieczór w towarzystwie przechodzący przez północ w sylwestra** (albo przez granicę „ostatnich N dni”) należy w całości do okresu, w którym się zaczął, zamiast być dwoma wieczorami w samotności.
- **Zaimportowane odtworzenia odcinków noszą nazwę odcinka** tam, gdzie Jellystat zachował tylko nazwę serialu.
- **Geolokalizacja znajduje bazę miast obok baz Country i ASN**, które zapisuje `geoipupdate`, zamiast się wyłączać.
- **Najczęściej cofany fragment podąża za filtrami strony**, tak jak jego liczniki.
- **„Twój rok jest gotowy”** trafia tylko do osób, których podsumowanie cokolwiek zawiera.
- **Ponowne rozwiązanie wyciszonego alertu pozostawia jego parę wyciszoną**; wyciszenie cofa tylko ponowne otwarcie.
- **Wygasły klucz API nie liczy się już do limitu dwudziestu.**
- **Limit sesji zwalnia najstarszą sesję** nawet wtedy, gdy wiele logowań przypada na tę samą sekundę.
- **Adres domowy znaleziony przez automatyczne sprawdzenie zostaje**, gdy ten sam adres zostanie usunięty z listy wpisanej ręcznie.
- **Zdjęcia w dzienniku audytu i na liście kluczy** tam, gdzie były tylko inicjały.
- **Lista powiadomień administratora mówi, do kogo należy osobiste miejsce docelowe**, zamiast „Twoje” przy każdym; **Ostatnio wysłane** pokazuje wszystkie miejsca docelowe ostatniego zdarzenia.
- **Esc na stronie tytułu lub osoby nigdy nie wychodzi z FinStats** i zamyka tylko okno dialogowe na wierzchu.
- **Strona jest rysowana nawet wtedy, gdy zapis anuluje pobieranie z wyprzedzeniem, na które czekała.**
- **Teraz odtwarzane pokazuje wstrzymanego widza, który przewija**, w miejscu, w którym jest teraz, a nie dopiero po wznowieniu odtwarzania.
- **Usunięcie połączenia aktualizuje listę dostępnych stron**, tak jak robi to zapisanie połączenia.
- **Data utworzenia bazy geolokalizacji zachowuje rok** w amerykańskim angielskim.

## [2.0.2] - 2026-09-28

### Security
- Za proxy limit logowań liczył pierwszy adres z `X-Forwarded-For`, czyli ten, który wysyła klient: przy każdej próbie nowy zmyślony adres, więc limit nigdy nie działał. Teraz liczy ostatni, ten zapisany przez proxy.
- *Dołączaj adresy IP i miejsca* wysyłało adresy do osobistego miejsca docelowego, którego właściciel nie ma uprawnienia *Widzi szczegóły sieci*, więc żadna strona by mu ich nie pokazała. Cudze nieudane logowania wymagają teraz również uprawnienia *Widzi serwer*, tak jak na stronie Bezpieczeństwo.

### Stability
- **Zatrzymanie nigdy nie przepada**: gdy odczyt biblioteki został odrzucony w pierwszych chwilach po starcie, zanim serwer zaczął na to czekać, odmowa była odnotowywana, ale FinStats działał dalej, zamiast się zatrzymać.
- **Jellyfin, który odpowiada w formie nieczytelnej dla FinStats, jest odrzucany**, a nie brany za wiarygodny: wszyscy użytkownicy albo wszystkie tytuły biblioteki przychodzące pod nieznanymi kluczami (`id` zamiast `Id`) przechodziły zabezpieczenie z pełną liczbą, a potem były oznaczane jako usunięte.
- **Koniec odtworzenia nigdy nie przepada**: gdy baza danych była zajęta dłużej, niż wynosi czas oczekiwania (bo trzymał ją import), odtworzenie zakończone w tej chwili pozostawało oznaczone jako trwające aż do następnego restartu. Teraz jest zapisywane w następnym przebiegu.
- **Ustawienia, których przywracanie nie umie odczytać, zostają nietknięte**: kopia zapasowa, której ustawienia nie pasowały już do tej wersji, przy przywracaniu resetowała wszystkie ustawienia: okno scalania, uprawnienia, adresy domowe, harmonogram.
- **Znak sterujący w nazwie lub tytule nie psuje już kart**: karta profilu, każda karta opowieści z tym tytułem i ich archiwum ZIP odpowiadały błędem.

### Fixed
- **Powiadomienia ustają dla osób wyłączonych lub usuniętych w Jellyfin**: do ich osobistych miejsc docelowych nadal trafiało wszystko, na co pozwalały ich dawne uprawnienia.
- **Powiadomienie w kolejce jest sprawdzane ponownie przed wysłaniem**: wyłączone miejsce docelowe albo właściciel, który stracił uprawnienie, nadal dostawali to, co czekało na ponowienie.
- **Z miejsca docelowego, które zawsze odpowiada `Retry-After`, FinStats rezygnuje** po pięciu próbach, jak z każdego innego, zamiast ponawiać przez trzydzieści dni.
- **Identyfikatory urządzeń na stronie osoby** są pokazywane tylko z uprawnieniem *Widzi szczegóły sieci*, jak wszędzie indziej.
- **Opublikowany rok we wszystkim czeka jeden dzień**: jego ukończone i porzucone seriale śledziły każde odtworzenie, więc włączenie porzuconego serialu zmieniało stronę publiczną w ciągu kilku sekund.
- **Serial, o który poproszono, liczy się jako obejrzany**, gdy obejrzano jeden z jego odcinków: każdy taki serial wyglądał na nigdy nieoglądany, w rozdziale Prośby i na karcie Prośby.
- **Rozdział Razem w podsumowaniu pomija telewizję na żywo**, tak jak reszta roku: kanał włączony we wspólnym gronie mógł zostać najważniejszym tytułem, a udział mógł przekroczyć 100%.
- **Karta Dni dla „ostatnich 12 miesięcy” kończy się na najnowszym dniu**: siatka gubiła ostatnie trzy lub cztery tygodnie.
- **Nowe odcinki serialu są ogłaszane raz na wieczór**: na serwerze na wschód lub zachód od Greenwich odcinki po obu stronach północy UTC były ogłaszane dwa razy.
- **Połączenie z Seerr nie ogłasza starych próśb jako gotowych do obejrzenia**: prośba dostępna już przy pierwszym odczycie, bez żadnej innej informacji o tym, kiedy się pojawiła, wyglądała, jakby dotarła w tej chwili.
- **Tytuły biblioteki usuniętej w Jellyfin znikają razem z nią**: zostawały w wyszukiwaniu, na półce Ostatnio dodane i w sumach.
- **Remuks pozostaje strumieniowaniem bezpośrednim**, gdy jeden odczyt na chwilę traci szczegóły transkodowania (przy przewijaniu), zamiast stawać się transkodowaniem do końca odtwarzania.
- **„Jellyfin nie działa” oznacza kilka nieudanych odczytów z rzędu**: pojedyncze przekroczenia czasu w odstępach wielu dni, na serwerze bezczynnie połączonym przez gniazdo, sumowały się w ten komunikat.
- **Import ze Streamystats pokazuje postęp na własnej karcie**, a nie na ostatnim wyniku importu z Jellystat.
- **Adres, którego host zaczyna się od „web”** (`http://web:8096`), jest odczytywany jako host, a nie obcinany jako ścieżka klienta webowego Jellyfin.
- **Błąd kolejki zaciera całe dane logowania** w adresie nawet wtedy, gdy hasło zawiera ukośnik.

## [2.0.1] - 2026-09-28

### Changed
- **Języki tytułu nie wchodzą w drogę**: nagłówek wymienia pierwsze trzy języki dźwięku i napisów, po jednym wierszu na każdy rodzaj, a nowa karta **Języki** wymienia wszystkie: kompletne razem, częściowe z liczbą odcinków, które je mają. Dwa tuziny języków napisów długiego serialu spychały grafikę o cały ekran w dół.

### Fixed
- **Plakat tytułu zostaje obok jego nazwy**: gdy tekst obok był dłuższy niż plakat (długi opis, wiele wytwórni), plakat zsuwał się na dół nagłówka, pod pustą grafikę.
- **Wersja, której FinStats nie umie odczytać, jest tak nazywana**: `app_version` ręcznie zmienione na coś nieczytelnego sprawiało, że FinStats odmawiał startu jako „starszy FinStats 2.0.1”, którym nie był. Nadal odmawia, ale teraz mówi, której wartości nie umiał odczytać i jak ją zapisać.

## [2.0.0] - 2026-09-25 to 2026-09-28

Podsumowanie 2026, profile publiczne i wspólne oglądanie

### Added
- **Z kim oglądasz**, w podsumowaniu: wieczory w towarzystwie, spędzony w nim czas, tytuł, który najczęściej łączył ludzi, i maksymalnie trzech towarzyszy, wymienionych z imienia w aplikacji i połączonych z ich profilami.
- **Seriale obejrzane do końca**, w podsumowaniu: ostatni odcinek każdego serialu obejrzany w danym roku, według tego samego rozumienia „obejrzanego” co paski postępu w profilu, a także seriale zaczęte i odłożone na później, obejrzane w mniej niż połowie, bez żadnego odtworzenia od dwóch miesięcy.
- **O co prosisz**, w podsumowaniu: prośby w Seerr złożone w danym roku, ile z nich dotarło i ile potem obejrzano. Tylko tam, gdzie są jakieś prośby.
- **Rok w porównaniu z poprzednim**: godziny, odtworzenia i dni oraz o ile więcej lub mniej.
- **Rok jako opowieść**: każdy rozdział jako karta 1080×1920, w formacie relacji na telefonach (rok, liczby, najpopularniejsze seriale, filmy i muzyka, gatunki, persona, godziny i dni, każdy dzień roku, rekordy, towarzystwo, ukończone seriale, prośby i poprzedni rok). Zapisz jedną albo wszystkie jako ZIP przez „Udostępnij rok jako karty”.
- **Karta nigdy nie wymienia nikogo innego**: karty są rysowane z kopii roku, która nie ma pola na inną osobę, pozycję w rankingu ani aplikację. W aplikacji karta nie zawiera żadnego imienia.
- **Rok serwera**, dla administratorów Jellyfin: wszystkie tytuły i sumy oraz persona całego domu, bez rankingu i bez wymieniania kogokolwiek. Przełącznik na stronie podsumowania, `scope=server` w API.
- **„Twój rok w podsumowaniu jest gotowy”**: powiadomienie w grudniu, wysyłane raz do każdej osoby, która w danym roku coś oglądała.
- **Profile publiczne**: osoba może opublikować część swojego profilu pod linkiem, który każdy otworzy bez konta (sumy i najpopularniejsze tytuły, serie dni i pory oglądania, cały rok jako opowieść z kart, ostatnie dziesięć odtworzeń); każda część jest wyłączona, dopóki właściciel nie włączy jej w Ustawieniach → Profil publiczny. Urządzenia, adresy, aplikacje, sposoby odtwarzania, ścieżki plików i inne osoby nigdy nie są publikowane, bez względu na to, co jest włączone, a wyświetlana nazwa to ta, którą wpisze właściciel, nigdy jego login w Jellyfin.
- **Nic na opublikowanej stronie nie jest nowsze niż dzień**, ani ostatnie odtworzenia, ani żadna suma, więc strona, którą ktoś obserwuje, nigdy nie zdradza, kto ogląda w tej chwili.
- **Karty do udostępniania**: każdy opublikowany profil ma obraz 1200×630, profilu i roku, który Discord, Signal, Mastodon i inne pokazują po wklejeniu linku i który można pobrać, by go opublikować. Rysowany na serwerze z tego, co pokazuje strona, i z niczego więcej.
- **Link, który można odwołać**: 22 losowe znaki, resetowane jednym kliknięciem; stary link odpowiada wtedy dokładnie tak jak link, który nigdy nie istniał.
- **Profile publiczne są wyłączone, dopóki administrator na nie nie zezwoli**: jeden przełącznik dla całego serwera, lista tego, kto co publikuje, i możliwość zdjęcia profilu. Publikacje, zmiany, resety i zdjęcia trafiają do dziennika audytu, a klucz nie może niczego upublicznić.
- **Razem**, osobna strona: kto ogląda z kim, ile i jak to się zmieniało. Czas razem, wieczory i udział czasu oglądania w towarzystwie, każde w porównaniu z poprzednim okresem; godziny w towarzystwie wobec godzin w samotności na dzień lub tydzień; każda para osób z jej wieczorami, czasem razem, ostatnim wspólnym oglądaniem i tym, co ogląda najczęściej; tytuły najczęściej oglądane razem; ostatnie wieczory; a dla każdego, kto może widzieć wszystkich, czas każdej osoby w samotności wobec czasu w towarzystwie.
- Liczy się każda para: wieczór trzech osób liczy się dla każdej z jego trzech par, podczas gdy wcześniej karta liczyła tylko dokładny skład osób. Czas razem dla pary to krótszy z dwóch pobytów.
- Karta **Oglądane razem** na Pulpicie i w profilu prowadzi do tej strony, a karta w profilu mówi, jaka część czasu oglądania tej osoby przypadła na towarzystwo.
- `GET /api/stats/groups` zyskuje `pairs`, `series`, `people`, `previous`, `bucket` oraz `watch_s` i `share` w sumach; dotychczasowe klucze się nie zmieniły.
- **Gdzie ludzie przerywają**, na stronie każdego filmu i odcinka: odsetek odtworzeń trwających w każdej minucie, od wszystkich na początku do tych, którzy dotarli do końca, z cofnięciami i włączeniami napisów w tych odtworzeniach zaznaczonymi na tej samej osi. Karta mówi, ile zatrzymań zmierzono (odtworzenia zarejestrowane przez FinStats), a ile oszacowano (odtworzenia zaimportowane, na podstawie czasu trwania), i potrzebuje trzech odtworzeń, zanim cokolwiek narysuje.
- **Kto ogląda dalej**, na stronie serialu: ile osób zaczęło każdy odcinek, po kolei, i kolumna **Ukończone** przy każdym odcinku. „Wszyscy rezygnują przy trzecim odcinku” to osoby, które nigdy nie włączą czwartego.
- **Pliki, które nigdy się nie odtwarzają**, na stronie Odtwarzanie: film lub odcinek uruchomiony co najmniej trzy razy, który ani razu nie przekroczył trzydziestu sekund, wraz z tym, kto próbował i w jakich aplikacjach. Dziennik Jellyfin nie zawiera błędów odtwarzania, więc świadkami są same odtworzenia; minimalna długość odtworzenia tu nie obowiązuje, bo właśnie do niej te odtworzenia nigdy nie docierają.
- **Najczęściej cofane** i **Włączone napisy**, na stronie Odtwarzanie: tytuły uszeregowane według przewinięć do tyłu na odtworzenie, z minutą, w której się skupiają, oraz według odsetka odtworzeń, w których w ciągu pierwszych dziesięciu minut włączono napisy. Wszystkie trzy listy wymagają uprawnienia *Widzi aktywność wszystkich*. Przewinięcia krótsze niż 20 sekund nigdy nie były rejestrowane, więc krótkie powtórzenie jednej kwestii jest z założenia niewidoczne.
- Przewinięcie zachowuje swój punkt wyjścia jako liczbę obok etykiety (`from_s` na osi czasu odtworzenia). Każde już zarejestrowane przewinięcie dostaje tę liczbę z etykiety, tak samo jak przywrócona kopia zapasowa sprzed tej zmiany.
- `GET /api/stats/files` dla trzech list; `GET /api/items/{id}` zyskuje `insights`, a odcinki serialu zyskują `users` i `finished`.
- **Klucze API**: każdy zalogowany może tworzyć klucze dla siebie w Ustawieniach → Klucze API, dla skryptu, panelu lub telefonu. Klucz jest wysyłany jako `Authorization: Bearer fs_…`, pokazywany raz, przechowywany jako skrót, może mieć datę wygaśnięcia i unieważnia się go jednym kliknięciem; administrator widzi klucze wszystkich i może je unieważnić. Klucz odpowiada tej samej osobie z tymi samymi uprawnieniami co logowanie, odczytywanymi na bieżąco przy każdym żądaniu, więc utrata prawa do logowania od razu zatrzymuje wszystkie klucze. Klucz nie może tworzyć ani unieważniać kluczy.
- **Kanał kalendarza**: `GET /api/calendar.ics?key=…` to to, co nadchodzi w Sonarr i Radarr, jako iCalendar, który można zasubskrybować w telefonie. Klucz o zakresie *Tylko kanał kalendarza* otwiera kanał i nic więcej; kanał odczytuje klucz z adresu, bo subskrybowany kalendarz nie może wysłać nagłówka, nigdy z ciasteczka, i nie wymienia nikogo z imienia.
- **Dziennik audytu** w Serwer → Audyt, dla administratorów Jellyfin: każde logowanie i nieudana próba z adresem, każde zmienione ustawienie lub uprawnienie i na co je zmieniono, każdy klucz utworzony, użyty lub unieważniony, każde połączenie, miejsce docelowe, kopia zapasowa i import, każde usunięte odtworzenie i rozwiązany alert, wraz z tym, kto, skąd, którym kluczem i czy się udało. Przechowywany przez rok, część kopii zapasowych. `GET /api/audit` przegląda go stronami.
- **Import ze Streamystats**, osobna sekcja w Ustawieniach obok Jellystat. Weź plik z **Settings → Backup & Import → Download Backup** w Streamystats i upuść go: lata historii docierają w kilka sekund, a nic, co już masz, nie jest liczone ponownie. `finstats import-streamystats <file>` robi to bez interfejsu, `POST /api/import/streamystats` przez API; naraz działa tylko jeden import, obojętnie jakiego rodzaju.
- **Korzystasz z obu trackerów? Zaimportuj oba pliki.** Odtworzenie jest rozpoznawane jako już obecne w FinStats po identyfikatorze, który nadał mu tracker, a gdy go brak, po tej samej osobie oglądającej tę samą pozycję, gdy jeden z końców odtworzenia leży blisko odtworzenia z historii; dzięki temu eksport z Jellystat i eksport ze Streamystats z tych samych wieczorów można zaimportować oba, w dowolnej kolejności, a wieczór, który FinStats zarejestrował sam, nie zostanie też zdublowany importem. „Blisko” określa okno scalania w Ustawieniach → Zbieranie danych.
- Zaimportowane wiersze mówią, czego nie mogą powiedzieć. Odtworzenie, które Streamystats zarejestrował sam, nie zachowuje nic o pliku (kodeka, rozdzielczości, przepływności ani kontenera), więc te pola zostają puste, zamiast wypełniać je domysłem, a karty, które by je pokazały, same się ukrywają. Odtworzenie, które Streamystats przejął z Jellystat, zawiera całą sesję i to ona jest odczytywana: datowana chwilą zakończenia, jedyną chwilą, którą zachowały oba trackery, i opisywana jako transkodowanie tylko wtedy, gdy sesja naprawdę nim była.
- Wiersze, które Streamystats zapisał, bo Jellyfin zgłosił pozycję jako obejrzaną, dla oglądania, którego sam nigdy nie widział, są liczone i pomijane: trwają tyle co cały film i byłyby czasem oglądania w wieczór, w którym nikt nie oglądał. Wynik podaje, ile ich było.
- Jak dane Streamystats są interpretowane, pole po polu, na podstawie prawdziwego eksportu, a nie dokumentacji: `docs/streamystats-import.md`.
- **Z którego trackera pochodzi każde odtworzenie**, jako filtr na stronie Aktywność: to, co FinStats zarejestrował sam, to, co przyszło z Jellystat, i to, co przyszło ze Streamystats. Oferowane są tylko te źródła, które faktycznie są w historii, a filtr podlega tym samym ograniczeniom co każda inna lista, więc ktoś, kto może widzieć tylko własne odtworzenia, dowiaduje się tylko, skąd pochodzi jego własna historia.
- **Filtry na stronie Aktywność to listy rozwijane z polami wyboru**, więc można poprosić o filmy *i* odcinki bez muzyki, o dwa sposoby odtwarzania albo o dwie osoby naraz, zamiast o jedno z każdego albo o wszystko. Typ multimediów zyskał opcję **Inne**, skoro można je teraz łączyć.
- Kilka osób naraz wszędzie tam, gdzie jest filtr użytkowników (Pulpit, odtwarzanie, biblioteki, bezpieczeństwo i reszta), a mapa i każda statystyka podążają za wyborem. Filtr użytkowników nadal tylko zawęża to, co ktoś może widzieć: bez uprawnienia *Widzi aktywność wszystkich* żądanie jest przypięte do pytającego, bez względu na to, kogo i ile osób wymienia adres.
- **Jasny motyw**: papier washi z tekstem w kolorze tuszu i cynobrowym akcentem. FinStats podąża za jasnym lub ciemnym ustawieniem urządzenia, dopóki nie wybierze się motywu.
- **Przycisk motywu obok Wyloguj się** przełącza kolejno Systemowy, Jasny i Ciemny. Wybór jest zapamiętywany w tej przeglądarce i stosowany przed narysowaniem strony, więc drugi motyw nigdy nie mignie.
- **Otwórz w Jellyfin**: każda strona filmu, serialu i odcinka ma przycisk, który otwiera stronę tego tytułu w Jellyfin, w nowej karcie.
- **Adres Jellyfin dla użytkowników** w Ustawieniach → Jellyfin: dokąd prowadzą te przyciski, na wypadek gdyby adres, z którym łączy się FinStats, był nieosiągalny dla przeglądarki. Puste pole oznacza adres, z którym łączy się FinStats; ustawić go może tylko administrator Jellyfin.

### Changed
- **FinStats ma nowy dom**: `github.com/finstats/finstats`, a obraz to `ghcr.io/finstats/finstats`. Zmień nazwę obrazu w poleceniu `docker run` lub w pliku compose, by dalej dostawać aktualizacje; folder danych zostaje bez zmian.
- **Ustawienia pokazują jedną sekcję naraz.** Strona miała czternaście kart w jednej kolumnie i sześćset słów pomocy. Teraz każda sekcja ma własny adres (`/settings/collection`), lista po lewej przełącza między nimi, a na telefonie lista zawija się w żetony. Linki do starych kotwic nadal trafiają we właściwe miejsce.
- **Ustawienie to jeden wiersz**: czym jest, jedno zdanie z regułą, kontrolka po prawej. Liczby w sekcji zapisuje się razem jednym przyciskiem na dole; przełączniki nadal zapisują się same. Karty stanu (Zadania, Połączenia wychodzące, Baza danych) i łącze do licencji są w sekcji **System**, a oba importery mają własną sekcję.
- **Znajdź ustawienie** w polu w nagłówku (`/` ustawia na nim fokus) po dowolnym słowie z jego nazwy; wynik mówi, w której sekcji ustawienie się znajduje, a Enter przenosi do jego wiersza.
- **Ustawienia są dla każdego zalogowanego**: sekcje do zarządzania serwerem zostają dla tych, którzy mogą nim zarządzać, a osoba, której pozwolono tylko na powiadomienia, może wreszcie dotrzeć do swoich miejsc docelowych.
- **Strona Serwer pokazuje jedną sekcję naraz**, tak jak Ustawienia: Przegląd (system i pamięć masowa), Zadania, Urządzenia, Wtyczki, Dziennik i Audyt, każda pod własnym adresem. Linki do `/server#jobs` nadal trafiają we właściwe miejsce.
- **Dziennik serwera to sekcja strony Serwer** (`/server/log`), a nie osobna strona w menu; `/events` przekierowuje tam wraz z filtrami.
- Powiadomienie o nieudanym zadaniu, nieudanej kopii zapasowej lub połączeniu, które przestało odpowiadać, prowadzi do sekcji ustawień, której dotyczy, a nie na górę strony.
- To, co biblioteka może powiedzieć o zaimportowanym odtworzeniu (czym było, który to odcinek, czas trwania, do której biblioteki należy), jest uzupełniane po **każdym** odczycie biblioteki, a nie tylko w chwili importu. Historia jest zwykle importowana, zanim FinStats w ogóle odczyta bibliotekę, więc te odtworzenia na zawsze zostawały zgadnięte albo nieznane.
- Przywracanie kopii zapasowej FinStats nie dodaje już odtworzenia, które przyszło z trackera pod innym identyfikatorem, a nadal zachowuje dwa prawdziwe oglądania tego samego w odstępie kilku minut.
- Szczegóły odtworzenia podają tracker, który je dostarczył, zamiast nazywać wszystko, co zaimportowane, odtworzeniem z Jellystat.

### Performance
- **Restart jest niemal natychmiastowy przy każdej historii.** Każdy start wyliczał na całej historii, kto oglądał razem, i od nowa rozstrzygał, które odtworzenia były lokalne, w obu przypadkach niczego nie zmieniając: 24 s i 5,5 s przy dziesięciu milionach odtworzeń. Teraz start robi tylko to, co zostawił poprzedni przebieg, a całość od nowa tylko po aktualizacji lub po zmianie ustawienia, od którego to zależy.
- **Tłum, który pojawia się naraz, jest rejestrowany w kilka sekund.** Każde nowe odtworzenie, każdy zapis postępu i każde urządzenie były zapisywane osobno, a każde zakończone odtworzenie zamykane z osobnym przeglądem całej historii tytułu; teraz przebieg zapisuje każdy rodzaj za jednym razem. 100 000 odtworzeń rozpoczynających się jednocześnie zajmuje 3 s, a 25 000 kończących się jednocześnie zajmowało wcześniej ponad cztery minuty.
- **Teraz odtwarzane pozostaje szybkie przy tysiącach widzów.** Ustalanie, kto ogląda razem, porównywało każdy strumień z każdym innym i wymieniało z imienia każdego towarzysza: czterdzieści sekund przy 10 000 strumieni, a do tego gigabajty pamięci. Teraz strumienie są porównywane w obrębie tytułu, a każdy pokazuje kilka imion i liczbę pozostałych osób.
- Ustalanie, kto oglądał razem, przepisywało każde zgrupowane odtworzenie z historii przy każdym starcie FinStats, imporcie lub zakończeniu odtworzenia; teraz zapisuje tylko to, co się zmieniło. Znajdowanie zakończonych odtworzeń nie zwalnia już z kwadratem liczby trwających.
- **Rozpoznawanie odtworzenia, które historia już ma, korzysta z indeksu**, więc import 150 000 odtworzeń trwa kilka sekund, a ponowne łączenie po odczycie biblioteki przechodzi przez tytuły, zamiast czytać każde odtworzenie i przez cały ten czas blokować bazę danych.
- **Analizy odtwarzania i przegląd są szybsze dla całego okresu**: ich niezależne części są wyliczane równolegle, wykres jednoczesnych strumieni ustala lokalny dzień raz na kwadrans zamiast raz na odtworzenie, a gatunki są liczone na tytuł. Przy milionie odtworzeń: z 4,1 s do 2,4 s i z 1,6 s do 1,1 s.

### Stability
- **Biblioteka telewizyjna nie wygląda już na wypatroszoną.** Odczyt biblioteki ma dwa przebiegi (każda pozycja, a potem mniejszy przebieg tylko dla obsady i twórców filmów i seriali), a kontrola, która decyduje, czy odczytowi można zaufać na tyle, by usunąć to, czego nie zobaczył, patrzyła na drugi z nich. W bibliotece telewizyjnej lub muzycznej te dwie liczby są bardzo od siebie odległe, więc kontrola odrzucała każdy odczyt i zatrzymywała FinStats, a instalacja restartowała się i robiła to samo od nowa. Nic nigdy nie zostało błędnie usunięte, ale żaden odczyt biblioteki nie mógł się zakończyć. Jeśli twoja biblioteka seriali lub muzyki nie przyjmowała nowych tytułów, to właśnie dlatego.
- Import mógł zakończyć się błędem „database is locked”, gdy inny zapis (odświeżane odtworzenie) trafił między jego pierwszy odczyt a pierwszy zapis. Oba importery biorą blokadę zapisu od samego początku, a dziennik bazy danych nie pozostaje już tak duży jak największy import w historii.
- **Plik bazy danych, który jest uszkodzony, ucięty, cudzy lub tylko do odczytu, jest odrzucany od razu**, z nazwą pliku i informacją, że nic nie zostało zmienione, zamiast 30 sekund ciszy; w bazie danych innego programu nic nigdy nie jest zapisywane.
- Kopia zapasowa lub migawka sprzed aktualizacji przerwana w połowie nie zostawia już pliku, który mógłby zająć miejsce dobrej migawki wśród trzech zachowywanych.
- **Nadpisanie bazy lokalizacji podczas pracy FinStats nie powoduje już awarii**: FinStats czyta z prywatnej kopii pliku.
- Aplikacja Jellyfin, która za każdym razem zgłaszała nowy identyfikator urządzenia, mogła sprawić, że pamięć FinStats rosła przez cały czas jego działania; urządzenia niewidziane od pięciu minut są teraz zapominane.
- Jedna osoba ma co najwyżej 30 zalogowanych sesji; najstarsza jest wylogowywana przy następnym logowaniu.

### Fixed
- **Ten sam wieczór nie jest już liczony dwa razy, gdy pozycja zmieniła nazwę.** Ponowne łączenie historii z tytułem dodanym ponownie w Jellyfin mogło zrobić z odtworzenia zaimportowanego z trackera duplikat już istniejącego, bo kontrola, która by to wychwyciła, działała, zanim identyfikator się zmienił. Teraz działa ponownie wszędzie tam, gdzie identyfikatory są łączone na nowo. Usuwane są zawsze tylko zaimportowane odtworzenia, nigdy te, które FinStats zarejestrował sam, a dziennik podaje, ile ich zniknęło. Pierwszy start wersji 2.0 raz przegląda całą historię w poszukiwaniu duplikatów, które już powstały.
- **Sesja podąża teraz za osobą taką, jaką Jellyfin widzi ją w tej chwili.** Administrator zdegradowany w Jellyfin zachowywał pełny dostęp do FinStats do końca swojej 30-dniowej sesji, a ktoś wyłączony lub usunięty w Jellyfin pozostawał zalogowany. Teraz tracą dostęp, gdy tylko FinStats odczyta użytkowników Jellyfin (co 15 minut albo od razu z Ustawień), tak jak zawsze było z kluczami API.
- Logowanie zapisuje na liście użytkowników FinStats to, co Jellyfin mówi o danej osobie, więc zaimportowana kopia zapasowa trackera nie może już zrobić z kogoś administratora, zanim pierwszy odczyt użytkowników to poprawi.
- Ktoś usunięty w Jellyfin jest oznaczany jako usunięty nawet wtedy, gdy dwa odczyty użytkowników przypadają na tę samą sekundę.
- **Błędy pobierania nigdy nie pokazują hasła klienta pobierania ani klucza API indeksera**, które Sonarr i Radarr mogą w nich cytować.
- Odtworzenia ponownie połączone z tytułem o zmienionej nazwie podczas odczytu biblioteki nie były sprawdzane pod kątem wspólnego oglądania aż do następnego restartu; teraz są, od razu.

## [1.6.5] - 2026-09-24

### Added
- **Przed każdą aktualizacją automatycznie powstaje pełna kopia zapasowa.** Gdy nowszy FinStats po raz pierwszy otwiera twoją bazę danych, zapisuje jej kompletną kopię (całą historię, ustawienia i wszystko inne) w `data/pre-update-backups/`, *zanim* cokolwiek zmieni. Jeśli aktualizacja kiedyś coś zepsuje, twoje dane są bezpieczne: zatrzymaj FinStats, połóż kopię w miejscu `finstats.db`, uruchom poprzednią wersję i zgłoś błąd. Nic nie przepada. Zachowywanych jest kilka najnowszych kopii; jeśli brakuje miejsca na dysku, ustaw `FINSTATS_SKIP_PREUPDATE_BACKUP=1`, aby to wyłączyć.

## [1.6.4] - 2026-09-24

### Added
- **Pięć nowych miejsc, do których można wysyłać powiadomienia**: Telegram, Slack, Pushover, Pushbullet i e-mail, obok dostępnych już miejsc docelowych typu webhook, Discord, ntfy i Gotify. Każde prosi tylko o to, czego potrzebuje (identyfikator czatu, klucz użytkownika, skrzynkę pocztową), a trzy, które działają w ramach jednej usługi (Telegram, Pushover, Pushbullet), są osiągalne zawsze tylko tam, więc tokenu nie da się wysłać na podobnie wyglądający host. Poczta wychodzi przez TLS od pierwszego bajtu albo przez `STARTTLS` i nie ma trzeciej możliwości.
- **Licencje** w **Ustawieniach**: własna licencja GPL FinStats i każda licencja zewnętrzna, na której jest zbudowany (253 pakiety Rust, dwie dołączone czcionki, kontury mapy i baza geolokalizacji), każdy tekst w całości, tak jak napisał go jego projekt. Wybierz komponent, a jego licencja otworzy się w oknie obok listy; pakiet z kilkoma plikami licencji przełącza się między nimi w tym samym miejscu. Lista jest generowana z pakietów, z których FinStats faktycznie jest budowany, więc nie da się dodać zależności bez zapisania jej licencji.

## [1.6.3] - 2026-09-24

### Fixed
- Zabezpieczenie wkracza teraz także wtedy, gdy Jellyfin nie zwraca *żadnych* bibliotek albo *żadnych* użytkowników (to zawsze uszkodzony odczyt, nigdy zwykły dzień), co przepuszczało dostrojenie z wersji 1.6.2. Utrata części bibliotek nadal jest traktowana jako coś zwyczajnego; utrata wszystkich naraz już nie.

## [1.6.2] - 2026-09-24

### Fixed
- Zabezpieczenie integralności danych dodane w wersji 1.6.1 było zbyt gorliwe: biblioteka, która naprawdę traciła większość pozycji (powiedzmy garść klipów, których pliki zniknęły), była brana za uszkodzony odczyt, więc FinStats odmawiał jej aktualizacji i się zatrzymywał. Teraz wkracza tylko przy *wyraźnie* uszkodzonym odczycie: gdy biblioteka na tyle duża, by miało to znaczenie, wraca całkowicie pusta albo gdy duża biblioteka zostaje wypatroszona prawie do zera. Codzienne zmiany są stosowane jak wcześniej, a przypadek „Jellyfin nic nie zwrócił” nadal jest wychwytywany.

## [1.6.1] - 2026-09-24

### Added
- **FinStats nie ufa już odczytowi z Jellyfin, który wymazałby twoją historię.** Jeśli biblioteka, dla której FinStats przechowuje tysiące pozycji, nagle wraca pusta lub prawie pusta (aktualizacja Jellyfin, która zmieniła API, błąd przebrany za pusty wynik), FinStats zachowuje dane, które ma, oznacza synchronizację jako nieudaną (i powiadamia o tym), po czym zatrzymuje się w uporządkowany sposób z jasnym komunikatem, zamiast oznaczać wszystko jako usunięte. Uruchom go ponownie, gdy Jellyfin wróci do normy, albo ustaw `FINSTATS_ALLOW_LIBRARY_SHRINK=1`, jeśli biblioteka naprawdę została opróżniona. To samo zabezpieczenie obejmuje listę bibliotek i listę użytkowników.

### Security
- Nagłówek `X-Forwarded-Host` wysłany przez klienta mógł poręczyć za obcy `Origin` i przepuścić zapis z innej witryny przez zabezpieczenie tego samego pochodzenia. Ten nagłówek jest teraz zaufany tylko za zaufanym proxy (`FINSTATS_TRUST_PROXY`), według tej samej reguły, której już używa ustalanie adresu klienta. Ciasteczko sesji `SameSite=Lax` było i pozostaje pierwszą linią obrony.
- Osobiste miejsce docelowe powiadomień można było skierować na `0.0.0.0` lub `::`, które prowadzą do tej maszyny. Kontrola, która pilnuje, by osobiste miejsca docelowe wskazywały adresy publiczne, liczy je teraz jako lokalne, obok pętli zwrotnej i zakresów prywatnych.

### Fixed
- Strona **Odtwarzanie** wczytuje się teraz także dla biblioteki zawierającej odtworzenie bez zapisanego sposobu odtwarzania (wiersz przywrócony z kopii zapasowej starszej niż ta kolumna); takie odtworzenie liczy się po prostu jako odtwarzanie bezpośrednie.
- Bardzo długie słowo w wyszukiwaniu lub filtrze nie powoduje już błędu serwera: SQLite odrzuca wzorzec `LIKE` dłuższy niż 50 000 znaków, więc słowa filtra są przed zapytaniem przycinane do rozsądnej długości.
- Strona **Ustawienia** nie przewija się już w bok na telefonie, gdy karta wyświetla całe zdanie, na przykład wynik przywracania.

## [1.6.0] - 2026-09-24

Powiadomienia

### Added
- Karta **Powiadomienia** w Ustawieniach: dodaj miejsce docelowe, zaznacz, o czym ma się dowiadywać, i naciśnij Test. Bez miejsca docelowego nie jest wysyłane żadne żądanie.
- Cztery rodzaje miejsc docelowych: zwykły **webhook** (JSON, opisany w `docs/api.md`, opcjonalny nagłówek `Authorization`), **Discord** (embed przez adres URL webhooka kanału), **ntfy** (ntfy.sh lub własny serwer, na wybrany temat) i **Gotify** (własny serwer, z tokenem aplikacji). Tytuły są przesyłane w treści wiadomości, a nie w nagłówku.
- Jedenaście rodzajów zdarzeń, każde z osobnym polem wyboru: niemożliwa podróż, konto widziane w nowym kraju, seria nieudanych logowań, zadanie, które się nie powiodło, kopia zapasowa, której nie dało się zapisać, coś, co przestało odpowiadać (Jellyfin, Sonarr, Radarr lub Seerr), i chwila, gdy znów odpowiada, nowe tytuły w bibliotece (zebrane według serialu i dnia, tak jak na półce na Pulpicie), prośba, którą można już obejrzeć, oraz rozpoczęcie lub zakończenie odtwarzania. Dwa ostatnie są domyślnie wyłączone.
- Prośba, którą można już obejrzeć, podaje tytuł, kto prosił i jak długo czekał.
- Osobiste miejsca docelowe: przyznaj uprawnienie **Otrzymuje powiadomienia**, a dana osoba będzie mogła dodać własne miejsce docelowe, do którego trafia tylko to, co i tak może zobaczyć: zawsze własne prośby i alerty, cudze z uprawnieniem *Widzi aktywność wszystkich*, adresy z uprawnieniem *Widzi szczegóły sieci*. Osobiste miejsce docelowe musi wskazywać adres publiczny; tylko administrator może skierować je do sieci wewnętrznej.
- **Dołączaj adresy IP i miejsca**, osobno dla każdego miejsca docelowego, domyślnie wyłączone: bez tego wiadomość o niemożliwej podróży podaje tylko miejsca („Oslo, Norwegia i Londyn, Wielka Brytania”).
- **Ostatnio wysłane** w tej samej karcie: co zostało powiedziane, do którego miejsca docelowego i co odpowiedziała druga strona, gdy wiadomość nie dotarła. Nieudana wiadomość jest ponawiana cztery razy w ciągu mniej więcej godziny, a na miejsce docelowe, które ogranicza liczbę żądań, FinStats czeka.
- **Adres FinStats**, ustawienie dla administratorów, to podstawa, z której budowany jest link w powiadomieniu. Puste pole oznacza wiadomości bez linku.
- Zaplanowane zadania Jellyfin na stronie Serwer: co każde z nich robi, opisane słowami, a nie nazwą z kodu, i czy należy do tych ciężkich. Trwające zadanie pokazuje pasek postępu, procent i pozostały czas; pozostałe pokazują, kiedy uruchomią się następnym razem, kiedy uruchomiły się ostatnio i czy się udało. Uwzględnione są też zadania, które Jellyfin ukrywa na własnym panelu, a nic w FinStats nie może uruchomić ani zatrzymać zadania.
- Pozostały czas zadania jest mierzony na podstawie tempa, w jakim faktycznie rósł jego procent, względem najnowszego odczytu na tyle odległego, by coś znaczył, a nie średniej z całego przebiegu, więc zadanie, które przyspiesza lub zwalnia, jest opisywane tempem, jakie ma teraz.
- Listy zadań, które FinStats i tak odczytuje z innych powodów (sprawdzanie skanowania biblioteki co 5 minut, szczegóły serwera co 15), zasilają ten sam pomiar, więc zadanie, które trwa od godziny, zwykle ma oszacowanie od chwili otwarcia strony. Nie wymaga to od Jellyfin żadnych dodatkowych zapytań.
- Trwające zadanie, dla którego nic jeszcze nie zmierzono, pokazuje **ETA** i krążący wielokropek zamiast liczby, a takie, którego procent nie zmienił się od pół minuty, ma obok napis „68% od 4 min”. Nic nie zastępuje oszacowania, którego nie zmierzono.
- Lista wszystkich wersji obok listy zmian: jedno kliknięcie przenosi do dowolnego wydania, czytane wydanie jest na niej wyróżnione, a na wąskim ekranie zmienia się ona w rząd serii nad notami.

### Changed
- Lista zmian nie jest już zwijana. Każda seria jest otwarta, a do poruszania się służy lista wersji, zamiast jednej otwartej grupy naraz i kliknięcia, by dotrzeć do każdej innej.
- **Ustawienia → Połączenia wychodzące** wymieniają każde miejsce docelowe powiadomień obok Jellyfin i połączeń z usługami: czym jest, jaki ma host, czy jest włączone i kiedy ostatnio przyjęło wiadomość.
- Nic w zbieraniu danych, historii ani w żadnej liczbie na ekranie się nie zmienia. FinStats nadal tylko *odczytuje* dane z Jellyfin, Sonarr, Radarr i Seerr.

## [1.5.2] - 2026-09-22

### Changed
- Odczyt listy sesji Jellyfin i wyzerowanie zegara, który o niego poprosił, to jedno wywołanie, więc nie mogą się już rozjechać. Tak właśnie wyglądał błąd lawiny żądań wykryty podczas budowania wersji 1.5.0.
- Udowodnienie tego zajmuje ułamek sekundy zamiast siedmiu minut na prawdziwym serwerze, więc jest sprawdzane przy każdej zmianie. Te same żądania, ta sama historia, te same liczby na ekranie.

## [1.5.1] - 2026-09-22

### Fixed
- Transkodowanie jest zapisywane raz, a nie raz na sekundę. Gdy odtwarzanie zaczyna się od transkodowania i wraca do odtwarzania bezpośredniego, Jellyfin zostawia w sesji szczegóły transkodowania; każdy odczyt był porównywany z wartością, którą FinStats właśnie nadpisał, więc każdy wyglądał na zmianę, a strona Aktywność zapełniała się wierszami „Transkodowanie” z napisem „Odtwarzanie bezpośrednie: …” pod spodem.
- Wiersze już zapisane są porządkowane przy starcie: tam, gdzie kilka kolejnych mówi to samo, zostaje pierwszy. Odtwarzanie, które wymagało transkodowania, nadal liczy się jako transkodowanie.

## [1.5.0] - 2026-09-22

Śledzenie sesji przez WebSocket

### Changed
- FinStats od chwili uruchomienia utrzymuje jedno otwarte połączenie z Jellyfin i to przez nie dowiaduje się o odtwarzaniu. Wersja 1.4.0 dostarczyła je jako ustawienie, domyślnie wyłączone, żeby sprawdziło się na prawdziwych serwerach; teraz jest zawsze włączone. Te same odtworzenia, pauzy i przewinięcia, wyliczane przez ten sam kod.
- Nic nie jest odtwarzane: FinStats nie pyta Jellyfin o nic. Wieczór, w którym nikt nie ogląda, nie kosztuje żadnych żądań.
- Coś jest odtwarzane: lista sesji jest odczytywana raz na sekundę. To stąd pauza, przewinięcie czy zmiana ścieżki biorą swoją dokładność i to właśnie kończy odtwarzanie, którego klient zniknął bez pożegnania.
- Wszystko, co wczytane, jest wstrzymane przez trzy odczyty z rzędu: pytanie ustaje, a informacje przenosi połączenie, podczas gdy wcześniej wstrzymany film kosztował to samo pytanie co sekundę, około 16 MB na godzinę. Na wznowienie albo na to, że ktoś inny coś uruchomi, FinStats reaguje w ciągu mniej więcej sekundy, a czas pauzy nadal nigdy nie liczy się jako czas oglądania.
- Każdy odczyt wychodzący prosi o skompresowane odpowiedzi i je rozpakowuje (Jellyfin, Sonarr, Radarr i Seerr kompresują, gdy się o to poprosi), co zmniejsza o ponad połowę dwa największe odczyty: listę sesji podczas odtwarzania i odczyt biblioteki. Baza geolokalizacji jest nadal pobierana w takiej postaci, w jakiej jest przechowywana.
- Karta Jellyfin w Ustawieniach mówi, która z dwóch połówek działa, a kropka stanu na żywo pokazuje, czy połączenie przenosi dane, a nie, w której połówce jesteśmy.
- Naraz działa tylko jedna z nich: gdy FinStats sam pyta, prosi Jellyfin, by przestał wysyłać, i prosi o wznowienie w chwili zakończenia ostatniego odtwarzania. Połączenie przez cały czas pozostaje otwarte.
- Jellyfin, który na subskrypcję nie odpowiada niczym, jest uznawany za sprawny, dopóki odpowiada na sygnały podtrzymania: FinStats mówi o tym raz, w międzyczasie sam odczytuje listę sesji i pozostaje zasubskrybowany, dopóki pierwsza przesłana wiadomość nie rozstrzygnie sprawy. „Nie umie wysyłać” i „właśnie się restartował, gdy pytaliśmy” wyglądają tak samo, więc cisza nigdy nie jest werdyktem.
- Tryb awaryjny się nie zmienił: połączenie, które się zamyka, milknie albo okazuje się nie znać tego protokołu, już w następnym przebiegu wraca do pytania według zegara i ponawia próby połączenia, w odstępach **Podczas oglądania sprawdzaj co** i **Gdy nic nie jest odtwarzane, sprawdzaj co**.

### Added
- Co robi kolektor, w `GET /api/status`: w którym z czterech stanów jest (`idle_socket`, `playing_poll`, `paused_socket`, `fallback`), czy połączenie jest otwarte i zasubskrybowane, jak często pyta, ile odczytów sesji wyszło w ostatniej minucie i kiedy to się ostatnio zmieniło. Odpowiedź pochodzi z pamięci (bez żadnego żądania i zapytania) i nie zawiera imion, tytułów ani liczb.
- Te pola są przy każdym przebiegu sprawdzane względem siebie, a niezgodność, która się utrzymuje, jest zapisywana w dzienniku.
- Jeden ogranicznik pod każdym odczytem sesji w procesie: najwyżej 2 na sekundę i 70 na minutę, a wszystko ponad to jest opóźniane, liczone i odnotowywane w dzienniku najwyżej raz na minutę. Zwykłe oglądanie pozostaje daleko poniżej tego progu.

### Fixed
- Aktualizacje na żywo naprawdę zastępują odpytywanie. Po włączeniu wersja 1.4.0 nadal pytała o listę sesji co pięć sekund i otwierała połączenie na nowo co dwadzieścia do czterdziestu sekund, bo oceniała połączenie po tym, jak dawno przyszła lista, a Jellyfin wysyła ją, gdy coś się *zmieni*, i nic pomiędzy. O tym, czy połączenie żyje, decyduje teraz to, czy Jellyfin w ogóle odpowiada, co jest sprawdzane sygnałem podtrzymania mniej więcej dwa razy na minutę.
- Lista sesji, która dotarła, gdy FinStats był między przebiegami, była odrzucana przy założeniu, że kolejna przyjdzie półtorej sekundy później. Teraz jest zachowywana i wykorzystywana.
- **Ostatnie sprawdzenie** w Ustawieniach i karta Połączenia wychodzące nie dezaktualizują się już, gdy połączenie milczy: pokazują odpowiedź Jellyfin na sygnał podtrzymania.

### Removed
- Ustawienie **Pozwól Jellyfin przesyłać, co jest odtwarzane** w Ustawieniach → Zbieranie danych oraz `live_socket` z API ustawień.
- `socket_enabled` ze stanu kolektora: nie ma już stanu „wyłączone”.
- `last_reconcile_at` ze stanu kolektora, zastąpione przez `socket_live`.

## [1.4.1] - 2026-09-22

### Changed
- Seerr jest odczytywany w całości tylko wtedy, gdy coś się zmieniło: co pięć minut FinStats prosi o jeden wiersz, czyli ostatnio zmienioną prośbę, i kończy przebieg, jeśli już ją zna. To mniej więcej kilobajt zamiast około sześćdziesięciu za stronę pięćdziesięciu próśb. Prośby wciąż w drodze są sprawdzane co kwadrans, wszystko jest wypisywane raz dziennie, a nowa prośba nadal pojawia się w ciągu pięciu minut.
- Kolejki Sonarr i Radarr są odczytywane co pięć sekund, gdy pokazuje je jakaś strona, co minutę, gdy coś jest w kolejce, i co pięć minut, gdy jest pusta, zamiast co minutę przez całą dobę. Otwarcie strony, połączenie usługi albo nowa prośba w Seerr odświeżają je od razu.

## [1.4.0] - 2026-09-21

Opcjonalne sesje przez WebSocket i wszystko, z czym łączy się FinStats

### Added
- **Pozwól Jellyfin przesyłać, co jest odtwarzane** w Ustawieniach → Zbieranie danych, wyłączone, dopóki go nie włączysz: jedno połączenie przenosi listę sesji, a około 17 000 żądań dziennie zamienia się w prawie żadne. Odtworzenia, pauzy, przewinięcia i grupy są wyliczane dokładnie tym samym kodem co wcześniej. Gdy połączenie działa, FinStats nadal wysyła jedno zwykłe żądanie na minutę, *gdy coś jest odtwarzane*, a żadnego, gdy nic nie jest. Połączenie, które się zamyka, milknie na piętnaście sekund albo okazuje się nie znać tego protokołu, w następnym przebiegu wraca do zegara i ponawia próby połączenia.
- **Połączenia wychodzące**, karta w Ustawieniach dla administratorów Jellyfin: każde miejsce, z którym może łączyć się FinStats (twój Jellyfin, usługa „jaki mam IP”, baza DB-IP oraz każdy Sonarr, Radarr lub Seerr), z przeznaczeniem, informacją, czy jest włączone, i tym, kiedy ostatnio odpowiedziało. Karta powstaje z tego, co FinStats już wie, i pokazuje tylko adresy: nigdy klucza, nigdy ścieżki bazowej.
- **Sprawdź teraz** w Ustawieniach → Sieć domowa, na dzień, w którym zmieni się twój adres publiczny.

### Changed
- Sprawdzanie „jaki mam IP” odbywa się raz, gdy FinStats po raz pierwszy potrzebuje adresu, zamiast co piętnaście minut.
- Karta Jellyfin w Ustawieniach mówi, w jaki sposób kolektor dostaje informacje, a gdy połączenie jest włączone, ale niczego nie przenosi, wyjaśnia dlaczego.

## [1.3.0] - 2026-09-20

Sonarr, Radarr i Seerr na jednej stronie

### Added
- **Połączenia** w Ustawieniach, dla administratorów Jellyfin: **Sonarr**, **Radarr** i **Seerr** (lub Jellyseerr/Overseerr), po kilka instancji każdego rodzaju, każda sprawdzana przed zapisaniem. Klient pobierania nie wymaga osobnej konfiguracji, bo Sonarr i Radarr już z nim rozmawiają, a FinStats czyta to, co one wiedzą.
- **Prośby**: kto o co poprosił, ile trwało, zanim to dotarło, i czy w ogóle zostało obejrzane. Kafelki z oczekującymi tytułami i typowym czasem oczekiwania, wykres zmian tego czasu miesiąc po miesiącu oraz listy tego, co dotarło tygodnie temu i nigdy nie zostało odtworzone, i tego, kto prosi najczęściej.
- **Nadchodzące**: nowe odcinki i premiery filmów z Sonarr i Radarr jako terminarz dzień po dniu, z zaznaczeniem, czy *ty* oglądasz dany serial, a dla osób, które mogą widzieć wszystkich, kto jeszcze go ogląda. Serial liczy się jako oglądany, gdy ktoś odtworzył jego odcinek w ciągu ostatnich czterech miesięcy. Rząd „Wkrótce” na Pulpicie, po jednym na każdym profilu z serialami, które ogląda dana osoba, oraz to, co nadchodzi dla tytułu, na jego własnej stronie.
- **Pobieranie**: kolejka na żywo każdego Sonarr i Radarr (co to jest, jak daleko zaszło, jak szybko idzie, co poszło nie tak przy imporcie), z osobą, która o to poprosiła, obok. Paczka sezonu to jeden wiersz bez względu na liczbę odcinków, a to, co wymaga uwagi, jest na górze. Poniżej to, co przyszło w ciągu tygodnia, miesiąca lub roku według indeksera, jakości i klienta pobierania, oraz które pobrania się nie powiodły. Pulpit pokazuje tę samą listę w skróconej formie.
- **Widzi pobierane pliki**, nowe uprawnienie. Bez niego użytkownicy nadal widzą, jak daleko zaszła ich *własna* prośba i ile zostało, i nic więcej: żadnych nazw wydań, prędkości ani innych pobrań. Cudze prośby wymagają uprawnienia *Widzi aktywność wszystkich*.

### Changed
- FinStats wysyła do tych usług `GET` i nic więcej: nie ma w nim kodu, który mógłby zatwierdzić prośbę, uruchomić wyszukiwanie albo dodać, wstrzymać lub usunąć pobieranie.
- Ich klucze API są przechowywane we własnej bazie danych FinStats i nigdy nie są odsyłane do przeglądarki, zapisywane w dzienniku ani umieszczane w kopii zapasowej. FinStats nie wykonuje żadnych przekierowań, więc klucz nie może trafić tam, czego nie wpisano, a certyfikaty są weryfikowane, chyba że wyłączy się to dla danego połączenia.

## [1.2.2] - 2026-09-20

### Added
- Języki dźwięku i napisów na każdym filmie i odcinku („Dźwięk: japoński · angielski”).
- Jak daleko sięga dubbing: serial lub sezon mówi, ile jego odcinków ma każdy język („angielski: 13 z 26 odcinków”), a lista odcinków ma kolumnę **Dźwięk**.
- **Języki ścieżek dźwiękowych** i **Języki napisów** na stronach bibliotek: ile plików można odtworzyć w każdym języku.
- Po tej aktualizacji FinStats raz ponownie odczyta bibliotekę, aby pobrać języki. Nigdy nie pisze „z dubbingiem”, bo Jellyfin nie podaje oryginalnego języka tytułu, a ścieżka bez znacznika języka jest wymieniana jako „Nieznany”.

## [1.2.1] - 2026-09-20

### Changed
- Zdjęcia w README zostały wykonane od nowa w tej wersji, z nowym zdjęciem strony **Bezpieczeństwo**. Nic w aplikacji się nie zmienia.

## [1.2.0] - 2026-09-20

Skąd ludzie oglądają, na mapie

### Added
- **Strona Bezpieczeństwo**, dla osób, które mogą widzieć szczegóły sieci i aktywność wszystkich: mapa świata z kropką w każdym miejscu, z którego oglądają twoi użytkownicy, o rozmiarze zależnym od tego, ile się tam dzieje. Dom jest zielony, miejsca poza domem fioletowe, strumienie na żywo pulsują, a nieudane logowania z zewnątrz są czerwone. Przeciągnij, by przesunąć, powiększaj przyciskami, klawiszami albo Ctrl + kółkiem, a nakładające się kropki łączą się, dopóki się ich nie przybliży. Pod mapą każde miejsce z jego osobami, odtworzeniami, czasem oglądania i logowaniami oraz kraje według liczby odtworzeń.
- **Niemożliwa podróż**: alert, gdy jedno konto jest widziane w dwóch miejscach, których nie łączy żaden lot, albo w dwóch odległych miejscach jednocześnie, z obiema obserwacjami, odległością, czasem między nimi i prędkością, jakiej by to wymagało. **Pokaż na mapie** rysuje trasę.
- **Nowy kraj**: pierwszy raz, gdy ktoś odtwarza lub loguje się z kraju, w którym wcześniej go nie widziano.
- Rozwiąż alert z notatką albo wycisz parę miejsc dla danej osoby, na przykład dla VPN lub telefonu, którego operator ma siedzibę w stolicy. Alerty znalezione w starej historii, po imporcie lub na świeżej bazie danych są zapisywane jako rozwiązane, zamiast zalewać listę. Alerty i decyzje w ich sprawie są częścią kopii zapasowych.
- Miejsca pochodzą z bazy miast czytanej lokalnie: żaden adres nigdy nie jest nigdzie wysyłany, a mapa jest rysowana z konturów dołączonych do FinStats, a nie przez usługę map. Pobierz darmową bazę DB-IP ze strony Bezpieczeństwo lub z **Ustawień → Bezpieczeństwo**, pozwól FinStats odświeżać ją co miesiąc (wyłączone, dopóki tego nie włączysz) albo wrzuć własny plik `.mmdb` (DB-IP, MaxMind GeoLite2-City) do `data/geoip/`. `FINSTATS_GEOIP_DB` wskazuje plik w innym miejscu.
- **Ustawienia → Bezpieczeństwo**: używana baza danych, comiesięczna aktualizacja oraz jak szybko (900 km/h) i jak daleko od siebie (500 km) muszą być dwie obserwacje, by liczyły się jako niemożliwa podróż. Bazy miast często mylą się o kilkaset kilometrów i właśnie do tego służy próg odległości.

## [1.1.3] - 2026-09-20

### Added
- Ctrl+Spacja przeszukuje też obsadę i twórców wszystkiego w bibliotece, w grupie **Obsada i twórcy**, ze zdjęciem, informacją, czy dana osoba gra, czy reżyseruje, i w ilu twoich tytułach występuje; wybór osoby otwiera jej stronę. Wyszukiwanie jest tak samo wyrozumiałe jak wyszukiwanie tytułów (słowa w dowolnej kolejności, znaki diakrytyczne pomijane, literówka dozwolona), a gdy kilka osób pasuje równie dobrze, pierwsza jest ta, która występuje w większej liczbie twoich tytułów.

## [1.1.2] - 2026-09-20

### Added
- **Ostatnio dodane** na Pulpicie, nad wykresem Aktywność: 30 najnowszych pozycji jako rząd plakatów przewijany w bok, ze strzałkami na komputerze. Każda mówi, kiedy się pojawiła i czym jest (film, album albo „Sezon 4 · 3 odcinki”), a pojedynczy nowy odcinek mówi, który to, z plakatem sezonu, jeśli taki istnieje.
- **Shift + kółko myszy** i klawisze strzałek przesuwają o jeden plakat, Page Up i Page Down o ekran, Home i End przenoszą na jeden lub drugi koniec, działa też przesuwanie palcem. Samo kółko nadal przewija stronę.
- Nowe odcinki tego samego serialu, które pojawiły się tego samego dnia, mają jeden wspólny wpis, a cały serial dodany naraz to jeden wpis („3 sezony · 60 odcinków”), więc jeden duży import nie wypycha z listy wszystkiego innego.
- Rząd ignoruje zakres czasu i osobę wybrane na Pulpicie, bo dotyczy biblioteki, a nie odtworzeń, i ukrywa się, gdy biblioteka jest pusta.

## [1.1.1] - 2026-09-20

### Added
- Strony otwierają się od razu: chwilę po wczytaniu FinStats po cichu pobiera to, co prawdopodobnie otworzysz za chwilę (Pulpit, twój profil i oś czasu, Aktywność, Użytkowników, Biblioteki i każdą bibliotekę, Odtwarzanie, strony serwera, podsumowanie i najaktywniejsze osoby), więc otwarta strona pojawia się natychmiast, a i tak odświeża się w tle.
- Filmy, seriale i członkowie obsady są pobierani tylko na sygnał zamiaru, bo są ich tysiące: zatrzymanie wskaźnika na łączu, dotknięcie go lub dotarcie do niego klawiaturą pobiera tę jedną stronę.
- Pobieranie z wyprzedzeniem czeka, aż otwarta strona skończy się wczytywać, pobiera jedną stronę naraz, gdy przeglądarka jest bezczynna, wstrzymuje się w karcie w tle, nic nie robi przy oszczędzaniu danych lub bardzo wolnym połączeniu i jest zapominane po wylogowaniu.
- Małe łącze **Repozytorium** ze znakiem GitHub na pasku stanu, obok wersji.

### Fixed
- **Oś czasu** zapominała, które biblioteki wyłączono, gdy stronę przeładowano lub otwarto z zakładki, choć wybór był zapisany w adresie.

## [1.1.0] - 2026-09-20

Oś czasu wszystkiego, co obejrzano

### Added
- **Oś czasu**, obok **Przeglądu** na każdym profilu: twoje oglądanie jako jeden szlak od dziś do pierwszego odtworzenia, jakie zna FinStats, z wieczorem odcinków zebranym w jeden przystanek. Każdy przystanek to plakat z tym, co i kiedy obejrzano („Sezon 2 · Odcinki 3–6”, film z informacją, czy zajął więcej niż jedno posiedzenie, albo album z liczbą utworów), i otwiera tytuł.
- Szlak dopasowuje się do szerokości okna: trzy przystanki w rzędzie na szerokim ekranie, dwa na węższym, co drugi rząd biegnie wstecz, a łuk łączy go z następnym, aż do jednej prostej linii na telefonie. Pierwszy przystanek każdego miesiąca nosi nazwę miesiąca.
- Starsze przystanki wczytują się same podczas przewijania, aż do początku historii.
- Przełączniki **Biblioteki** nad szlakiem pomijają to, czego nie chcesz widzieć, na przykład muzykę, a wybór jest częścią adresu.
- To, co nadal jest odtwarzane, ma napis **Odtwarzane teraz**.
- Każdy widzi własną oś czasu; cudze wymagają uprawnienia „Widzi aktywność wszystkich”.

## [1.0.5] - 2026-09-20

### Added
- Ochrona przed cofnięciem wersji: baza danych pamięta najnowszą wersję FinStats, która ją otworzyła, a starsza wersja odmawia startu na niej, zamiast po cichu pracować na danych, których nie rozumie w pełni. Komunikat mówi, co zrobić. Wersje do 1.0.4 włącznie zostały wydane, zanim ta kontrola powstała.

## [1.0.4] - 2026-09-20

### Fixed
- W szczegółach odtworzenia przycisk kopiowania obok długiego **ID urządzenia** spadał do osobnego wiersza.
- Na stronie Odtwarzanie ostatnia kolumna tabeli **Które klienty transkodują** była ucięta („Transc…”); dwuwyrazowe nagłówki teraz się zawijają.

### Changed
- Zrzuty ekranu w README są znów aktualne; wciąż pokazywały wersję 0.5.0.

## [1.0.3] - 2026-09-20

### Added
- Strona odcinka pokazuje własny obraz odcinka, ten sam, który Jellyfin pokazuje na swojej liście odcinków, zamiast plakatu serialu. Odcinek, dla którego Jellyfin nie ma obrazu, zachowuje plakat.

## [1.0.2] - 2026-09-20

### Fixed
- Na **Liście zmian** grupy v0.9 i v0.10 nie miały nagłówka obok numeru wersji. Nagłówek grupy to teraz zawsze pierwsze zdanie opisu jej pierwszego wydania, więc długi wstęp nie jest ucinany w pół zdania.

## [1.0.1] - 2026-09-20

### Fixed
- Opublikowany obraz nie startował na świeżej maszynie: „unable to open database file: /data/finstats.db”, po półminutowym zawieszeniu. Docker tworzy brakujący folder `data` jako root, a FinStats działa jako zwykły użytkownik. Kontener przejmuje teraz folder na własność, a potem przełącza się na tego użytkownika, więc `docker run` działa za pierwszym razem. Dla plików, których właścicielem jest ktoś inny niż użytkownik 1000, ustaw `PUID` i `PGID`; `--user` działa jak wcześniej.
- Folder danych, do którego naprawdę nie da się zapisywać, jest zgłaszany od razu, z poleceniem, które to naprawia, zamiast po 30 sekundach jako błąd bazy danych.

## [1.0.0] - 2026-09-20

Gotowy dla wszystkich

### Added
- Gotowy obraz, teraz pod `ghcr.io/finstats/finstats`, dla 64-bitowych maszyn Intel/AMD i ARM (działa Raspberry Pi 4 lub 5): `:latest` to najnowsze wydanie, tag wersji pozostaje tam, gdzie jest, a `:edge` to wersja rozwojowa.
- Każde wydanie pojawia się na stronie wydań projektu na GitHub razem z tą listą zmian.

### Changed
- Numery wersji znaczą teraz to, co mówią: aktualizacje 1.x nie zepsują twoich danych, kopii zapasowych ani ustawień.
- Plik Docker Compose i wszystkie instrukcje używają opublikowanego obrazu. Przy samodzielnie zbudowanym `finstats:latest` wystarczy zmienić nazwę obrazu; folder `data` przechodzi bez zmian.

## [0.10.2] - 2026-09-20

### Fixed
- Na telefonie Ustawienia i strona serialu mogły przewijać się w bok: etykieta przeznaczona tylko dla czytników ekranu wychodziła poza swoją tabelę, a listy odcinków i rząd obsady robiły to samo. Teraz przewijają się wewnątrz siebie.
- Podsumowanie nie przewija się już w bok, gdy słowo w tle rozdziału jest szersze niż strona.
- Długi wiersz „ogląda na…” pod **Teraz odtwarzane** kończy się wielokropkiem, zamiast być uciętym w połowie litery.
- Plakat przed każdym serialem w profilu był dla czytników ekranu drugim, bezimiennym łączem; teraz jest przed nimi ukryty, a łączem jest tytuł obok.

## [0.10.1] - 2026-09-20

### Fixed
- Pobieranie kopii zapasowej kończyło się w przeglądarce niepowodzeniem („the source file could not be read” w przeglądarce Firefox), a strona niczego nie wyjaśniała: FinStats kompresował drugi raz kopię zapasową, która już jest skompresowana, a podwójnie spakowany transfer urywał się przed końcem. Kopie zapasowe są wysyłane w takiej postaci, w jakiej są, z rozmiarem podanym z góry, więc przeglądarka może pokazać prawdziwy postęp.

## [0.10.0] - 2026-09-20

Kopie zapasowe i przenosiny na nową instalację

### Added
- Kopia zapasowa co tydzień, z zachowaniem pięciu najnowszych: każde odtworzenie z osią czasu pauz i przewinięć, oznaczenia obejrzenia, uprawnienia, adresy domowe, dziennik serwera i ustawienia, w jednym małym pliku, kilkaset KB na tysiące odtworzeń. **Ustawienia → Kopie zapasowe** wymieniają je, każdą z przyciskiem **Pobierz**, tworzą kopię na żądanie i pozwalają ustawić, jak często je robić i ile zachowywać. Leżą w folderze `backups` w katalogu danych. Kopia zapasowa nie zawiera klucza API Jellyfin ani sesji logowania, ale jest kompletną historią oglądania z adresami IP, więc trzymaj ją w prywatnym miejscu.
- Przywracanie scala, a nie nadpisuje: odtworzenia, które już są, są pomijane, więc można je bezpiecznie wykonać dwa razy albo na instancji, która działa już od jakiegoś czasu. Odznacz „Przywróć także ustawienia i uprawnienia”, by przywrócić tylko historię. Kopie zapasowe działają między wersjami w obu kierunkach. Z terminala: `finstats backup` i `finstats restore <file>`.

### Changed
- Zbieranie danych sprawdza co sekundę, gdy ktoś ogląda, i co 5 sekund, gdy nikt nie ogląda, podczas gdy wcześniej przez cały czas sprawdzało co 5 sekund, więc pauzy, przewinięcia i zmiany ścieżek są rejestrowane z dokładnością do sekundy. Oba ustawienia są w **Ustawieniach → Zbieranie danych**; zmieniony wcześniej stary interwał trzeba ustawić ponownie.

### Fixed
- Adres IP w szczegółach odtworzenia łamał się na dwa wiersze w środku adresu. Oznaczenie Lokalnie/Zdalnie przenosi się pod spód, gdy brakuje miejsca.

## [0.9.1] - 2026-09-20

### Fixed
- Osoby w domu były pokazywane jako zdalne. Urządzenie we własnej sieci, które łączy się z Jellyfin przez jego publiczną nazwę, na przykład przez odwrotne proxy lub domenę, przychodzi z publicznym adresem IP gospodarstwa domowego, a za lokalne liczyły się tylko adresy prywatne. FinStats poznaje teraz adres publiczny i liczy takie odtworzenia jako lokalne w całej historii, a do tego pamięta wcześniejsze adresy.

### Added
- **Ustawienia → Sieć domowa**: adresy, które FinStats traktuje jako dom, z możliwością dodania własnych (wcześniejszy adres, drugi dom, wyjście VPN) albo wyłączenia sprawdzania. Sprawdzanie co 15 minut pyta prostą usługę „jaki mam IP”, próbując kilku (Amazon, Cloudflare i inne), bo DNS blokujący reklamy, taki jak Pi-hole, często je blokuje, a jedna z nich w ogóle nie potrzebuje DNS. Żądanie nie zawiera nic o tobie ani o twoim serwerze, a po wyłączeniu przełącznika FinStats nie rozmawia z niczym poza Jellyfin. `FINSTATS_PUBLIC_IP_URL` ustawia własną usługę.

## [0.9.0] - 2026-09-20

Każda tabela w kolejności, jakiej chcesz

### Added
- Każdą tabelę da się sortować: kliknij nagłówek, by posortować według niego, drugi raz, by odwrócić kolejność, a trzeci raz, by wrócić do pierwotnej. Znaczenie komórek jest rozpoznawane, więc „3 d. 2 godz.” sortuje się jako czas, a „1,4 GB” jako rozmiar, a puste komórki trafiają na koniec. Dotyczy to listy użytkowników, widzów, odcinków, urządzeń, wtyczek, zaplanowanych zadań, widoku tabeli każdego wykresu, macierzy uprawnień oraz list słupkowych (kodeki, rozdzielczości, klienty i reszta), które dostają własny wąski nagłówek.
- Aktywność i Dziennik serwera sortują wszystkie swoje strony, a nie tylko wiersze na ekranie, a kolejność jest zapisywana w adresie, więc przetrwa przeładowanie albo udostępniony link.
- Tabele mające dziesięć lub więcej wierszy dostają nad sobą małe pole filtra.

## [0.8.2] - 2026-09-20

### Changed
- Wybór roku w podsumowaniu to ten sam przełącznik co każdy inny wybór zakresu w FinStats (7 dni · 30 dni · 90 dni …).

## [0.8.1] - 2026-09-19

### Added
- Strony aktorów i reżyserów, otwierane z „Najczęściej oglądanych osób” w podsumowaniu albo z nowego rzędu **Obsada i twórcy** na stronie każdego filmu, serialu lub odcinka: wszystko, w czym występują na twoim serwerze, ile z tego obejrzano i kto, oraz co jeszcze czeka. Strona podąża za wybranym zakresem czasu, osoby, które mogą widzieć tylko własne statystyki, widzą tam tylko własne oglądanie, a Esc zabiera z powrotem.

## [0.8.0] - 2026-09-19

Podsumowanie ma nowy wygląd i więcej do powiedzenia

### Added
- **Najczęściej oglądane osoby**: aktorzy, z którymi spędzono najwięcej czasu, i reżyserzy tego, co oglądano, jako rząd portretów. Obsada i twórcy są odczytywani razem z biblioteką i pojawiają się po pierwszym odczycie biblioteki po tej aktualizacji.
- **Twój rok w dniach**: każdy dzień roku jako jeden kwadrat, tym jaśniejszy, im więcej oglądania, z najdłuższą serią i najbardziej intensywnym dniem nad nim. Najedź na dzień albo przejdź po kalendarzu klawiszami strzałek, by zobaczyć, co w nim było.
- **W liczbach**: odtworzenia, czas oglądania, różne tytuły, dni z oglądaniem, najdłuższa seria i ile było **powtórek**, a do tego podział roku na odcinki, filmy i muzykę.
- Rozdział o gatunkach mówi, ilu gatunków się dotknęło i jak duży udział miał najpopularniejszy.

### Changed
- Nowy wygląd: otwiera się na twoim roku „jeszcze raz”, z nagłówkiem obok plakatów, które go wypełniły, nad wykresem fali całego roku z jednym słupkiem na tydzień i podświetlonym najgłośniejszym, a rok wybiera się tuż nad nim. Każdy rozdział ma nagłówek z kluczowym słowem w kolorze fioletowym, zdanie niosące liczby i słowo rozdziału stojące duże i blade w tle. Kończy się tak jak filmy, napisami końcowymi: w rolach głównych, reżyseria, wyświetlane na, czas trwania.
- Godziny, dni tygodnia i miesiące dzielą jeden rozdział **Wzorce aktywności** z przełącznikiem między nimi, a nagłówek podąża za tym, na co patrzysz („Piątek na czele”).

### Fixed
- **Teraz odtwarzane** na Pulpicie było puste, gdy nikt nie oglądał, więc nagłówek wyglądał, jakby należał do filtrów pod nim. Pole „Teraz nic nie jest odtwarzane” wróciło.

## [0.7.10] - 2026-09-19

### Added
- Grafika bibliotek: każda karta biblioteki i własna strona biblioteki pokazują obraz, którego Jellyfin dla niej używa, przez pośrednika obrazów FinStats, tak jak plakaty. Biblioteka bez obrazu zachowuje ikonę.

## [0.7.9] - 2026-09-19

### Changed
- Karta Gatunki znów otwiera się jako **lista**, z radarem jako drugą opcją przełącznika. Dotychczasowy wybór jest zachowywany.

## [0.7.8] - 2026-09-19

### Added
- Widok radaru dla karty Gatunki, na Pulpicie i w profilach: jedno ramię na gatunek, przełączane między **Radar** i **Lista** w rogu karty, a wybór jest zapamiętywany. Najechanie na punkt albo użycie klawiszy strzałek pokazuje czas oglądania i udział tego gatunku.

## [0.7.7] - 2026-09-19

### Fixed
- Filmy należące do kolekcji były pokazywane jako „Nie ma już w bibliotece”. Gdy włączone jest *grupowanie filmów w kolekcje*, Jellyfin zwraca kolekcję zamiast filmów, które się w niej znajdują, więc FinStats nigdy ich nie widział; na jednym serwerze ukryło to 556 z 1238 pozycji w bibliotece Filmy. FinStats prosi teraz o same filmy, a odczyt uruchamiany przez tę aktualizację przywraca je i ponownie łączy ich odtworzenia.

## [0.7.6] - 2026-09-19

### Changed
- W profilach karta **Seriale** przeniosła się niżej, tuż nad Gatunki, więc długa lista seriali nie wypycha już z widoku czasu oglądania, wykresu aktywności i najpopularniejszych tytułów. Serie dni zostają na górze, a otwarty serial pozostaje otwarty po zmianie zakresu czasu.

## [0.7.5] - 2026-09-19

### Changed
- Wyszukiwanie dopasowuje słowo po słowie, w dowolnej kolejności, z czymkolwiek pomiędzy, zamiast wymagać dokładnej frazy, więc „alya hides” znajduje *Alya Sometimes Hides Her Feelings in Russian*. Wielkość liter, znaki diakrytyczne i interpunkcja nie mają znaczenia, początkowe „The” jest pomijane, a literówka jest wybaczana („mentalsit” znajduje *The Mentalist*); słowa mające trzy litery lub mniej nadal trzeba wpisać poprawnie. Lepsze dopasowania są wyżej, a muzykę można też znaleźć po wykonawcy.
- Pola wyszukiwania na stronach Aktywność i Dziennik serwera dopasowują słowo po słowie we wszystkich kolumnach: „alya opera” znajduje odtworzenia tego serialu w przeglądarce Opera.

## [0.7.4] - 2026-09-19

### Changed
- Lista zmian jest pogrupowana według serii: każda seria `0.x` to jedna zwijana grupa z jej wydaniami, opisem tego, czego dotyczyła seria, liczbą wydań i datami. Na start otwarta jest tylko najnowsza, a zwinięta grupa nadal pokazuje, którą wersję masz uruchomioną.

## [0.7.3] - 2026-09-19

### Fixed
- Zegar w Teraz odtwarzane zacinał się co kilka sekund: 5-sekundowe odświeżanie też go przerysowywało, więc gdy serwer był odrobinę do przodu, liczba przeskakiwała za wcześnie, a potem stała przez sekundę. Teraz porusza go tylko tyknięcie raz na sekundę, dokładnie o jedną sekundę (zmierzone odstępy od 999 do 1001 ms), a odświeżanie nadal koryguje go po pauzie, przewinięciu lub prawdziwym rozjechaniu się, trafiając w tyknięcie.

## [0.7.2] - 2026-09-19

### Added
- Wspólne oglądanie na żywo: każda karta Teraz odtwarzane mówi „Razem z: maria”, gdy osoby oglądają razem to samo, podczas gdy wcześniej grupa była rozpoznawana dopiero po zakończeniu odtworzeń. Na żywo osoby liczą się jako oglądające razem, gdy są przy tym samym tytule i albo zaczęły w obrębie okna grupy, albo są prawie w tym samym miejscu, co działa nawet wtedy, gdy FinStats został zrestartowany w trakcie strumienia.

## [0.7.1] - 2026-09-19

### Changed
- Teraz odtwarzane liczy każdą sekundę: zegar tyka, a pasek płynnie przesuwa się pomiędzy, zamiast skakać przy każdym zapytaniu serwera. Serwer nadal jest pytany co 5 sekund i pozostaje źródłem prawdy: wyświetlanie synchronizuje się ponownie po pauzie, przewinięciu albo gdy serwer jest do przodu, ale nie z powodu pozycji nieaktualnej o zaledwie kilka sekund, bo klienty zgłaszają się tylko mniej więcej co dziesięć sekund. Wstrzymane strumienie stoją w miejscu.

## [0.7.0] - 2026-09-19

Kto ogląda razem

### Added
- Wspólne oglądanie: gdy różne osoby zaczynają ten sam tytuł w tym samym czasie, FinStats liczy to jako oglądanie razem. Karta **Oglądane razem** na Pulpicie pokazuje grupy, wspólne tytuły i godziny spędzone razem, każdy profil pokazuje, z kim dana osoba ogląda najczęściej, wspólne odtworzenia mają na stronie Aktywność mały znaczek osób, a szczegóły odtworzenia mówią, z kim je oglądano. Działa na całej historii, łącznie z zaimportowanymi odtworzeniami.
- Ustawienie określające, jak blisko siebie muszą być początki, domyślnie 60 sekund: mniej więcej jedna trzecia prawdziwych wspólnych sesji zaczyna się w odstępie od 6 do 60 sekund. Osoby muszą też przez kilka minut oglądać równolegle, więc dwie osoby, które przypadkiem otworzyły ten sam odcinek, nie tworzą grupy.

### Changed
- Wyszukiwanie otwiera się skrótem **Ctrl + Spacja** zamiast Ctrl/⌘ + K.

## [0.6.1] - 2026-09-19

### Added
- Administratorzy mogą otworzyć podsumowanie innej osoby, na górze strony Podsumowanie albo przyciskiem **Otwórz podsumowanie** w jej profilu. Podsumowanie mówi wtedy o tej osobie z imienia zamiast „ty”.

### Changed
- Wszyscy pozostali nadal widzą tylko własne podsumowanie, bez względu na posiadane uprawnienia, i nadal nie ma podsumowania całego serwera.

## [0.6.0] - 2026-09-19

Twój profil i to, gdzie jesteś w każdym serialu

### Added
- Postęp seriali w profilach: każdy rozpoczęty serial jako pasek z jednym segmentem na odcinek (obejrzany, zaczęty albo jeszcze nie), rozwijany sezon po sezonie, posortowany według tego, jak blisko końca jesteś, z osobnymi widokami ukończonych seriali i wszystkiego.
- **Oznacz jako obejrzane** we własnym profilu, dla odcinka, sezonu lub serialu, dla tego, co oglądano, gdy nic tego nie rejestrowało. Uwzględniane są też własne znaczniki obejrzenia z Jellyfin. Oznaczenia zostają w FinStats i nigdy niczego nie zmieniają w Jellyfin; zarejestrowanego odtworzenia nie da się odznaczyć, własne oznaczenia tak.
- Serie dni w profilach: najdłuższy ciąg kolejnych dni z odtworzeniem, bieżąca seria i liczba dni, w które coś oglądano.
- **Mój profil** na pasku bocznym.
- Aktywność pokazuje, gdzie zatrzymało się odtwarzanie (na przykład `44:15 / 45:00`) pod paskiem postępu, a datę i godzinę pod „3 godz. temu”. Odtworzenia zaimportowane z Jellystat nie mają pozycji zatrzymania, bo Jellystat nigdy jej nie zapisywał.

### Changed
- Odcinki, których brakuje albo które jeszcze nie zostały wyemitowane, nie są już traktowane jako część biblioteki. Jellyfin wymienia je jako pozycje zastępcze bez pliku, a wcześniej liczyły się do sum odcinków.

## [0.5.1] - 2026-09-19

### Added
- Logo: przycisk odtwarzania pocięty na trzy słupki wykresu, na pasku bocznym i na ekranach logowania, jako ostry SVG z wielorozmiarowym `favicon.ico` dla przeglądarek i pasków zakładek, które go potrzebują, oraz ikona ekranu głównego dla telefonów i tabletów.

## [0.5.0] - 2026-09-19

Ty decydujesz, kto co widzi

### Added
- Uprawnienia w Ustawieniach → Dostęp, dla wszystkich naraz albo osobno dla każdej osoby: **Widzi aktywność wszystkich** (statystyki i historia innych osób, strona Użytkownicy, każdy strumień na żywo), **Widzi szczegóły sieci** (adresy IP, identyfikatory urządzeń, lokalnie czy zdalnie), **Widzi serwer** (strona Serwer, dziennik serwera, nieudane logowania, ścieżki plików) i **Zarządza FinStats** (ustawienia, zadania, import, usuwanie odtworzeń).
- Logowanie dla poszczególnych osób, by wpuścić kilka osób bez otwierania FinStats dla każdego użytkownika Jellyfin.
- Zmiany działają natychmiast, także odbieranie dostępu, a wszystko jest egzekwowane przez serwer, a nie przez ukrywanie przycisków.

### Changed
- Administratorzy Jellyfin nadal mają wszystko i tylko oni mogą zmieniać uprawnienia; ktoś, kto może zarządzać FinStats, nie może niczego przyznać ani sobie, ani nikomu innemu.
- Podsumowanie roku pozostaje osobiste bez względu na przyznane uprawnienia.
- W istniejących instalacjach nic się nie zmienia, dopóki czegoś nie przyznasz.

## [0.4.5] - 2026-09-19

### Changed
- FinStats jest udostępniany na licencji **GNU General Public License v3.0**. Żadna wcześniejsza wersja nigdy nie została opublikowana, więc nic nie zostało wydane na innych warunkach. Licencje dołączonych czcionek są dostarczane razem z nimi.

## [0.4.4] - 2026-09-19

### Added
- Esc cofa: na otwartej bibliotece, użytkowniku lub tytule wraca do miejsca, z którego tam trafiono, z listą dokładnie taką, jaką ją zostawiono. Wszystko, co otwarte, zamyka się najpierw (szczegóły odtworzenia, wyszukiwanie, lista rozwijana, menu mobilne), a Esc nic nie robi, gdy coś piszesz albo obsługujesz wykres klawiaturą.

## [0.4.3] - 2026-09-19

### Changed
- FinStats pamięta, co każda strona pokazała ostatnio: powrót do profilu, biblioteki lub zakresu czasu, które już oglądano, rysuje je natychmiast, łącznie ze zdjęciem profilowym, podczas gdy świeże liczby wczytują się w tle, a strona przerysowuje się tylko wtedy, gdy coś się zmieniło. To, co zapamiętane, żyje w karcie przeglądarki i jest usuwane po wylogowaniu albo po jakiejkolwiek zmianie.
- Wskaźniki wczytywania pojawiają się tylko wtedy, gdy wczytywanie faktycznie trwa długo. Wcześniej każda strona pokazywała je przez co najmniej jedną trzecią sekundy.

### Fixed
- Biblioteki usunięte w Jellyfin, które nigdy nie miały odtworzenia, nie są już wymieniane. Usunięta biblioteka z historią jest nadal pokazywana, oznaczona jako usunięta.

## [0.4.2] - 2026-09-19

### Fixed
- Na stronie Użytkownicy znacznik **Administrator** stał w osobnym wierszu i sprawiał, że ten wiersz był wyższy od pozostałych. Znaczniki stoją teraz obok nazwy.

## [0.4.1] - 2026-09-19

### Fixed
- Słowo „null” nie pojawia się już pod tytułem Pulpitu poza grudniem i styczniem, gdy nie ma banera podsumowania do pokazania.

## [0.4.0] - 2026-09-19

Zobacz, co się zmieniło, bez wychodzenia z aplikacji

### Added
- Karta **Lista zmian** pokazująca każde wydanie, z oznaczeniem uruchomionej wersji.
- Wersja na pasku stanu prowadzi do niej, a karta pokazuje kropkę po aktualizacji, dopóki się do niej nie zajrzy.

### Changed
- Ponowne łączenie historii z pozycjami o zmienionych nazwach jest znacznie szybsze: z 18 sekund do jednej trzeciej sekundy przy 666 osieroconych tytułach, więc start pozostaje natychmiastowy, choć historia rośnie.

## [0.3.3] - 2026-09-19

### Changed
- Biblioteka jest odczytywana ponownie zaraz po zakończeniu zadania Jellyfin **Scan Media Library**, nigdy według osobnego zegara i nigdy w trakcie skanowania. FinStats zawsze tylko odczytywał dane z Jellyfin i nigdy nie uruchamia tam skanowania.
- Cotygodniowy odczyt zapasowy obejmuje serwery, które polegają na monitorowaniu w czasie rzeczywistym, gdzie zadanie skanowania może nigdy się nie uruchamiać.
- Stary interwał jest teraz tylko zapasowy, na wypadek gdy śledzenie skanowania jest wyłączone albo serwer nie zgłasza zadania skanowania.

### Added
- Przełącznik **Podążaj za skanowaniem biblioteki Jellyfin** w Ustawieniach → Zbieranie danych, domyślnie włączony.

## [0.3.2] - 2026-09-19

### Fixed
- Historia przetrwa zmiany nazw. Jellyfin nadaje plikowi o zmienionej nazwie nowy identyfikator, przez co jego wcześniejsze odtworzenia zostawały pod surową nazwą folderu (`Title (2010) [tmdbid-…] [imdbid-…]`), bez plakatu. Te odtworzenia są ponownie łączone z bieżącą pozycją biblioteki: po identyfikatorze TMDB/IMDb/TVDB, jeśli stara nazwa go zawiera, a w przeciwnym razie po oczyszczonym tytule i roku, a odcinki po serialu oraz numerze sezonu i odcinka. Dopasowanie musi być jednoznaczne, więc tytuły, które naprawdę usunięto, zostają takie, jakie są.
- Kanały telewizji na żywo zaimportowane z Jellystat były liczone jako filmy, bo Jellystat nie zapisuje typu pozycji. Teraz są rozpoznawane jako telewizja na żywo, a nowe odtworzenia telewizji na żywo są zapisywane z typem nadanym przez Jellyfin.

### Changed
- Telewizja na żywo jest całkowicie pomijana w podsumowaniu: w rankingach, sumach, personie, rekordach i pozycji w rankingu.

### Added
- `finstats relink` do ponownego łączenia historii na żądanie. Uruchamia się też przy starcie i po każdym odczycie biblioteki.

## [0.3.1] - 2026-09-19

### Changed
- Podsumowanie jest ściśle osobiste: każdy, łącznie z administratorami, widzi tylko własne, a widok „Wszyscy”, przełącznik użytkowników i rozdział o serwerze zniknęły.
- Podsumowanie otwiera się na roku, który jest „gotowy”: bieżącym w grudniu, a w pozostałych miesiącach poprzednim. Inne lata i „Ostatnie 12 miesięcy” są o jedno kliknięcie dalej.

## [0.3.0] - 2026-09-19

Twój rok w podsumowaniu

### Added
- Karta **Podsumowanie**, przewijana opowieść o roku: obejrzane godziny, najpopularniejsze seriale, filmy, utwory i gatunki oraz persona widza (nocny marek, ranny ptaszek, weekendowy widz, serialowy maratończyk, kinoman, meloman, wierny nawykom) z wykresami według godziny dnia i dnia tygodnia.
- Rok miesiąc po miesiącu, z tytułem, który określił każdy miesiąc.
- Rekordy: najbardziej intensywny dzień, największy maraton, najdłuższa seria dni, najdłuższe pojedyncze odtworzenie, najczęściej powtarzany tytuł, pierwsze odtworzenie roku i najstarszy obejrzany tytuł.
- Odkrycia: nowo rozpoczęte seriale, ukończone filmy i odcinki oraz seriale, które dostały dokładnie jeden odcinek.
- Jednowierszowy baner na Pulpicie w grudniu i styczniu, gdy podsumowanie jest gotowe.

## [0.2.0] - 2026-09-19

Więcej, niż Jellystat kiedykolwiek zapisał

### Added
- Oś czasu każdego odtworzenia: start, pauza, wznowienie, przewinięcie, zmiany ścieżki dźwiękowej i napisów, przejście z odtwarzania bezpośredniego w transkodowanie i zatrzymanie. Do tego liczba pauz i przewinięć, miejsce, od którego wznowiono odtwarzanie, i informacja, czy widz był w sieci lokalnej.
- Karta **Serwer**: wersja Jellyfin, oczekujące aktualizacje i restarty, zajętość dysku według biblioteki (Jellyfin 10.11+), wtyczki, wyniki zaplanowanych zadań i każde zarejestrowane urządzenie.
- Strona **Odtwarzanie** z analizami: jednoczesne strumienie w czasie, które klienty wymuszają transkodowanie, jak daleko ludzie docierają przed zatrzymaniem, odtworzenia lokalne wobec zdalnych i zachowania widzów.
- Z czego składa się twoja biblioteka: rozdzielczości, kodeki, zakres dynamiki, kontenery, tytuły według dekad, nowe pozycje według miesięcy, największe tytuły i tytuły, których nikt nigdy nie obejrzał, sprawdzane zarówno w historii FinStats, jak i we własnych znacznikach obejrzenia Jellyfin.
- Pulpit: szczyt jednoczesnych strumieni, szacowana ilość przesłanych danych, udział odtworzeń zdalnych, gatunki według czasu oglądania i nieudane logowania.
- Strony pozycji: łącza do IMDb i TMDB, wytwórnie, głębia bitowa i liczba klatek na sekundę oraz to, kto ma tytuł oznaczony w Jellyfin jako obejrzany.
- Strony użytkowników: gatunki oraz filmy, odcinki i ulubione, które Jellyfin ma w swoich zapisach.

### Fixed
- Etykieta czasu na karcie bieżącego odtwarzania nie wychodzi już poza kartę.
- Długie etykiety na stronie Odtwarzanie były ucinane zbyt wcześnie.
- Serial, którego nie ma już w bibliotece, nadal pokazuje listę sezonów i odcinków.

## [0.1.0] - 2026-09-19

Pierwsze wydanie

### Added
- Zbieranie danych na żywo z sesji Jellyfin: użytkownik, tytuł, klient, urządzenie, adres IP, sposób odtwarzania, powody transkodowania i akceleracja sprzętowa, szczegóły obrazu, dźwięku i napisów oraz czas oglądania wobec czasu pauzy.
- Logowanie kontem Jellyfin. Dwuetapowa konfiguracja tworzy własny klucz API FinStats, a hasła nigdy nie są przechowywane. Użytkownicy niebędący administratorami mogą opcjonalnie się logować, by widzieć tylko własne statystyki.
- Strony Pulpit, Aktywność, Użytkownicy, Biblioteki, Odtwarzanie i Dziennik serwera, paleta poleceń (Ctrl/⌘ K) i pasek stanu na żywo.
- Import z Jellystat: duże kopie zapasowe są strumieniowane na dysk i importowane w kilka sekund, a dwukrotny import tego samego pliku jest bezpieczny.
- Jeden mały plik wykonywalny z wbudowanym interfejsem webowym i jednym plikiem SQLite, a do tego obraz Docker.
