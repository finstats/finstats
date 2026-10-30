# Sürüm notları

FinStats’te değişen her şey, en yenisi en üstte. Bu dosya ikili dosyanın içine derlenir ve uygulamadaki
**Sürüm notları** sekmesinin gösterdiği de budur.

Biçim: `## [version] - date` ya da birkaç günde yapılan bir sürüm için `## [version] - first day to release day`,
ardından her değişiklik için bir madde ile `### Added`, `### Changed`, `### Security`, `### Performance`, `### Stability`,
`### Fixed` ya da `### Removed`. Güvenlik, birinin kötüye kullanabileceği bir şeydir; Performans, daha hızlı ya da daha
hafif hale gelen bir şey; Kararlılık, artık çökmeyen, durmayan, sızıntı yapmayan ya da veri kaybetmeyen bir şey;
Düzeltildi ise yanlış yanıt veren bir şey. Bir `x.y.0` sürümü başlığının altında kısa bir başlık satırı taşır: uygulama
bu başlığı tüm serinin manşeti olarak kullanır.

## [2.3.1] - 2026-10-26 to 2026-10-28

### Security
- **Kurulum tek kullanımlık bir kod ister**: kurulana kadar FinStats başlarken günlüğüne bir kod yazar (`docker logs finstats`) ve sihirbaz, bir adresi denemeden ya da herhangi bir şeyi kurmadan önce bu kodu sorar. Aksi halde porta ilk ulaşan kişi FinStats’i kendine ait bir Jellyfin’e yönlendirip onun yöneticisi olabilir ya da denemeyi ağda dolaşmak için kullanabilirdi.
- **Kütüphane erişimi Jellyfin’i izler**: herkes yalnızca Jellyfin’in kendisine açtığı kütüphaneleri, bunlarda da yalnızca Jellyfin’in yaş sınırı, etiketleri ve derecelendirilmemiş içerik kuralları altında kendisine listelediği başlıkları görür; kütüphane sayfalarında, aramada, oyuncu kadrosunda, resimlerde, Son eklenenler’de, İzleme listesi’nde, Yakında’da, takvim akışında ve yeni başlık bildirimlerinde. Oturum açmış herkes her kütüphaneye göz atabiliyordu.
- Bir bağlantı, birini oturum açtıktan hemen sonra karşılama ekranından başka bir siteye gönderebiliyordu. Artık yalnızca FinStats’in kendi sayfalarına gidilir.
- Sunucu günlüğü ve içgörülerdeki başarısız oturum açma girişimleri, herkesin oturum açma adreslerini ve etkinliğini *Sunucuyu görme* iznine sahip herkese veriyordu. Diğer her yerde olduğu gibi, başka birinin satırları *Herkesin etkinliğini görme*, bir adres ise *Ağ ayrıntılarını görme* iznini gerektirir.
- Kişisel bir bildirim hedefi, ikinci kez sorgulandığında farklı yanıt veren bir ad üzerinden yerel ağa ulaşabiliyordu. Artık denetlenen adrese bağlanılır, internete hiç ulaşmayan daha fazla aralık yerel sayılır, evin kendi genel adresi de reddedilir ve sorgulanamayan bir ad, yerel bir adla aynı sözlerle reddedilir.
- Kendi `X-Forwarded-For` satırını ekleyen bir proxy’nin arkasında ya da bir IPv6 ağından, her oturum açma denemesi yeni bir adresten gelebiliyor ve sınır hiç uygulanmıyordu. Bir IPv6 /48 artık beş dakikada 50 deneme hakkı alır, dolu bir sınır eski bir adresi unutmak yerine yeni bir adresi reddeder, tüm sunucu dakikada 60 başarısız denemede durur ve başarısız bir oturum açma, adres ve pencere başına bir kez denetim kaydına yazılır.
- `FINSTATS_TRUST_PROXY`, `X-Forwarded-For`, `X-Forwarded-Host` ve `X-Forwarded-Proto` başlıklarına güvenilen proxy’leri (adresler ya da aralıklar) adlandırır; `1` ya da `true` artık yalnızca loopback’teki ya da özel bir ağdaki bir eşe güvenir. Açıkken her eş kendi adresini seçebiliyordu; bir proxy’nin arkasında kapalıyken on yanlış tahmin herkesi dışarıda bırakıyordu.
- Kullanıcılar okunduktan sonra bilinmeyen, devre dışı bırakılmış, kaldırılmış ya da FinStats’e alınmayan bir ad için oturum açma, Jellyfin’e sorulmadan yanlış parola olarak yanıtlanır, üstelik Jellyfin’in yanıt vereceğinden daha erken değil. Bu tür adlar Jellyfin hesaplarını kilitleyebiliyor, Jellyfin’in uzaktan erişim kuralını aşabiliyor ya da FinStats’in adresinin arkasına saklanabiliyordu.
- Jellyfin’in ev ağında tuttuğu biri (*Uzak bağlantılara izin ver* kapalı), FinStats’e yalnızca oradan oturum açar, çünkü Jellyfin bu ayarı FinStats’in kendi adresine göre değerlendiriyordu; birinin devre dışı olup olmadığını söylemeyen bir kullanıcı okuması da o kişiyi dışarıda tutar.
- Oturum açılan sayfa https üzerindeyse oturum çerezi, bunu belirtmeyen bir proxy’nin arkasında da Secure olarak işaretlenir.
- Başarılı bir oturum açma artık kendinden önceki tahminleri sıfırlamaz. https üzerinde oturum çerezi, komşu bir addaki bir sitenin değiştiremeyeceği ve başka hiçbir ad altında kabul edilmeyen bir `__Host-` çerezidir; oturumu kapatmak da tarayıcının gönderdiği her oturumu sonlandırır.
- Takvim akışını adresinden yalnızca bir takvim anahtarı açar; diğer her anahtar başlıkta gönderilir.
- Bir API anahtarı artık izin veremez, kimin girebileceğini ya da neyin herkese açık olduğunu değiştiremez, yedek geri yükleyemez, bir hizmeti ya da bildirim hedefini ekleyemez, değiştiremez veya deneyemez: bunların her biri anahtardan uzun yaşar, bu yüzden oturum açılmış bir tarayıcı gerektirir.
- Her yanıt `Cross-Origin-Opener-Policy: same-origin` ve bir `Permissions-Policy` taşır, API yanıtları `Cross-Origin-Resource-Policy: same-origin` taşır ve aksini söylemedikçe `no-store` olur, CSP de artık `blob:` resimlere izin vermez. FinStats’i açan bir sayfa, daha sonra onun sekmesini kendine ait bir oturum açma sayfasına çevirebiliyordu.
- FinStats’in hangi şemayla erişildiğini bildiği yerlerde (güvenilen bir proxy’nin `X-Forwarded-Proto` başlığı ya da genel adres), başka bir şemadan gelen yazma isteği siteler arası sayılıp reddedilir.
- `/auth/me`, bir başlığın sayfası ve Kütüphane sağlığı, Jellyfin’e genel adresinden bağlantı verir; FinStats’in Jellyfin’i kendisinin okuduğu adres yalnızca yöneticilere gider.
- Jellyfin’den yalnızca FinStats’te bulunan bir şey için resim alınır, yayımlanmış bir profilin posterleri ve kartları da buna dahildir; böylece başka bir kütüphanenin resimleri kimlikle istenemez.
- Özel bir torrent izleyicisinin anahtarları indirme kuyruğunun hatalarından da, adlarının yanı sıra biçimlerine göre de silinir; yalnızca *İndirilenleri görme* iznine sahip birine hangi hizmetin nasıl başarısız olduğu söylenir, iç adresi ya da nereye yönlendirdiği asla söylenmez.
- Sunucu sayfası bir cihazın son kullanıcısını yalnızca *Herkesin etkinliğini görme* izniyle gösterir; Bağlanmamış medya da *FinStats’i yönetme* izninin yanı sıra bunu gerektirir. Bu izin olmadan Görevler bölümü yalnızca kişinin kendi oynatmalarını ve akışlarını sayar, kaldırılmış bir kütüphane kişinin kendi oynatmalarına göre listelenir ve bekleyen bir Tautulli panosu iptal edilemez.
- Ayarlar genel adres hizmetini yalnızca sunucu adıyla, Jellyfin’in adresini ve veri klasörünü yalnızca yöneticilere gösterir ve kütüphanenin kaybettiği bir başlık yalnızca okuyanın görebileceği oynatmalardan tanımlanır.
- Tabloları düz tablo olmayan bir Tautulli yedeği, FinStats yeniden başlatılana kadar diğer tüm yazmaları bekletebiliyordu. Böyle bir dosya reddedilir ve SQLite yedeği savunmacı biçimde okur.
- 4 GB’ı aşan bir yükleme reddedilir ve sıkıştırılmış bir Tautulli yedeği bildirdiği boyuttan öteye açılmaz; ikisi de FinStats’in kendi veritabanının bulunduğu diski doldurabilirdi.
- Jellyfin, Sonarr, Radarr, Seerr, genel adres sorgusu ya da GeoIP indirmesinden gelen bir yanıt, olabileceği en büyük boyuttan öteye okunmaz; boyut açıldıktan sonra sayılır.
- Herhangi bir Jellyfin kullanıcısı, tek bir başlığın binlerce kısa oynatmasını bildirerek FinStats’in herkes için oynatma kaydetmesini durdurabiliyordu. Kimin birlikte izlediğini hesaplamak artık tek geçişte yapılır ve çalışırken hiçbir kilit tutmaz.
- Art arda kaydedilen oynatmalar ya da cihazdan cihaza sırayla oynatılan tek bir başlık artık yabancıları tek bir grupta birleştirmez ve yalnızca kendini görebilen birine yalnızca kendi oynatmasının yanında yürüdüğü kişilerden, yürüdüğü süre boyunca söz edilir.
- Bir bağlantı, `/.//` gibi bir yol üzerinden, oturum açıldıktan sonra birini karşılama ekranından hâlâ başka bir siteye gönderebiliyordu.
- Bir yönetici artık kaç yedeğin tutulacağını ya da ne sıklıkla yazılacağını, birleştirme süresini (artık en fazla bir saat) ya da kullanıcıların, etkinlik günlüğünün ve yedeklerin ne zaman okunacağını değiştiremez ve bir yöneticinin yedekleri, bir sistem yöneticisinin yedeklerini asla dışarı itmez; ev adresleri ve seyahat kuralları uyarıların gerektirdiği izinleri ister, geçmişi içe aktarmak ise *Herkesin etkinliğini görme* iznini de gerektirir.
- Kaydedilmiş bir anahtar, belirteç ya da parola yalnızca kaydedildiği sunucuya gider ve bir bildirim hedefi API anahtarıyla değil, yalnızca oturum açıkken oluşturulur ya da değiştirilir. Bir yönetici başka birinin kişisel hedefini kapatabilir ya da kaldırabilir, ama başka yere yönlendiremez, sertifika denetimlerini kapatamaz, ntfy konusunu ya da Pushover kullanıcı anahtarını okuyamaz.
- Discord ve Slack webhook’ları `https` olmalıdır, çünkü adresleri aynı zamanda belirteçleridir.
- Başarısız oturum açma seli artık gerçek bir oturum açmayı gizleyemez ya da uyarıları gömemez: hesap olmayan adlar çeyrek saatte bir, kimseyi adlandırmayan tek bir bildirim oluşturur, uyarılar önce gönderilir, Slack ve Discord bir mesajın söylediğini düz metin olarak gösterir ve kimseyi etiketlemez.
- Yıl özetindeki sıralama, Kapasite, bir bildirim ve yayımlanmış bir profil sayfaların söylediğinden fazlasını söylemez: yalnızca kendini görebilen birine sıralama ya da herkesin süresindeki pay gösterilmez, *Herkesin etkinliğini görme* izni olmadan yoğun anlar yalnızca üç ya da daha fazla akışlı günler olarak gösterilir, FinStats’i yönetemeyen birine başarısız bir iş ya da bağlantı bildirilmez ve herkese açık bir profil günde bir kez değişir.
- Oluşturulan (generated) bir sütun içeren Tautulli yedeği reddedilir ve içe aktarılmış bir oynatma için resim ancak kütüphanede başlığı olduğunda alınır.
- Bir geri yükleme FinStats’i asla olduğundan daha geniş açmaz: ayarlar yalnızca istendiğinde geri gelir, oturum açma, izinler ve herkese açık profiller hem bu kurulumun hem de yedeğin izin verdiği şey olur ve genel adresler bu kurulumunkiler olarak kalır. `finstats restore <file>`, sayfanın yaptığı gibi yalnızca geçmişi, `--with-settings` ile ayarları da geri getirir.
- Yalnızca geçmişin geri yüklenmesi artık ev adreslerini, sessize alınmış uyarıları, bu kurulumun kendi kayıtları gibi görünen denetim kayıtlarını ya da kütüphanede olmayan konumlandırılmış başlıkları geri getirmez ve hiçbir geri yükleme 1990’dan önceye ya da yarından sonraya tarihli oynatmaları, günlük satırlarını ya da cihazları geri getirmez. Geri yüklenen bir oynatma, hangi kaynaktan geldiğini iddia ederse etsin içe aktarılmış sayılır, böylece bir yeniden başlatma onu asla sürdürmez.
- Veri klasörü yalnızca FinStats’in kendi kullanıcısına aittir (0700, dosyaları 0600, imajda `umask 077`). FinStats, başka bir kullanıcıya ait bir veri klasörü üzerinde root olarak çalışmayı reddeder ve kullanılacak `docker exec -u` komutunu söyler; entrypoint yalnızca klasörün ve FinStats’in kendi girdilerinin sahibini değiştirir.
- Bir hedefin adı denetim karakteri içeremez ve bir cihazın, kişinin ya da başlığın adı artık günlükte sahte bir satır oluşturamaz.
- Bir bağlantının rota parametresi artık kendi sayfasınınkinden başka bir API yoluna ulaşamaz ve FinUI’nin öğe oluşturucusu bir adresi yalnızca http(s) ya da göreli olduğunda ayarlar, asla `srcdoc` olarak değil.
- Arama, yayımlanmış profiller ve elle başlatılan işler artık FinStats’i meşgul edemez: arama 200 karakterde durur, bir profil her dil için ayrı saklanır, kartlar kısa bir kuyrukta bekler; bir iş, adres sorgusu, konum veritabanı indirmesi ve test mesajı, nasıl istenirse istensin yalnızca belirli aralıklarla başlar, test mesajları gönderen başına sayılır. Konum veritabanını indirmek, Güvenlik sayfasını okumanın gerektirdiği izinleri de gerektirir.
- Bir isteğin gövdesi ilk 15 saniyesi geçtikten sonra gelmeye devam etmeli, kimsenin okumadığı bir yanıttan 30 saniye sonra vazgeçilir ve bir adres, güvenilen bir proxy dışında, en fazla 64 bağlantı tutar. Gövdesini hiç göndermeyen birkaç yüz istek, FinStats'ın kimseye yanıt vermesini engelleyebiliyordu.
- Bir posta sunucusu hiçbir yanıtın olabileceğinden daha fazla okunmaz (bütün bir mektup için 1 MB) ve bir test iletisi, kuyruktaki bir iletinin sürebileceği kadar sonra durur. Hiç satır bitirmeyen bir sunucu, FinStats durdurulana kadar giderek daha fazla bellek tutmasına yol açabiliyordu.
- Bir kitaplık okuması, içe aktarmanın getirdiklerine göre değil, okumaların kaydettiği başlıklara göre değerlendirilir ve zaten okunmuş bir kitaplıktaki Jellystat başlığı kaldırılmış olarak gelir. Uydurma başlıkların içe aktarılması, her kitaplık okumasının her açılışta FinStats'ı durdurmasına yol açabiliyordu.
- Tüm sunucu için dakikada 60 başarısız oturum açma sınırı yalnızca Jellyfin'e iletilen denemeleri, yanıt bekleyenler dahil, sayar. FinStats'ın kendisinin reddettiği adlar, birkaç adresi olan herkesin herkesin oturum açmasını engellemesine izin veriyordu ve aynı anda gönderilen pek çok tahmin Jellyfin'e ulaşıyordu.
- Jellyfin'in ev ağıyla sınırladığı kişi yalnızca oturum açarken değil, oturumu ve anahtarları dahil her istekte orada tutulur; ev, özel bir ağ ya da hanenin aranıp bulunan genel adresidir, asla ev olarak yazılmış bir adres değildir ve `FINSTATS_TRUST_PROXY`'nin adlandırmadığı bir proxy arkasında dışarıda tutulur. Evden dışarı çıkarılan bir çerez ya da anahtar her yerden her şeyi açıyordu ve bir yönetici evi istediği bir adrese açabiliyordu.
- Bir API anahtarı artık kötüye kullanıldığını fark edecek şeyleri ya da yaptıklarını geri alacak yedekleri kapatamaz: kaç yedek ve ne sıklıkla tutulduğu, birleştirme penceresi, ev adresleri ve seyahat kuralları, kullanıcı, etkinlik günlüğü ve yedek zamanlamaları, tüm uyarıları susturmak ya da çözmek, bir yedeği çöpe taşımak veya bir bildirim hedefini kaldırmak. Her biri, anahtar iptal edildikten sonra da sürüyordu.
- Sonarr ve Radarr afişleri ile istek listesindeki sonraki bölüm de kitaplık erişimine uyar: Jellyfin'in birinden gizlediği bir dizinin o kişi için ne afişi ne de tarihi vardır. Hizmetlerin kimliklerini saymak hangi gizli dizilerin geldiğini ele veriyordu ve bir liste gizli bir dizinin sonraki bölümünü söylüyordu.
- Yıl özetinin Birlikte bölümü, tıpkı Birlikte sayfası gibi, yalnızca oynatması seninkiyle yan yana giden kişileri adlandırır. Başka biri aracılığıyla katıldığın bir grup, hiç birlikte izlemediğin kişileri adlandırıyordu.
- Jellystat veya Streamystats içe aktarması, veritabanının yazma kilidini almadan önce dosyanın tamamını okur ve ardından yalnızca tuttuğu satırları yazar. Kilit altında okunduğunda, birkaç gerçek satır ve gigabaytlarca başka şeyden oluşan bir dosya, oynatmalar ve oturum açmalar dahil diğer tüm yazmaları bekletiyordu.
- Geri yükleme kişi başına bir izin tutar, bir hanenin kat kat fazlasından çok kişi için değil, ve bir denetim kaydının kopyasını az önce yazdığı her kaydı okumadan arar. Hazırlanmış bir yedek, geri yükleme sırasında belleği ve veritabanının yazma kilidini giderek daha uzun tutabiliyordu.
- Ayarlarıyla yapılan bir geri yükleme, genel adres sorgusunu ve konum veritabanı indirmesini yalnızca bu kurulumda da açık oldukları yerde açar ve bu kurulumdan asla daha az yedek tutmaz ya da yedekleri daha seyrek yazmaz. Bir yedek tutan bir yedek, sonraki çalıştırmanın diğer tümünü kaldırmasına yol açabiliyordu.
- Hiçbir diskin alamayacağı bir dosya boyutu boyut yok olarak okunur ve boyutlar hiçbir toplam taşmayacak şekilde toplanır. İçe aktarılan tek bir boyut, kitaplık sayfalarının kalıcı olarak hata vermesine yol açıyordu.
- FinStats'a https ile ulaşıldığını bildiğinde (genel adresi https ise ya da güvenilen bir proxy bunu söylüyorsa) oturum yalnızca `__Host-` çerezinden alınır ve genel adres https ise her yeni oturum bir https oturumudur. Hiçbir şey söylemeyen bir proxy üzerinden `Origin` olmadan gönderilen oturum açma, komşu bir sitenin birinin tarayıcısına yerleştirebileceği düz bir oturum oluşturuyordu.

### Performance
- Yayımlanmış bir profil, üzerindeki her resim ve kart için yeniden değil, en fazla dakikada bir kez hesaplanır.
- Bir kişinin geçmişi, kaç oynatma başlarsa başlasın, güvenlik uyarıları için aynı anda yalnızca bir kez taranır.
- Oturum açmak ya da kapatmak, bir anahtarı iptal etmek ya da bir kişinin izinlerini değiştirmek, yalnızca bunun ilgilendirdiği canlı bağlantılardan yeniden bakmalarını ister.
- Tautulli bağlantı panosu geçmişi her Plex kullanıcısı için ayrı ayrı değil, bir kez okur.
- Önbellekte olmayan bir resim, diğerleri beklerken tek bir istekle alınır ve kaynağın veremediği bir resim 30 saniye boyunca yeniden istenmez.
- Web arayüzünün dosyaları bir kez gzip ile sıkıştırılıp saklanır ve yazı tipleri yeniden sıkıştırılmaz.

### Stability
- Jellystat’ın eski biçimindeki bir yedek bütün olarak değil satır satır okunur ve herhangi bir yedeğinkinden uzun bir satır reddedilir.
- Aynı anda yapılan iki Tautulli yüklemesi artık tek bir dosyaya yazmaz.
- Bir kütüphanenin başlıklarının ya da Kütüphane sağlığının sonunun çok ötesindeki bir sayfa numarası hata değil, boş bir sayfadır.
- Yüklemesi yarıda kesilen bir içe aktarma ya da geri yükleme artık yeniden başlatılana kadar diğer tümünü reddetmez; ne o ne de başlamadan önce bırakılan bir içe aktarma geride bir dosya bırakır.
- Bir yedeğin tek bir satırı 8 MB’tan öteye okunmaz ve aynı anda yalnızca bir Tautulli yedeği alınır.
- Bir oynatma en fazla 2.000 olay tutar ve bir içe aktarma, başlangıcı ve ardından gelen bitişi 1990 ile yarın arasında kalmayan bir oynatmayı atlar.
- Sıfırın çok altındaki tek bir bölüm numarası artık her kütüphane okumasını kilitlemez: Kütüphane sağlığı (özeti de), Kapasite, yeniden bağlama ve konumlandırma sayıları taşma olmadan toplar ve karşılaştırır, sürüm derlemesi de taşmayı denetler; böylece gözden kaçan bir şey döngüye girmek yerine tek bir görevi durdurur.
- Sezon, bölüm ya da bitiş numarası, Jellyfin’den, Jellystat’tan ya da Tautulli’den gelsin, yalnızca Jellyfin’in tutabileceği aralıkta (0 ile 2.147.483.647 arası) saklanır.
- Kimlikleri yinelenen bir Tautulli yedeği reddedilir, çünkü tek bir kimlik altındaki birkaç MB’lık satır sürecin belleğini tüketiyordu; bir yedek değişmez olarak açılır ve geride `-wal` ya da `-shm` bırakmaz.
- Bir bağlantı isteğini 15 saniye içinde göndermelidir ve aynı anda en fazla 512 bağlantıya hizmet verilir: birkaç yüz yarı açık soket FinStats’i erişilemez hale getiriyordu. Canlı kanaldaki bir çerçeve 4 KB ile sınırlıdır.
- Sonarr, Radarr ya da Seerr’den gelen birkaç kilobayt artık gigabaytlarca belleğe dönüşemez: bir liste, FinStats’in ondan okuduğundan fazlasını tutmaz.
- Hiç yanıt vermeyen bir bildirim hedefi yalnızca kendini bekletir: hedeflere yan yana gönderilir ve sürekli zaman aşımına uğrayan bir hedef on dakika boyunca rahat bırakılır.
- Bir kuyruğun kalan süresi ve toplam hızı, geri yüklenen bir oynatmanın sayıları ve bir oynatmanın çöp kutusundan çıktığı gün artık taşamaz.
- Bir geri yükleme, yazma kilidini almadan önce dosyanın tamamını okur, kendi boyutundan çok daha büyüğe açılan bir dosyayı reddeder ve gruplar halinde yazar; böylece toplayıcı bu sırada kaydetmeyi sürdürür.
- Yedekler UTC ile adlandırılır ve asla var olan bir dosyanın üzerine yazılmaz; yalnızca FinStats’in kendi güncelleme öncesi kopyaları budanır.
- Geleceğe tarihli bir sunucu günlüğü satırı artık etkinlik günlüğünün okunmasını durdurmaz: günlük, gelecekte olmayan en yeni satırından itibaren okunur.
- Yayımlanmış bir bağlantıya birlikte gelen istekler yanıtını bir kez hesaplar ve kartları kendine ait bir şeritte çizilir; böylece bağlantıya sahip biri artık sahibin Yıl özeti kartlarının 503 yanıtı vermesine yol açamaz.
- Jellyfin'in kitaplık listesinde olmayan bir kitaplığın oynatmaları, aynı başlığın başka bir kitaplıktaki kopyasına taşınmadan önce iki gün bekler. Bir kitaplığı atlayan tek bir liste onları hemen taşıyordu ve kitaplık geri geldiğinde taşınmış olarak kalıyorlardı.

### Fixed
- Bir hesaptaki başarısız oturum açmalar pencere başına tek bir uyarıdır ve yirmide bir alarma dönüşür; bir ad, bir hesaba karşı yalnızca tam yazıldığında sayılır ("x alice" artık alice’e karşı sayılmaz).
- Discord ve Slack mesajları önce kaçışlanır, sonra kesilir: Markdown dolu bir cihaz adı, Discord’un onu anan her uyarıyı reddetmesine yol açıyordu.
- Kişisel bir hedef için genel bir IPv6 adresi kabul edilir.
- İnsanların oturumunu kendisi açan bir proxy’nin arkasında (nginx `auth_basic`), FinStats’in çerezi artık reddedilmez.
- `{{url}}` içeren bir görünen ad, yayımlanmış bir sayfanın önizlemesinde yazıldığı gibi gösterilir.
- 9999 yılı için istenen özet, hata yerine varsayılan yılı gösterir.
- Açıkken izni alınıp geri verilen canlı bir sayfa, yeniden yüklenmeden tekrar güncel tutulur.

### Removed
- **Synnmørsk** artık FinStats'in dillerinden biri değil: onu seçmiş bir tarayıcı, dayandığı yazı dili olan Nynorsk'u okur.

## [2.3.0] - 2026-10-07 to 2026-10-26

Kapasite, canlı kanal, Basit mod ve diller

### Added
- **Diller**: FinStats 23 dil konuşuyor. İngiliz ve Amerikan İngilizcesi ile Norveççe Bokmål ve Norveççe Nynorsk, projenin bakımcısı tarafından yazılıp denetleniyor; Danca, Almanca, İspanyolca, Fince, Fransızca, Hintçe, İzlandaca, İtalyanca, Japonca, Korece, Felemenkçe, Lehçe, Portekizce (Brezilya), Rusça, İsveççe, Türkçe, Ukraynaca ve Çince (basitleştirilmiş ve geleneksel) burada kimsenin denetlemediği çevirilerdir ve henüz sahip olmadıkları kısımları İngilizce söylerler. İki düzenin her sayfası, herkese açık profil, Sürüm notları ve sunucunun sözle verdiği yanıtlar (bir hata, Yıl özeti kartları, bir Jellyfin görevinin ne yaptığı), tarayıcı başına Ayarlar → Görünüm → Dil altında seçilen dildedir; bir dil seçilene kadar tarayıcının kendi dili belirler. Sayılar, tarihler, birimler ve listeler o dilin yazdığı biçimde yazılır. Bir betik `Accept-Language` ile bir dil isteyebilir.
- **Basit mod**: Plex’ten gelenler için, klasik bir Plex istatistik sunucusunun düzenine göre kurulmuş ikinci bir düzen. Menü yerine üstte bir çubuk ve yalnızca istatistik sayfaları: Ana sayfa, Kütüphaneler, Kullanıcılar, Geçmiş, Grafikler ve Ayarlar; profiliniz, Ayarlar, Sürüm notları, tema ve oturumu kapatmak için de bir dişli. Tarayıcı başına Ayarlar → Görünüm → Düzen altından seçilir. FinStats’in yalnızca kendi yarısını yüklediği için daha hızlı açılır; Yıl özeti, Birlikte, Yolda, Güvenlik ve sunucu sayfaları FinStats düzeninde kalır. FinStats’in kendi renklerini açık ve koyu temada taşır ve üzerinde hiçbir şey hareket etmez.
- **Basit modda Ana sayfa**: oynatılanlar, akış başına bir kart (nasıl oynatıldığı, evden mi dışarıdan mı, ne zaman biteceği), son N gündeki oynatma sayısına ya da süresine göre sıralı kartlar halinde izleme istatistikleri, her kütüphanenin içerdikleri ve son gelenler.
- **Basit modda Grafikler**: medya türüne göre oynatmalar (gün gün, hafta günü ve saate göre, ilk on uygulama ve kişi), nasıl oynatıldıklarına göre (gün gün, her gün aynı anda en çok akış, dosyanın ve gönderilen görüntünün çözünürlüğü, uygulamalar ve kişiler) ve ay ay; oynatma sayısına ya da süresine göre, herkes için ya da seçilen kişiler için.
- **Basit modda Kütüphaneler, Kullanıcılar ve içerikler**: her kütüphane ve kişi için en son neyin, neyle ve nereden oynatıldığını gösteren tablolar (adres yalnızca *Ağ ayrıntılarını görme* izniyle) ve her kütüphane, kişi ve içerik için rakamlarını ve kendi geçmişini gösteren bir sayfa.
- Betikler için: `/api/stats/split` (uygulamaya, kişiye, çözünürlüğe, güne ya da aya göre oynatmalar; medya türüne ya da oynatma biçimine göre bölünmüş); farklı izleyici sayısına göre üst listeler (`sort=users`); izlenmiş oynatmalarla sınırlı etkinlik (`watched=1`); yalnızca istenen kısımlar için içgörüler (`parts=`); her gün için de kod dönüştürme, doğrudan akış ve doğrudan oynatma olarak bölünmüş zirve; her kütüphanenin içerdikleri ve en son oynatılanı; türe göre son eklenenler (`kind=`); medya türüne göre ısı haritası; bir kişinin son oynatması; bir akışın evden oynatılıp oynatılmadığı.
- **Canlı kanal**: pano, Yolda sayfasındaki indirmeler ve durum çubuğu, sekme başına tek bir WebSocket (`/api/live`) üzerinden kendilerini güncel tutar; böylece oynatılanlar ve indirilenler bir sonraki bakışta değil, değiştikleri anda ekranda değişir. WebSocket yükseltmelerini iletmeyen bir ters vekil arkasında ya da sessizce kopan bir bağlantıda sayfalar eskisi gibi birkaç saniyede bir sorar. `/api/now-playing`, `/api/downloads` ve `/api/summary` betikler için olduğu gibi kalır.
- **Sunucu → Kapasite**: FinStats’in gördüklerine dayanarak bu sunucu yükü kaldırır mı. Kayıtlardaki ve son 90 gündeki en yoğun anlar: aynı anda kaç akış olduğu ve bunlardan kaçının görüntüsünün donanımla, donanımsız ya da hiçbir şey söylenmeden kod dönüştürüldüğü (Tautulli’den içe aktarılan bir geçmiş Plex’e aittir ve dışarıda bırakılır). Kütüphanelerin bulunduğu her disk ve Jellyfin’in altı ayda eklediklerinin onu ne hızla doldurduğu (daha yakın zamanda doldurulmaya başlanmış bir disk, ilk dosyasından bir ay sonrasından itibaren ölçülür, çünkü ilk ayı ilk doldurulduğu aydır) ve Jellyfin 10.11 ya da daha yenisinin boş alan bildirdiği yerlerde bu hızla ne kadar yeteceği. FinStats sunucunun kaç akış kaldırabileceğini asla söylemez.
- **.ics dışa aktar**, Yolda → Yakında altında: listeyi filtrelediğiniz haliyle, bir kez içe aktarmak üzere bir takvim dosyası olarak indirir. Kendini güncellemez; kendini güncelleyen bir takvim için Ayarlar → API anahtarları bölümünden alınan bir takvim anahtarı hâlâ kullanılabilir.
- **Bakmaya değer**: Kapasite, kod dönüştürme yaygınken Jellyfin’in 90 gün içinde bildirdiği her kod dönüştürme donanım hızlandırması olmadan çalıştığında, Jellyfin’in kütüphane taraması zamanın dörtte birini ya da daha fazlasını aldığında ve bir diskin altı aydan az yeri kaldığında bunu not eder. Her biri rakamlarıyla gelir ve Kütüphane sağlığı bulguları gibi bir kenara bırakılabilir.
- **Kütüphaneniz nelerden oluşuyor** bir ya da birkaç kütüphaneyle sınırlanabilir, Kütüphaneler sayfasında ve her kütüphanenin kendi sayfasında (`library_id=a,b`).
- **Tarih ve saat biçimleri** Ayarlar → Görünüm'de: tarih 31.01.2026, 31/01/2026, 31-01-2026, 01/31/2026, 2026-01-31, 2026.01.31 ya da 2026/01/31 gibi, saat 24 ya da 12 saatlik, başta sıfırla ya da sıfırsız, ya da her biri dilin kendi yazışıyla. Bu tarayıcıda saklanır.
- **Bir karşılama**: biri hangi tarayıcıda olursa olsun ilk kez oturum açtığında ve kurulumun sonunda FinStats dili, tarih ve saat biçimlerini ve düzeni, her birinin resmiyle sorar. Atla İngiliz İngilizcesi, 31/01/2026, 13:05 ve FinStats'ı tutar.

### Changed
- Sayfanın içindeki logo, adının yanındaki yalın işarettir ve bir ön ayar hangi yazı tipini seçerse seçsin Inter ile yazılır; tek başına durduğu yerde (favicon) yuvarlatılmış karosunu korur.
- Yazılımla kodu dönüştürülen bir oynatma, Jellyfin bunu söylediğinde öyle kaydedilir; önceden bunu söylemeyen bir sunucudan gelen oynatmayla aynı görünüyordu. Sayfalar ikisine de hâlâ Yazılım der.
- Bir güncelleme artık, geçişlerinin yanında, eski bir FinStats’in yazdığı bir veritabanı için bir sürümün yapılmasını gerektirdiği işleri bir kez ve en eski sürümden başlayarak çalıştırır. 2.3.0’ınki ilk başlatmada sunucunun ayrıntılarını okur, böylece Kapasite hemen hazır olur.

### Performance
- **Açık bırakılan bir sayfa hiçbir şey istemez**: pano güncel kalmak için dakikada 30, Yolda sayfasındaki indirmeler 18 istek yapıyordu; canlı kanalla ikisi de hiç istek yapmaz. Kaç sekme açık olursa olsun FinStats Jellyfin’den, Sonarr’dan ve Radarr’dan öncekinden fazlasını istemez ve gizli bir sekme indirme kuyruğunun yavaşlamasına hâlâ izin verir.
- Bir kişinin sayfası yalnızca o kişinin oynatmalarını okur: toplamları ve son oynatması sunucudaki her oynatma üzerinden hesaplanıp sonra o kişiye daraltılıyordu; bu, bir milyon oynatmada yaklaşık dört saniye sürüyordu.
- Bir kütüphanenin sayfası, her kütüphaneninkini sayıp birini tutmak yerine yalnızca o kütüphanenin içeriklerini ve oynatmalarını okur: bir milyon oynatmada sürenin üçte biri.
- Jellyfin’in saati FinStats’inkinden ilerideyken *Jellyfin’in kütüphane taramasından sonra*, fark kapanana kadar her dakika tüm kütüphaneyi yeniden okuyordu; artık her tarama bir kez izlenir.
- Açılamayan bir GeoIP dosyası her çeyrek saatte bir yeniden kopyalanıyor (yüz megabayt ve fazlası) ve her adresin konumu yeniden belirleniyordu; artık yalnızca dosya değiştiğinde yeniden denenir.
- İşaretçi bağlantısının üzerinde bir an durduktan sonra Kütüphane sağlığı anında açılır: önceden getirilen veri, sayfanın hiç bakmadığı bir yerde tutuluyordu.

### Stability
- Bir geçiş çalıştıktan sonra başarısız olan ve Docker tarafından hemen yeniden başlatılan bir güncelleme, her başlatmada yarı güncellenmiş veritabanının yeni bir kopyasını alıyordu ve üç başlatma, önceki sürümün açabileceği tek kopyayı budayıp siliyordu. Artık güncellemeden önceki kopya korunur.
- Kütüphane okunurken Jellyfin’de silinen bir içerik (Radarr’ın değiştirdiği bir dosya), okumanın hâlâ orada olan başka bir içeriği kaçırıp onu kaldırılmış olarak işaretlemesine yol açabiliyordu; ardından oynatmaları onun başka bir kopyasına taşınabiliyordu. Bir okumanın görmediği şey artık önce kimliğiyle sorgulanır.
- Bir dosya yeniden adlandırılmadan ya da yükseltilmeden önce alınmış bir yedeği geri yüklemek, o zamandan beri yeni dosyaya taşınmış her oynatmayı ikinci kez geri yüklüyor ve kalıcı olarak ikiye katlıyordu.
- Seerr’in yanıtında FinStats’in okuyamadığı bir istek (bir Seerr güncellemesinde adı değişen bir alan) günlük tam okuma tarafından kaldırılmış olarak işaretleniyordu.
- Bir hataya takılan bir görev kalıcı olarak “çalışıyor” durumunda kalıyor ve FinStats yeniden başlatılana kadar hiçbir tetikleyici onu yeniden başlatmıyordu; artık başarısız olarak sona erer.
- Hasarlı bir Tautulli zip dosyası yüklemeyi, her Plex kullanıcısının belirteçlerini içeren dosya kaldırılmadan önce durduruyordu; dosya altı saatlik temizliğe kadar kalıyordu.
- Dosyası gelirken sonlandırılan bir içe aktarma ya da geri yükleme, o dosyayı içindeki adreslerle birlikte veri klasöründe kalıcı olarak bırakıyordu; artık bir sonraki başlatmada kaldırılır.
- Bir resim önbelleğe alınırken yarı yazılmış haliyle sunulabiliyordu ve tarayıcı onu öyle saklıyordu.
- İptal edilen API anahtarları, kaç tane oluşturulup iptal edilmiş olursa olsun kalıcı olarak saklanıyordu.
- Birinin izlendi ve favori işaretlerinin boş ya da okunamaz dönen bir okuması, bir sonraki okumaya kadar hepsini siliyordu; artık bir kütüphanenin boş okuması gibi reddediliyor.
- Hiçbir bölümün olamayacağı bir bitiş numarasına (oraya yazılmış bir tarih) sahip bir bölüm, kütüphane okumasından sonra Kütüphane sağlığını sonsuza dek meşgul edebiliyordu.

### Fixed
- Kalın yazının içinde kod bulunan bir sürüm notu (2.0.2 ve 2.0.3’tekiler) kodu çizmek yerine ters tırnakları gösteriyordu.
- Sayıları gösteren bir grafiğin ekseni 0, 3, 5, 8, 10, hatta 0, 0, 1, 1, 1 diye numaralanabiliyordu: 2,5'lik adımlar yuvarlanarak yazılıyordu. Sayılar artık tam adımlarla artıyor.
- FinStats’i yönetme izni olmayan birinin oluşturduğu takvim anahtarı, FinStats’in herkese açık adresi yerine tarayıcının kendi adresini taşıyan bir bağlantıyla geliyordu, bu yüzden yalnızca içeriden çalışıyordu; sayfa ayrıca kendisine verilmeyen ayarları istiyordu.
- Bir kişinin son oynatması, başka hiçbir yerde oynatma sayılmayacak kadar birkaç saniye kısa olabiliyor ve Geçmiş sayfasında görünmeyen bir içeriği gösterebiliyordu.
- Son silinenler bölümünden başarısız olan bir geri yükleme hiçbir şey söylemiyor ve düğmesi dönmeye devam ediyordu; Kütüphane sağlığındaki başarısız bir *Geri getir* ile Güvenlik’teki başarısız bir *Yeniden aç* ya da *Tümünü çözüldü say* da öyle.
- Takvim akışı bir takvim uygulamasında finstats değil, FinStats olarak adlandırılır.
- İndirme kuyruğu, her indirmeyi kimin istediğini kuyruğu görebilen herkese söylüyordu; başka birinin isteği artık, Yolda sayfasında olduğu gibi *Herkesin etkinliğini görme* iznini gerektirir.
- *Bildirim alma* izninin geri alınması, o kişinin hedeflerine gönderilenleri durdurmuyordu ve kişi bu hedefleri artık kapatamıyordu.
- Birleştirme süresi içinde duraklatılıp devam ettirilen bir oynatma bir kez durdurulmuş olarak bildiriliyor, devam etmesi ve gerçek sonu ise hiç bildirilmiyordu.
- Bir parçanın oynatmaları, albümü kütüphaneden ayrıldığında başka bir sanatçının aynı adlı parçasına (“Intro”) taşınabiliyordu.
- IPv6 içinde IPv4 adresi olarak yazılmış bir bildirim hedefi (NAT64, 6to4), yerel ağa ulaştığı halde herkese açık sayılıyordu.
- Başka bir sunucuya yönlendirilen bir Sonarr ya da Radarr bağlantısı, eski sunucunun posterlerini bir hafta boyunca göstermeye devam ediyordu.
- Güvenlik uyarıları, iki ya da daha fazla kişi seçildiğinde bakılacak bir şey olmadığını söylüyordu ve son uyarısı çözülerek boşalan bir sayfadan geri dönüş yolu yoktu.
- Süresi dolmamış her API anahtarı “az önce” sona erdiğini söylüyordu.
- Geri’ye basıldıktan sonra bir iletişim kutusu sayfanın üzerinde kalıyordu ve kapatılana kadar sayfa kaydırılamıyordu.
- Denetim günlüğü sekiz giriş türünü (geri getirilen bir oynatma ya da yedek, boşaltılan çöp kutusu, herkese açık profiller, değiştirilen bir görünüm) kimlikleriyle gösteriyordu ve Ayrıntı sütunu boştu.
- Bir içeriği bağlam menüsünden izleme listesine eklemek, sunucu reddettiğinde sessizce başarısız oluyordu.
- Panodaki *Birlikte izlendi*, aynı anda seçilen birkaç kişiyi tek kişi gibi ele alıyordu.
- Yıl özeti, ondan küçük saat ve gün değerlerini kendi binlik virgüllerinin yanında tarayıcının ondalık virgülüyle yazıyordu.
- Kapasite, bir diskin "son 0 ayın hızıyla" dolacağını söylüyordu.
- Güvenlik sayfasında yeri olmayan adresler, sayfa hangi kişiye ve döneme daraltılmış olursa olsun herkesin tüm zamanlardaki oynatmalarını sayıyordu.
- *FinStats’i yönetme* izni olan ama *Herkesin etkinliğini görme* izni olmayan biri, herkesin oynatmasını çöpe atabiliyordu.
- Yıl özeti ya da herkese açık profil kartı, tarayıcının önbelleğinden daha önce seçilen dilde gelebiliyordu.
- Canlı kanalın gönderdiği indirmeler, sayfa hangi dilde olursa olsun İngiliz İngilizcesiydi.
- Birinin görebileceklerini değiştiren bir oturum açma, o kişinin diğer açık sekmelerine ancak yarım dakikalık bir sonraki denetimde ulaşıyordu.
- Canlı bağlantısını daha yeni bir sekmeye bırakan sekme, sonraki sayfasında bir tane geri alıyordu; böylece bir kişinin sekmeleri birbirinin bağlantısını kapatıp duruyordu.
- Tablolar, rakamları boşlukla gruplayan dillerde 1.000 ile 1.023 birim arasındaki boyutları metin gibi sıralıyordu; Türkçe yüzdeler de öyle.
- Adında yer tutucu olan bir başlık, örneğin "Big {device} Bunny", sunucunun iletilerinde dolduruluyordu.
- Tablolar, İngilizce ve İskandinav dilleri dışındaki her dilde süreleri ve bir toplamdan sayıları (5 / 26) metin gibi sıralıyordu.
- Bir dilde açık kalan sekme, başka bir sekme başka bir dili seçer seçmez sunucuyu o dilde duyuyordu.
- Bir sayının yanındaki sözcük Lehçe, Rusça ve Ukraynacada yanlış biçimdeydi ("5 wieczory", "2 потоков").
- Kapasite, bir kütüphanenin ilk dolumunu büyüme sayıyor ve her kütüphanenin eklediğini, hangi süre üzerinden ölçülürse ölçülsün altı aya bölüyordu; başka bir dosyanın yerini alan bir dosya (Radarr ya da Sonarr'dan bir yükseltme veya Jellyfin'in yeni kimlik verdiği bir dosya), yalnızca fark yeniyken tamamen yeni sayılıyordu.
- SyncPlay ile birlikte izleyenler, biri diğerinden dakikalar sonra katıldığında bir grup sayılmıyordu; aynı anda aynı yerde biten oynatmalar artık birliktedir ve geçmiş bir kez yeniden gruplanır.
- *Herkesin etkinliğini görme* izni olmayan biri, bir kütüphanenin *Hiç izlenmeyenler* listesinden başka birinin bir başlığı oynattığını anlayabiliyordu; artık yalnızca kendi izlediği sayılır.

## [2.2.0] - 2026-10-01 to 2026-10-07

Kütüphane sağlığı ve FinUI

### Added
- **Bilgisayarda size ait bir menü**: Ayarlar → Görünüm sekiz seçenek sunar: eskisi gibi kenar çubuğu, sayfaların ne için olduğuna göre gruplanmış bir menü, işaretçinin altında kenar çubuğuna açılan bir simge şeridi, aramayla başlayan bir kenar çubuğu, bir rıhtım, bir komut çubuğu, en üste sabitlediğiniz sayfalar ve renkli kutucuklar. Kenar çubukları sağda durabilir. Rıhtım siz aşağı kaydırırken kayıp gizlenir, yukarı kaydırdığınızda ya da pencerenin altını işaret ettiğinizde geri gelir; onunla birlikte durum çubuğu üste taşınır.
- **Telefonda size ait bir menü**: Daha fazla düğmesi olan bir sekme çubuğu (yeni varsayılan), yarıya kadar açılan bir sayfa, tüm ekran, başparmağın çevresine yelpaze gibi açılan sayfalar ya da menüye dönüşen bir adres çubuğu. Her biri kendi sayfaları arasında arar ve diğer sözcükleri aramaya devreder.
- İkisi de seçildikleri tarayıcıda saklanır, böylece telefon ve bilgisayar kendi seçimlerini korur; ikisi de FinUI bileşenidir: `desktop-nav` ve `mobile-nav`.
- **Bağlam menüsü**: FinStats’in herhangi bir yerinde bir içeriğe, kişiye ya da kütüphaneye sağ tıklayın (panodaki bir poster, bir listenin satırı, bir tablodaki ad, Yakında’da henüz kütüphanede olmayan bir film), parmağınızı üzerinde tutun ya da menü tuşuna basın; bastığınız noktadan bir menü büyür: açın, yeni sekmede açın, bir filmi ya da diziyi izleme listenize ekleyin, Jellyfin’de ya da TMDB ve TVDB’de açın, bağlantısını kopyalayın. Shift ile sağ tıklama hâlâ tarayıcıya aittir. FinUI’nin `context-menu` bileşeni.
- **İletişim kutusu olmadan arama**: arama menünüzden büyür (bir kenar çubuğu genişleyerek ona dönüşür, rıhtım yükselerek ona dönüşür, komut çubuğunun adı ona dönüşür, telefonun menüsü onu içinde taşır) ya da Ayarlar → Görünüm’de seçilirse menü yerinde kalırken sayfanın yerini alır. Ctrl+Space onu eskisi gibi açar; her şeyin üzerinde açılan pencere kalktı. FinUI’nin `search` bileşeni.
- **Bir kütüphanedeki her şey**: bir kütüphanenin Son eklenenler kartı *İçindeki her şey* seçeneğini sunar: içerdiği her film, dizi ya da albüm; ada, yıla, eklenme tarihine ya da boyuta göre poster olarak, siz yazdıkça daraltılarak. Kaydırdıkça daha fazlası gelir.
- **Sunucu → Kütüphane sağlığı**: FinStats her dosyayı komşularıyla karşılaştırır ve göze batanları, arkasındaki rakamlarla birlikte listeler. Yalnızca bildirir; Jellyfin’de hiçbir şey değiştirilmez, silinmez ya da yeniden taranmaz.
- **Boşluklar**: bir sezonun iki dosyası arasında eksik bölümler (“2. sezon: 3 ile 6 arasında 4. ve 5. bölümler eksik”). İki bölüm içeren bir dosya ikisi olarak da sayılır; özel bölümlere ve sezon sonuna dokunulmaz.
- Çözünürlük, HDR ya da codec bakımından dizinin geri kalanından **farklı sezonlar** ve **farklı bölümler**: 1080p bir sezondaki SD bir bölüm gibi, sezonuna benzemeyen tek bir dosya. Çözünürlük genişliğe ya da yüksekliğe göre belirlenir, bu yüzden kırpılmış 1920×800 bir film 1080p sayılır.
- **Kopyalar ve sürümler**: izleme listesinin ve Yolda sayfasının zaten kullandığı sağlayıcı kimliklerine göre iki kez bulunan aynı film ya da bölüm. Aynı çözünürlükteki iki kopya iki kez tutulan alanı gösterir; 1080p bir kopyanın yanındaki 4K kopya iki sürüm olarak listelenir ve asla israf sayılmaz.
- **Zayıf dosyalar**: çözünürlüğünün ve codec’inin genellikle gerektirdiğinin çok altında bir bit hızı (“1,1 Mbps’de 4K HEVC”); bir sezonun zayıf bölümleri tek bir bulgudur.
- **Yarıda kalan dublajlar**: bazı sezonlarda her bölümde olup diğerlerinde hiç olmayan bir ses dili.
- **Hiç tanınmamış**: Jellyfin’in TMDB, TVDB ya da IMDb kimliği olmadan eşleştirdiği filmler ve diziler.
- Bilerek öyle olan bir bulguyu bir notla **yok sayın**. Dosyaları değiştiğinde kendiliğinden geri gelir. FinStats’i yönetebilen ve sunucuyu görebilenler yok sayabilir; yok saymalar denetim günlüğünde yer alır ve yedeğin parçasıdır.
- Bir kütüphanenin sayfası orada bakılacak kaç şey olduğunu söyler ve onlara götürür.
- **FinUI**, FinStats’in üzerine kurulduğu bileşenler, artık kendi başına bir proje (`github.com/finstats/finui`): her düğme, kart ve denetim, her durumda ve iki temada `finui.finstats.no` adresinde; ayrıca renklerini, köşe yuvarlaklığını, yoğunluğunu ve daha fazlasını izleyerek değiştirip tek bir komutla evinize götürebileceğiniz **FinUI create**. **Ayarlar → Sistem** oraya götürür.
- **Ayarlar → Görünüm**: herkes FinStats’in kendisine nasıl görüneceğini bir FinUI ön ayarının koduyla seçer: temel renk, vurgu, grafik renkleri, kontrast, köşe yuvarlaklığı, yoğunluk, kenarlıklar, kartlar, düğmeler, alanlar, tablolar, vurgulama, hareket, simgeler, menü, her şeyin arkasındaki sayfa, odak halkası, başlıkların nasıl dizildiği ve metin, başlık ve sayılar için yazı tipleri. Seçiminiz yalnızca size aittir; sayfa kodun ne anlama geldiğini gösterir ve değiştirmek için FinUI create’e götürür.
- **Stiller**: FinUI create’te tek seçimle başlanabilecek on bir bütünlüklü görünüm; kodunu Ayarlar → Görünüm alır. Bunlar Washi (FinStats’in geldiği hali), Studio, Noir, Matinee, Console, Pastille, Ledger, Gazette, Arcade, Canopy ve Harbour. FinStats’in kendisi tek bir görünümle gelir ve stil listesi sunmaz.
- **Hareket eden simgeler**: bir bağlantıdaki, düğmedeki ya da sekmedeki her simge, üzerine gelindiğinde kendine özgü hareketini bir kez yapar (yenile döner, indirmenin oku tepsisine düşer, çöp kutusu kapağını kaldırır) ve basılan bir yenile düğmesi yanıtı gelene kadar dönmeye devam eder. Azaltılmış hareket açıkken ve Hareket: Kapalı seçiliyken durağan kalır.
- **FinMotion ile hareket**: FinUI’nin bileşenleri artık hareket ediyor: bir anahtarın topuzu karşıya fırlatılır, bir sekmenin alt çizgisi bir sonraki sekmeye doğru süzülür, reddedilen bir alan başını iki yana sallar, kopyalama onayı tek bir darbeyle çizilir, bildirimler bir el iskambil kâğıdı gibi üst üste yığılır ve üzerine gelindiğinde yelpaze gibi açılır, telefonun sayfası çekildiğinde lastik gibi esner, sıralanmış bir tablonun satırları yerlerine kayar, değişen bir rakam kilometre sayacı gibi döner ve grafikler ilk görüldüklerinde kendilerini çizer. Hepsi Ayarlar → Görünüm’deki Hareket seçimine uyar ve azaltılmış hareketle durağan kalır. FinMotion, FinUI’nin üzerine konan kendi başına bir projedir (`github.com/finstats/finmotion`).
- **FinUI’de daha fazla simge**: 71’den 239’a; her birinin kendine özgü bir hareketi var (ses açma ve kısma, bir kulaklık ve sesi kapatılmış bir kulaklık, yazan bir kalem, çalan bir zil, titreyen bir telefon) ve her biri FinUI’nin galerisinden durağan ya da animasyonlu SVG olarak indirilebilir.
- **Yazı tipleri**: seçilebilecek otuz dört açık lisanslı aile (Geist, Roboto, Manrope, IBM Plex Sans, Montserrat, Lora, Playfair Display, Fira Code ve daha fazlası); Inter gibi FinStats ile birlikte gelir, asla başka bir yerden getirilmez ve yalnızca seçildiğinde yüklenir.
- **Daha fazla temel ve vurgu**: Nord, Solarized, Gruvbox, Catppuccin, Rosé Pine, Tokyo Night, Dracula, Everforest ve Ocean; on altı vurgu daha, onu gündüz bir renk, gece başka bir renk. Her temelin artık gündüz için kendi kâğıdı var: açık temada griler birbirine benziyordu.
- **Silme 30 gün boyunca geri alınabilir**: silinen bir oynatma ya da yedek çöp kutusunda bekler ve ardından gelen bildirimdeki **Geri al** onu geri getirir. Etkinlik ve Ayarlar → Yedekler, son silinenleri, varsa, her birinin ne zaman kalıcı olarak gideceğiyle birlikte listeler. Çöp kutusundaki bir oynatma her toplamın, grafiğin, yıl özetinin ve profilin dışında kalır ve bir içe aktarma ya da geri yükleme onu yeni olarak geri getirmez; onu biriyle birlikte izleyen kişi, oynatma geri dönene kadar tek başına izlemiş sayılır.

### Changed
- Dark Reader ve benzeri eklentiler FinStats’in sayfalarına dokunmaz (`darkreader-lock`): FinStats iki temayı da kendisi çizer ve yeniden boyanan bir sayfa onlarla çatışıyordu.
- Telefonda menü artık soldan kayarak gelmiyor: başka biri seçilmedikçe sekme çubuğu altta durur.
- FinStats bu güncellemeden sonra, birkaç bölüm içeren dosyaların nerede bittiğini öğrenmek için kütüphanenizi bir kez okur.
- Bir içeriğin sayfasında **İzleme listesine ekle**, yanındaki **Jellyfin’de aç** kadar yüksektir; bir boy küçüktü.
- Zaman aralığı ve kişi filtreleri sayfa kaydırılırken ekranın üstünde kalır, böylece uzun bir sayfanın dibinden 7 gün Tümü yapılabilir; Yolda sayfasının istek ve yakında filtreleri de öyle.
- Başa dön düğmesi pencerenin altını hangi menü tutuyorsa ondan uzak durur: rıhtımın yanında köşenin alt kısmında, telefonun sekme çubuğunun, başparmak düğmesinin ya da adres çubuğunun hemen üstünde, diğer durumlarda durum çubuğunun üstünde. FinUI’nin `to-top` bileşenidir.
- Bir oynatmayı ya da yedeği silmek onu hemen kaldırmak yerine çöp kutusuna taşır (`PUT` isteği `{"deleted": true}` ile; `DELETE` API’den kalktı). Saklanan sayıyı aşan yedekler hâlâ kalıcı olarak kaldırılır.
- **Sürüm notları** bir kart yığını yerine tek bir sayfadır: her not kendi satırındadır ve önündeki sütunda türü (Eklendi, Değişti, Düzeltildi ve diğerleri) ile o türün simgesi durur; her seri bir başlığın altındadır. Sürüm listesi sağda kalır.

### Performance
- FinStats’i açmak yalnızca değişenleri indirir: tarayıcının zaten sahip olduğu bir stil sayfası, betik ya da yazı tipi yeniden gönderilmek yerine birkaç baytla doğrulanır. Her ziyaret, her biri onları ayırt edecek bir etiket taşısa da FinStats’in tüm stil sayfalarını ve modüllerini eksiksiz indiriyordu.

### Stability
- Bir yedeği geri yüklemek ayarları yalnızca ayarlar satırından okur: orada daha fazlasını tutan daha yeni bir FinStats’ten gelen bir dosyanın ayarları sessizce düşürülüyordu.
- Açık bir sekme artık ziyaret edilen her sayfayla büyümüyor: terk edilen her sayfa, sekme kapanana kadar FinMotion’ın hareket ettirdikleri listesi ve filtre çubuğunun kaydırma dinleyicisi tarafından tutularak ekrandakinin arkasında bellekte kalıyordu (ziyaret başına yaklaşık 2.600 öğe).
- Yanlış yazılmış bir komut (`finstats bakcup`) ya da dosyası olmadan `restore`, artık ne yapacağını bilmediğini söylemeden önce bir veritabanı oluşturmuyor ya da yükseltmiyor.

### Fixed
- Bir kutucuğun küçük grafiği, yanındaki sayı geniş olduğunda (“56 gün 23 sa”) ya da dört kutucuk daha dar bir satırı paylaştığında artık kutucuğun dışına taşmıyor: grafik yer açar ve sayı genişliğini korur.
- Açık temada birincil düğme işaretçinin altında mora dönüyordu; artık mührün rengini korur.
- Ayarlar’da bir anahtar, ayarın adı ve yardım metniyle aynı hizada durur; satırın tepesinde asılı duruyordu.
- Panodaki En etkin kullanıcılar gibi, bir adın altında hiçbir şey olmayan sıralı bir listede ad, ortalanmış bir resmin yanında satırının tepesinde duruyordu; artık ortalanmış.
- Telefon ile masaüstü arasındaki bir genişlikte bir istatistik kutucuğu satırı (üçerli dizilen dört kutucuk) sonuncuyu kendi satırında yalnız bırakıyordu; dört kutucuk artık dörderli, ikişerli ya da birer birer, üç kutucuk üçerli ya da birer birer dizilir.
- Sunucu okuması, kimsenin oturum açmadığı ve FinStats’in saklamadığı bir cihazı da sayıyordu: Sunucu sayfası iki cihaz listelerken görev üç cihaz diyordu.
- Bir saniye, bir dakika, bir saat: uyarılar “1 seconds apart” ve “in 1 minutes” diyordu, bir dakikalık aralıkla çalışan bir Jellyfin görevi de “every 1 minutes” çalışıyordu.
- `finstats --help` FinStats’in okuduğu her değişkeni adlandırır; `FINSTATS_PUBLIC_IP_URL`, `FINSTATS_ALLOW_LIBRARY_SHRINK` ve `FINSTATS_SKIP_PREUPDATE_BACKUP` eksikti.
- Kurulum sihirbazı, adres olmayan bir adres için bir sonraki cümleye karışan bir cümleyle “bağlanılamadı” diyordu; artık bunun bir adres olmadığını söyler.
- Ayarlar → Yedekler’de bir yedek yazılırken sorulan bir soru, yedek bittiğinde kayboluyordu.
- Menü açılan şerit olarak ayarlıyken aramayı kapatmak açık aramayı ve kapalı şeridi aynı anda gösteriyor, ardından işaretçi gelip gidene kadar şeridi açık bırakıyordu. Esc artık klavye odağını arama açıldığında bulunduğu yere geri verir, bir tıklamayla gelen odağı ise serbest bırakır.
- Telefonda aramayı kapatmak, arama gidene kadar sayfayı karartılmış tutuyor ve sonra tek karede aydınlığa geçiyordu; karartma artık aramayla birlikte solar.
- Tautulli içe aktarmasının adımları yedeğin **Backup Database** düğmesinden sonra indirildiğini söylüyordu. Tautulli onu kendi makinesine kaydeder: adımlar artık nereye kaydettiğini (`backups` klasörü, Docker ile `/config`), hangi dosyanın alınacağını ve onun buraya kopyalanması gerektiğini söyler.
- Açılması bitmeden kapatılan arama, açılan şeridin animasyonunu bozuyordu: o andan sonra şerit anında açılıp kapanıyordu.
- Açılan şerit, ondan bir sayfa seçildikten sonra başka bir yere tıklanana kadar açık kalıyor ve bazı tarayıcılarda ya da yakınlaştırılmışken simgelerinin altında bir kaydırma çubuğu çiziyordu. Artık yalnızca işaretçi ve klavye için açılır ve kapalıyken kaydırma çubuğu çizmez.
- Güvenlik sayfasında, veritabanının bir ülkeye yerleştirdiği ama haritada gösterilemeyen bir adres, konumu bilinmeyen adresler arasında değil, o ülkenin içinde sayılır.

## [2.1.2] - 2026-10-01

### Added
- **Tautulli’den içe aktarma**: Plex geçmişinizi buraya taşıyın. Tautulli’nin veritabanı yedeğini (`.db` ya da onu içeren `.zip`) **Ayarlar → İçe aktar** altında yükleyin; oynatmaları Etkinlik sayfasındaki *Kaydeden* filtresinde kendilerine ait bir yer alır.
- **Kimin kim olduğunu söylemek için bir kablo panosu**: bir yanda Plex kullanıcıları, diğer yanda Jellyfin kullanıcıları. Her Plex kullanıcısından şimdiki kişisine bir kablo sürükleyin, ya da önce birine sonra diğerine tıklayın, ya da klavyeyi kullanın. Kablosu bağlanmamış biri içe aktarılmaz ve iki Plex hesabı tek bir kişide birleşebilir.
- Filmler ve bölümler kütüphanenizle adlarına göre eşleştirilir (bir film adı ve yılıyla, bir bölüm dizisi ve numarasıyla); henüz sunucunuzda olmayan bir içerik saklanır ve geldiğinde bağlanır. Devam ettirilen bir izleme tek bir oynatmadır; müzik içe aktarılmaz.
- Yedek, Plex’in erişim belirteçlerini içerir: FinStats onları asla okumaz ve siz içe aktardığınızda ya da baştan başladığınızda dosyayı kaldırır.
- **Ayarlar → Veri** altında **Bağlanmamış medya**: Tautulli, Jellystat ya da Streamystats’ten gelen ve kütüphanenizde hiçbir şeyle eşleşmeyen her içerik; nereden geldiği ve büyük olasılıkla zaten nerede bulunduğuyla birlikte. **Bul**, onu en iyi tahminlerden ya da bir aramadan bağlar (bir film, bir dizinin özel bölümü olarak da bulunabilir) ve FinStats seçimi hatırlar, böylece aynı geçmiş yeniden içe aktarıldığında kendiliğinden bağlanır. Seçimler yedeğin parçasıdır.

### Performance
- **Yıl özeti üç ila beş kat daha hızlı açılır**: yılın en çok izlenen oyuncuları ve yönetmenleri, yıldaki her kişi için yıl bir kez yeniden taranarak hesaplanıyordu. Gerçek bir geçmişte ölçüldüğünde bir yıl 0,29 sn’den 0,09 sn’ye, son 12 ay 0,53 sn’den 0,12 sn’ye indi.

### Fixed
- **İçe aktarılan geçmiş başka bir adla bilinen içerikleri de bulur**: bir oynatma bir içeriğin orijinal dildeki adıyla da eşleşir; böylece Plex’in ya da Tautulli’nin 오징어 게임 olarak bildiği dizi Squid Game ile, *Im Westen nichts Neues* de *All Quiet on the Western Front* ile eşleşir. Kütüphanenizin orijinal adları bir sonraki taramada okunur.
- Adlar aramanın karşılaştırdığı gibi karşılaştırılır (büyük/küçük harf, aksanlar, noktalama, ada yazılmış bir yıl, baştaki bir “The”) ve çıkış yılı bir yıl farklı olabilir; böylece “Star Wars: Episode I - The Phantom Menace”, “… – The Phantom Menace” adlı içeriği, 2014 yapımı bir Kingsman da 2015 yapımı olanı bulur. Tamamen başka bir adla bilinen bir içerik için ise hâlâ asla tahmin yürütülmez.
- Başarısız bir Streamystats içe aktarması ya da geri yüklemesi, bildirimlerde iç adıyla duyuruluyordu.
- Yıl özetindeki *Yılı kartlarla paylaşın* düğmesi ile yanındaki izleyici sıralaması arasında boşluk yoktu.

## [2.1.1] - 2026-10-01

### Changed
- **İzleme listesi, bir içeriğin bulunduğu her yerde sunulur**: Yolda → Yakında’da olduğu gibi panodaki Yakında kartlarında ve bir bölümün ya da sezonun sayfasında; ikincisi dizisini listeye ekler.
- Panodaki bir posterde izleme listesi düğmesi, her posterin köşesini kaplamak yerine karta işaret ettiğinizde ya da klavyeyle ulaştığınızda görünür; dokunmatik ekranda her zaman oradadır.

### Stability
- Biri **Ayarlar → Giden bağlantılar** bölümünü açık tutarken bir Sonarr, Radarr ya da Seerr bağlantısını kaydetmek, ikisini birbirini sonsuza dek bekler halde bırakabiliyordu.
- **Okunamayan ayarlar FinStats’i başlangıçta durdurur** ve hatalı ayarı adlandırır; her ayarı sessizce varsayılanına sıfırlamaz.
- **Ayarlarda aynı anda yapılan iki değişiklik** (örneğin ayarlar sayfası ve bir görevin zamanlaması) artık birbirini geri almıyor.
- Veritabanı meşgulken görülen bir duraklatma, sarma ya da parça değişikliği kaybolmak yerine bir sonraki turda yazılır.
- Bir kütüphane okuması, aynı içerik olduğu anlaşılan birkaç girdi barındıran bir izleme listesi yüzünden artık başarısız olmuyor.

### Fixed
- **Değiştirilen bir dosya “artık sunucuda” sayılmaz**: birinin izleme listesindeki bir filmin dosyasını yükseltmek artık ona filmin yeni geldiğini söylemiyor.
- Jellyfin’in eklentilerinin, zamanlanmış görevlerinin ya da depolamasının başarısız tek bir okuması artık Sunucu sayfasındaki o listeyi bir sonraki okumaya kadar boşaltmıyor.
- **Bir Streamystats oynatması her sayfada durduğu yerde durur**: bir içeriğin bırakma eğrisi ve bir bölümün “bitirildi” bilgisi bunun yerine oynatmanın uzunluğuna bakıyor ve onu tahmini sayıyordu.

## [2.1.0] - 2026-09-29 to 2026-09-30

İzleme listesi

### Added
- **Size ait bir izleme listesi**: bir filmi ya da diziyi sayfasından, Yakında’dan, Son eklenenler’den ya da aramadan (Ctrl+Enter) ekleyin ve kendi profilinizdeki **İzleme listesi** altında bulun.
- **Her girdi şu anki durumunu söyler**: sunucuda, “26 bölümden 5”, istendi, “Geliyor: Cuma”, sunucuda değil, kütüphaneden ayrıldı ya da izlendi. Her baktığınızda, dizi ilerlemenizle aynı “izlendi” okumasıyla hesaplanır. İzlenen içerikler, siz kaldırana kadar kendi gruplarında kalır.
- **Henüz sunucunuzda olmayan bir içerik**, doğrudan Sonarr’ın ya da Radarr’ın beklediklerinden listeye eklenebilir. Geldiğinde içeriğe kendiliğinden bağlanır ve yeniden adlandırılan bir dosya girdisini de yanında götürür.
- **“İzleme listenizde: artık mevcut”**: beklediğiniz bir şey geldiğinde kendi hedeflerinize giden bir bildirim. Siz işaretleyene kadar kapalı.
- **İzleme listenizi başka kimse göremez**, bir yönetici bile; bildirimi de yalnızca sizin hedeflerinize gider. Jellyfin’e, Seerr’e, Sonarr’a ya da Radarr’a hiçbir şey yazılmaz.
- İzleme listeleri yedeğin parçasıdır ve geri yükleme hiçbir şeyi ikiye katlamadan onları birleştirir.

### Changed
- `/api/upcoming` girdileri içeriklerinin TVDB ve TMDB kimliklerini taşır ve `/api/auth/me` sunucunun izleme listesi tutup tutmadığını söyler.

## [2.0.4] - 2026-09-29

### Added
- **Görev zamanlaması, Jellyfin’in yaptığı gibi**: FinStats’in kendi başına yaptığı her işin tetikleyicileri vardır (her gün belirli bir saatte, her hafta belirli bir günde, belirli aralıklarla, başlangıçta ya da Jellyfin’in kütüphane taramasından sonra) ve her birinin isteğe bağlı bir süre sınırı olabilir. Zamanlamasını görmek ve değiştirmek için bir göreve tıklayın; *Tetikleyici ekle*, Jellyfin’dekiyle aynı iletişim kutusunu açar.
- **Görevler, Ayarlar’da kendi başına bir bölümdür**; her işin son çalışması sözle (“Son çalışma 38 dakika önce, 6 dakika sürdü”) ve bir Çalıştır düğmesiyle gösterilir; eskiden Sistem’in en üstünde duruyordu.
- **Meta veri değişiklikleri**: FinStats her saat Jellyfin’in son bakıştan bu yana kaydettiklerini (adlar, özetler, türler, derecelendirmeler, oyuncular ve ekip, dosya ayrıntıları ve resimler) okur, böylece Jellyfin’de yapılan bir düzenleme bir kütüphane taraması beklenmeden görünür.
- Yedekleme ve konum veritabanı da Görevler’den çalıştırılabilir; Esc bir görevin zamanlamasından listeye geri döner.

### Changed
- **Tema artık üç duraklı bir anahtardır**: Açık · Cihaz · Koyu; aralarında dolaşan düğmenin yerinde. Üçte birine tıklayın, topuzu sürükleyin ya da ok tuşlarını kullanın. Topuz seçimin simgesini taşır ve seçim değiştiğinde simge kayarak gelir.
- **Oturumu kapatma artık Ayarlar → Hesap altında**; bu bölüm burada kimin oturum açtığını da söyler; kenar çubuğu tema anahtarını tutar.
- **Eski zamanlama ayarları artık tetikleyicilerdir**: son çare aralığıyla birlikte *Jellyfin’in kütüphane taramasını izle*, *Her N günde bir yedekle* ve GeoIP’nin *Veritabanını güncel tut* anahtarı, sizdeki değerlerle, görevlerindeki karşılık gelen tetikleyicilere dönüştü. Toplama, Yedekler ve Güvenlik bunun yerine zamanlamaya bağlantı verir.
- Bir başlatma artık her işi yeniden çalıştırmıyor: her birinin en son ne zaman bittiği saklanır.
- Henüz tahmini olmayan bir Jellyfin işinde “Kalan süre…” yerine, sürenin geleceği yerde noktaları dönen “yaklaşık … kaldı” yazar.

### Performance
- **Güvenlik haritası artık takılmıyor**: sürükleme ve yakınlaştırma zaten çizilmiş haritayı taşır ve hareket bittiğinde onu bir kez çizer; şimdi oynatılıyor nabzı da kimse dokunmazken tüm haritanın saniyede 120 kez yeniden boyanmasına artık yol açmıyor. Yavaş bir bilgisayarda sürükleme saniyede 34 kareden 60 kareye çıktı.

### Fixed
- **Jellyfin’de değiştirilen bir poster ya da portre FinStats’te de görünür**: resimler, Jellyfin’de o arada ne olursa olsun, sunucuda bir hafta ve tarayıcıda bir hafta daha önbellekte tutuluyordu. Artık Jellyfin’in resim etiketi altında önbelleğe alınır, böylece yeni bir resim yeni bir dosyadır ve tarayıcı her gösterdiğinde kopyasının güncel olup olmadığını denetler. Bir portreyi değiştirmek yalnızca kişiyi yeniden kaydeder, bu yüzden Meta veri değişiklikleri kişileri de okur.
- **Sunucu → Görevler, sayfa açılır açılmaz bir tahmin gösterir**: kalan süre yalnızca sayfanın açıkken kendi yaptığı okumalardan ölçülüyordu, bu yüzden her ziyaret yüzdenin ilerlediğini görene kadar “Kalan süre…” ile başlıyordu. FinStats artık çalışan bir Jellyfin işinin süresini kendisi ölçer (biri çalışırken her 10 saniyede bir) ve sayfaya tahmin hazır sunulur. Jellyfin’in baştan sona %0 bildirdiği bir iş (bir kütüphane taramasının ilk aşaması) ilerleyene kadar noktalarını korur: hiçbir şey uydurulmaz.
- **Harita ipuçları noktanın üzerinde görünür**: Firefox’ta şimdi oynatılıyor noktasının ipucu çok uzağa düşebiliyordu ve noktaların üst üste bindiği yerlerde (evde oynatan biri) hangi ipucunun görüneceği bir piksele bağlıydı. Artık tek bir ipucu işaretçinin altındaki her şeyi adlandırır.
- **Durum çubuğunun tek bir üzerine gelme biçimi var**: her bağlantı (akışlar, Depo, sürüm) aynı küçük kutu olarak yanar ve öğeler arasındaki noktalar boşluklarda ortalanır. Sürümün üzerine gelmek eskiden önündeki noktayı ve boşluğu da yakıyordu.
- **Bir dosya yolu kartının tüm genişliğini kaplar**: bir içeriğin Dosya kartında ve Sunucu sayfasındaki Jellyfin klasörlerinde yol dar tek bir hücreye sıkıştırılıyor ve boş alanın yanında dokuz satır boyunca aşağı akıyordu.
- Kaybolmakta olan bir iletişim kutusu artık altındaki sayfaya yapılan bir tıklamayı yutmuyor.

## [2.0.3] - 2026-09-28

### Security
- Sunucu sayfası her cihazın kimliğini *Sunucuyu görme* iznine sahip herkese veriyordu; kimlikler, diğer her yerde olduğu gibi *Ağ ayrıntılarını görme* iznini de gerektirir.
- Ilki kaydedilmeden çevrilen iki izin anahtarı aynı başlangıç listesini iki kez gönderiyordu ve ikincisi ilkini geri alıyordu: iki anahtar da seçileni gösterirken geri alınan bir izin verilmiş olarak kalıyor, kapatılan bir herkese açık profil bölümü yayında kalıyordu.

### Stability
- **Aynı anda yalnızca bir içe aktarma ya da geri yükleme**: birlikte gönderilen iki yüklemenin ikisi de denetimi geçiyordu ve geri yükleme içe aktarmalara hiç bakmıyordu; ikincisi veritabanını bekliyor ve dakikalarca süren yüklemeden sonra başarısız oluyordu.
- **Bozuk bir `%` kaçışı içeren bir adres bulunamadı sayfasını açar**, sonsuza dek dönen bir yükleme simgesi bırakmaz.
- **Bir listedeki büyük bir sayfa numarası boş bir sayfadır**: taşma yapıyor ve yeniden ilk sayfa olarak geliyordu.

### Fixed
- **Tablolar Almanca ya da Norveççe bir tarayıcıda sayıları doğru sıralar**: “1,204” bin iki yüz dört değil ondalık bir sayı olarak, bir milyonu aşan her şey ise metin olarak okunuyordu.
- **Bir grafiğin Tablo görünümü tarihleri, hafta günlerini ve konumları zamana göre sıralar**, harflerine göre değil.
- **İndirme geçmişi içe aktarmaların nereden geldiğini söyler**: Sonarr ve Radarr dizinleyiciyi ve protokolü yalnızca yakalamada belirtir, bu yüzden bu dökümler hep boştu. Başarısız bir sezon paketi bölüm başına bir değil, tek bir hatadır.
- **Birlikte izlenen bir oynatmayı silmek** o içeriğin gruplarını yeniden bulur, tek kişilik bir “grup” bırakmaz.
- **Bir kişi için birlikte geçen süre, kaldığı süreyi asla aşmaz**: Birlikte sayfası ona akşamın tamamınınkini yazıyordu.
- **Yılbaşı gecesi gece yarısını aşan bir birlikte izleme akşamı** (ya da “son N gün”ün sınırını aşan) iki ayrı yalnız akşam yerine bütünüyle başladığı pencereye aittir.
- **İçe aktarılan bölüm oynatmaları bölümün adıyla anılır**, Jellystat’in yalnızca dizinin adını sakladığı yerlerde de.
- **Konum belirleme artık kendini kapatmıyor, şehir veritabanını Country ve ASN veritabanlarının yanında da buluyor**; bu ikisini `geoipupdate` yazar.
- **En çok geri sarılan sıcak nokta sayfanın filtrelerine uyar**, sayıları gibi.
- **“Yılınız hazır”** yalnızca yıl özetinde bir şey olan kişilere gider.
- **Sessize alınmış bir uyarıyı yeniden çözüldü saymak, çiftini sessizde tutar**; sessizi yalnızca yeniden açmak kaldırır.
- **Süresi dolmuş bir API anahtarı artık yirmi anahtar sınırına sayılmıyor.**
- **Oturum sınırı en eski oturumu bırakır**, çok sayıda oturum açma aynı saniyeye denk geldiğinde bile.
- **Sorgunun bulduğu bir ev adresi yerinde kalır**, aynı adres elle yazılan listeden çıkarıldığında bile.
- **Denetim günlüğünde ve anahtar listesinde resimler**, eskiden yalnızca baş harflerin olduğu yerde.
- **Bir yöneticinin bildirim listesi kişisel bir hedefin kime ait olduğunu söyler**, herkesinkinde “Sizin” yazmaz; **Son gönderilenler** son olayının her hedefini gösterir.
- **Bir içerikte ya da kişide Esc asla FinStats’ten çıkmaz** ve yalnızca en üstteki iletişim kutusunu kapatır.
- **Sayfa, beklediği ön getirmeyi bir kaydetme iptal ettiğinde bile çizilir.**
- **Şimdi oynatılıyor, duraklatılmışken ileri geri saran bir izleyiciyi gösterir**: oynat’a bastığı anda değil, şu an bulunduğu yerde.
- **Bir bağlantıyı kaldırmak hangi sayfaların var olduğunu günceller**, birini kaydetmenin yaptığı gibi.
- **Konum veritabanının derleme tarihi yılını korur**, ABD İngilizcesinde de.

## [2.0.2] - 2026-09-28

### Security
- Bir vekil arkasında oturum açma sınırı, istemcinin gönderdiği ilk `X-Forwarded-For` adresini sayıyordu: her denemede uydurulmuş yeni bir adres gönderiliyor ve sınır hiç uygulanmıyordu. Artık vekilin yazdığı son adres sayılır.
- *IP adreslerini ve konumları ekle*, sahibinin *Ağ ayrıntılarını görme* izni olmadığı kişisel bir hedefe, hiçbir sayfanın ona göstermeyeceği adresleri gönderiyordu. Başka birinin başarısız oturum açmaları artık Güvenlik sayfasındaki gibi *Sunucuyu görme* iznini de gerektirir.

### Stability
- **Bir durdurma asla kaybolmaz**: başlatmadan hemen sonraki ilk anlarda, sunucu henüz onu dinlemeye başlamadan bir kütüphane okuması reddedildiğinde ret not ediliyor ve FinStats durmak yerine çalışmaya devam ediyordu.
- **FinStats’in okuyamadığı bir biçimde yanıt veren bir Jellyfin reddedilir**, ona güvenilmez: her kullanıcının ya da bir kütüphanenin her içeriğinin bilmediği anahtarlar altında gelmesi (`id` olarak, `Id` değil) korumayı tam bir sayıyla geçiyor ve sonra hepsi kaldırılmış olarak işaretleniyordu.
- **Bir oynatmanın sonu asla düşürülmez**: veritabanı bekleme süresini aşacak kadar meşgulken (onu tutan bir içe aktarma), o anda biten bir oynatma bir sonraki yeniden başlatmaya kadar oynatılıyor olarak işaretli kalıyordu. Artık bir sonraki turda yazılır.
- **Geri yüklemenin okuyamadığı ayarlara dokunulmaz**: ayarları artık bu sürüme uymayan bir yedek, geri yüklemede her ayarı sıfırlıyordu: birleştirme süresi, izinler, ev adresleri, zamanlama.
- **Bir kişi adındaki ya da içerik adındaki denetim karakteri artık kartları bozmuyor**: profil kartı, o içeriği gösteren her hikâye kartı ve onların ZIP dosyası bir hatayla yanıt veriyordu.

### Fixed
- **Jellyfin’de devre dışı bırakılan ya da silinen biri için bildirimler durur**: kişisel hedeflerine eski izinlerinin izin verdiği her şey gönderilmeye devam ediyordu.
- **Kuyruktaki bir bildirim gönderilmeden önce yeniden sorgulanır**: kapatılmış bir hedefe ya da bir iznini kaybetmiş bir sahibe, yeniden denenmeyi bekleyen şey hâlâ gönderiliyordu.
- **Her zaman `Retry-After` ile yanıt veren bir hedeften vazgeçilir**: otuz gün boyunca yeniden denenmek yerine, diğerleri gibi beş denemeden sonra.
- **Bir kişinin sayfasındaki cihaz kimlikleri**, diğer her yerde olduğu gibi yalnızca *Ağ ayrıntılarını görme* izniyle gösterilir.
- **Yayımlanan bir yıl her şey için bir gün bekler**: bitirilen ve yarıda bırakılan diziler her oynatmayı anında izliyordu, bu yüzden yarıda bırakılmış bir dizide oynat’a basmak herkese açık sayfayı saniyeler içinde değiştiriyordu.
- **İstenen bir dizi, bölümlerinden biri izlendiğinde izlenmiş sayılır**: İstekler bölümünde ve İstekler kartında istenen her dizi hiç izlenmemiş görünüyordu.
- **Yıl özetinin Birlikte bölümü Canlı TV’yi dışarıda bırakır**, yılın geri kalanı gibi: birlikte açık bırakılan bir kanal en çok izlenen içerik olabiliyor ve pay %100’ü geçebiliyordu.
- **“Son 12 ay”ın Günler kartı en yeni günde biter**: ızgara en son üç dört haftayı düşürüyordu.
- **Bir dizinin yeni bölümleri akşam başına bir kez duyurulur**: UTC gece yarısının iki yanına düşen bölümler, UTC’nin doğusundaki ya da batısındaki bir sunucuda iki kez duyuruluyordu.
- **Seerr’i bağlamak eski istekleri izlemeye hazır olarak duyurmaz**: ilk okunduğunda zaten mevcut olan ve ne zaman mevcut olduğuna dair başka bir kaydı bulunmayan bir istek o anda gelmiş gibi okunuyordu.
- **Jellyfin’de silinen bir kütüphanenin içerikleri onunla birlikte gider**: aramada, Son eklenenler rafında ve toplamlarda kalıyorlardı.
- **Bir remux doğrudan akış olarak kalır**, bir okuma kod dönüştürme ayrıntılarını bir an kaybettiğinde (bir sarma) oynatmanın geri kalanı için kod dönüştürmeye dönüşmez.
- **“Jellyfin çalışmıyor” art arda başarısız okumalar demektir**: sokette boşta bekleyen bir sunucuda günler arayla gelen rastgele zaman aşımları toplanarak bu sonuca varıyordu.
- **Bir Streamystats içe aktarması ilerlemesini kendi kartında gösterir**, Jellystat içe aktarmasının son sonucunun üzerinde değil.
- **Ana makine adı “web” ile başlayan bir adres** (`http://web:8096`), Jellyfin’in web istemcisi yolu sanılıp kırpılmaz, ana makine olarak okunur.
- **Bir kuyruk hatası bir adresin tüm oturum bilgilerini gizler**, parola eğik çizgi içerse bile.

## [2.0.1] - 2026-09-28

### Changed
- **Bir içeriğin dilleri yolu kapatmaz**: başlık alanı ilk üç ses ve altyazı dilini her biri tek satırda adlandırır ve yeni bir **Diller** kartı hepsini listeler: tam olanlar bir arada, kısmi olanlar kaç bölümde bulunduklarıyla. Uzun bir dizinin iki düzine altyazı dili görseli bir ekran aşağı itiyordu.

### Fixed
- **Bir içeriğin posteri adının yanında kalır**: yanındaki metin posterden uzun olduğunda (uzun bir özet, çok sayıda stüdyo), poster başlık alanının dibine, boş görselin altına batıyordu.
- **FinStats’in okuyamadığı bir sürüme öyle denir**: elle okunamaz bir şeye dönüştürülmüş bir `app_version`, FinStats’in “daha eski FinStats 2.0.1” diyerek başlamayı reddetmesine yol açıyordu, oysa öyle değildi. Hâlâ reddeder ve artık hangi değeri okuyamadığını ve nasıl yazılması gerektiğini söyler.

## [2.0.0] - 2026-09-25 to 2026-09-28

Yıl özeti 2026, herkese açık profiller ve birlikte izleme

### Added
- **Kiminle izlediğiniz**, yıl özetinde: birlikte geçen akşamlar, bunlara harcanan süre, insanları en çok bir araya getiren içerik ve uygulamada adı verilip profillerine bağlanan en fazla üç arkadaş.
- **Sonuna kadar izlenen diziler**, yıl özetinde: o yıl her birinin izlenen son bölümü, profilin ilerleme çubuklarıyla aynı “izlendi” okumasıyla; ayrıca başlanıp sonraya bırakılanlar: yarısından azı izlenmiş ve iki aydır hiçbir şeyi oynatılmamış olanlar.
- **Ne istediğiniz**, yıl özetinde: yıl içinde yapılan Seerr istekleri, kaçının geldiği ve kaçını sonra izlediğiniz. Yalnızca isteklerin bulunduğu yerde.
- **Yıl, bir öncekine karşı**: saatler, oynatmalar ve günler ve ne kadar fazla ya da az olduğu.
- **Bir hikâye olarak yıl**: her bölüm, telefon hikâyelerinin kullandığı biçimde 1080×1920 boyutlu bir kart olarak (yıl, rakamlar, en çok izlenen diziler, filmler ve müzik, türler, izleme tarzı, saatler ve günler, yılın her günü, rekorlar, birliktelik, bitirilen diziler, istekler ve önceki yıl). “Yılı kartlarla paylaşın” düğmesinden birini ya da hepsini bir ZIP olarak kaydedin.
- **Bir kart asla başka kimsenin adını vermez**: kartlar, yılın başka bir kişi, bir sıralama ya da bir uygulama için alanı olmayan bir kopyasından çizilir. Uygulamada bir kart hiç ad taşımaz.
- **Sunucunun yılı**, Jellyfin yöneticileri için: her içerik ve toplam ve evin izleme tarzı; kimse sıralanmaz ya da adlandırılmaz. Yıl özeti sayfasında bir anahtar, API’de `scope=server`.
- **“Yıl özetiniz hazır”**: aralık ayında, o yıl izleyen her kişiye bir kez giden bir bildirim.
- **Herkese açık profiller**: bir kişi profilinin bir kısmını, herkesin hesap olmadan açabileceği bir bağlantıda yayımlayabilir (toplamlar ve en çok izlenen içerikler, seriler ve ne zaman izlediği, kart hikâyesi olarak tüm yıl, son on oynatma); her kısım, sahibi Ayarlar → Herkese açık profil altında açana kadar kapalıdır. Cihazlar, adresler, uygulamalar, oynatma yöntemleri, dosya yolları ve diğer kişiler, ne açılmış olursa olsun asla yayımlanmaz ve gösterilen ad sahibinin yazdığı bir addır, asla Jellyfin kullanıcı adı değildir.
- **Yayımlanan bir sayfadaki hiçbir şey bir günden yeni değildir**, son oynatmalar da her toplam da; böylece birinin göz kulak olduğu bir sayfa şu anda kimin izlediğini asla söylemez.
- **Paylaşılabilir kartlar**: yayımlanan her profil, profilin ve yılın 1200×630 boyutlu bir resmiyle gelir; bağlantı yapıştırıldığında Discord, Signal, Mastodon ve diğerleri bunu gösterir ve paylaşmak için indirilebilir. Sunucuda, yalnızca sayfanın gösterdiklerinden çizilir.
- **Geri alabileceğiniz bir bağlantı**: rastgele 22 karakter, tek tıkla sıfırlanır; eski bağlantı bundan sonra hiç var olmamış bir bağlantı gibi yanıt verir.
- **Herkese açık profiller bir yönetici izin verene kadar kapalıdır**: tüm sunucu için tek bir anahtar, kimin neyi yayımladığının listesi ve bir profili yayından kaldırmanın yolu. Yayımlamalar, değişiklikler, sıfırlamalar ve kaldırmalar denetim günlüğündedir ve bir API anahtarı hiçbir şeyi herkese açık yapamaz.
- **Birlikte**, kendine ait bir sayfa: kim kiminle, ne kadar izliyor ve bu nasıl değişti. Birlikte geçen süre, akşamlar ve izleme süresinin birlikte geçen payı, her biri önceki pencereye karşı; gün ya da hafta başına birlikte geçen saatlere karşı yalnız geçen saatler; her kişi çifti, akşamları, birlikte geçen süreleri, en son ne zaman izledikleri ve en çok ne izledikleriyle; en çok birlikte izlenen içerikler; son akşamlar; ve herkesi görebilen herkes için her kişinin yalnız geçen süresine karşı birlikte geçen süresi.
- Her çift sayılır: üç kişilik bir akşam üç çiftin her biri için sayılır; kart eskiden yalnızca tam olarak aynı kişi kümesini sayıyordu. Bir çiftin birlikte geçen süresi, iki kalış süresinden kısa olanıdır.
- Panodaki ve profildeki **Birlikte izlendi** kartı sayfaya bağlanır ve bir profilin kartı o kişinin izleme süresinin ne kadarının birlikte geçtiğini söyler.
- `GET /api/stats/groups` `pairs`, `series`, `people`, `previous`, `bucket` ve toplamlarında `watch_s` ile `share` kazanır; mevcut anahtarlar değişmez.
- **Kişilerin bıraktığı yer**, her film ve bölüm sayfasında: baştaki herkesten sona ulaşanlara kadar, her dakikada hâlâ izleyen oynatmaların payı; bu oynatmaların geri sarmaları ve altyazı açmaları aynı eksende işaretli. Kart kaç durmanın ölçüldüğünü (FinStats’in kaydettiği oynatmalar) ve kaçının tahmin edildiğini (ne kadar sürdüklerinden, içe aktarılan oynatmalar) söyler ve bir şey çizmeden önce üç oynatma ister.
- **Kim izlemeye devam ediyor**, bir dizi sayfasında: her bölüme sırayla kaç kişinin başladığı ve bölüm başına bir **Bitirilen** sütunu. “Herkes üçüncü bölümde bırakıyor”, dördüncü bölümde hiç oynat’a basmayan kişiler demektir.
- **Hiç oynamayan dosyalar**, Oynatma sayfasında: üç ya da daha fazla kez başlatılıp hiç otuz saniyeyi geçemeyen bir film ya da bölüm; kimin denediği ve hangi uygulamalarda. Jellyfin’in günlüğü oynatma hatası taşımaz, bu yüzden tanık oynatmaların kendisidir; en kısa oynatma süresi burada uygulanmaz, çünkü bu oynatmaların asla ulaşamadığı şey tam olarak odur.
- **En çok geri sarılan** ve **Açılan altyazılar**, Oynatma sayfasında: oynatma başına geri atlamalara göre sıralanan içerikler ve bunların yoğunlaştığı dakika; bir de ilk on dakika içinde altyazının açıldığı oynatmaların payına göre sıralananlar. Üç listenin üçü de *Herkesin etkinliğini görme* iznini gerektirir. 20 saniyeden kısa atlamalar hiç kaydedilmedi, bu yüzden tek bir repliğin kısa tekrarı tasarım gereği görünmez.
- Bir sarma, nereden geldiğini etiketinin yanında bir sayı olarak saklar (bir oynatmanın zaman çizelgesinde `from_s`). Zaten kaydedilmiş her sarma bu sayıyı etiketten alır; bundan önceki bir zamandan geri yüklenen bir yedek de öyle.
- Üç liste için `GET /api/stats/files`; `GET /api/items/{id}` `insights` kazanır, bir dizinin bölümleri de `users` ve `finished` kazanır.
- **API anahtarları**: oturum açan herkes Ayarlar → API anahtarları altında kendisi için bir betik, bir pano ya da bir telefon için anahtarlar oluşturabilir. Bir anahtar `Authorization: Bearer fs_…` olarak gönderilir, bir kez gösterilir, karması alınarak saklanır, bir son kullanma tarihi taşıyabilir ve bir tıkla iptal edilir; bir yönetici herkesinkini görür ve iptal edebilir. Oturum açmayla aynı kişiye aynı izinlerle çözülür ve her istekte canlı okunur; bu yüzden oturum açma hakkını kaybetmek her anahtarı bir anda durdurur. Bir anahtar ne anahtar oluşturabilir ne de iptal edebilir.
- **Takvim akışı**: `GET /api/calendar.ics?key=…`, Sonarr ve Radarr’ın yakında getireceklerini bir telefonun abone olabileceği bir iCalendar olarak sunar. *calendar* kapsamlı bir anahtar akışı açar ve başka hiçbir şeyi açmaz; abone olunan bir takvim başlık gönderemediği için akış anahtarı adresten okur, asla çerezden okumaz ve kimseyi adlandırmaz.
- **Denetim günlüğü**, Jellyfin yöneticileri için Sunucu → Denetim altında: adresiyle birlikte her oturum açma ve başarısız deneme, değiştirilen her ayar ya da izin ve neye değiştirildiği, oluşturulan, kullanılan ya da iptal edilen her anahtar, her bağlantı, hedef, yedek ve içe aktarma, silinen her oynatma ve çözülen her uyarı; kimin, nereden, hangi anahtar üzerinden yaptığı ve işe yarayıp yaramadığıyla. Bir yıl saklanır, yedeklerin parçasıdır. `GET /api/audit` sayfa sayfa gezinir.
- **Streamystats’ten içe aktarma**, Ayarlar altında Jellystat’in yanında kendi bölümü. Streamystats’in **Settings → Backup & Import → Download Backup** dosyasını alıp bırakın: yılların geçmişi birkaç saniyede gelir ve zaten sahip olduğunuz hiçbir şey yeniden sayılmaz. `finstats import-streamystats <file>` bunu arayüzsüz, `POST /api/import/streamystats` API üzerinden yapar; aynı anda hangi türden olursa olsun yalnızca bir içe aktarma çalışır.
- **İki izleme aracını da mı kullandınız? İki dosyayı da içe aktarın.** Bir oynatma, FinStats’te zaten olan bir oynatma olarak önce izleme aracının ona verdiği kimlikle, bu olmazsa aynı öğeyi izleyen aynı kişi ve oynatmanın iki ucundan birinin geçmişteki bir oynatmanınkine yakın olmasıyla tanınır; böylece aynı akşamların bir Jellystat ve bir Streamystats dışa aktarması, herhangi bir sırayla, ikisi de içe aktarılabilir ve FinStats’in kendisinin izlediği bir akşamın üzerine de içe aktarma yapılmaz. “Yakın”, Ayarlar → Toplama altındaki birleştirme süresidir.
- İçe aktarılan satırlar neyi söyleyemediklerini açıkça belli eder. Streamystats’in kendisinin izlediği bir oynatma dosya hakkında hiçbir şey tutmaz (codec, çözünürlük, bit hızı ya da kapsayıcı yok), bu yüzden bunlar bir tahminle doldurulmak yerine boş kalır ve bunları gösterecek kartlar kendilerini gizler. Jellystat’ten aldığı bir oynatma ise oturumun tamamını taşır ve onun yerine o okunur: iki izleme aracının da sakladığı tek an olan bittiği anla tarihlenir ve yalnızca oturum gerçekten bir kod dönüştürmeyse kod dönüştürme olarak tanımlanır.
- Streamystats’in, Jellyfin bir öğeyi izlendi olarak bildirdiği için hiç görmediği bir izleme adına yazdığı satırlar sayılır ve atlanır: tüm film kadar uzundurlar ve kimsenin izlemediği bir akşamda izleme süresi olurlardı. Sonuç bunlardan kaç tane olduğunu söyler.
- Streamystats verisinin alan alan nasıl yorumlandığı, belgelerden değil gerçek bir dışa aktarmadan öğrenilmiş haliyle: `docs/streamystats-import.md`.
- **Her oynatmanın hangi izleme aracından geldiği**, Etkinlik’te bir filtre olarak: FinStats’in kendisinin kaydettikleri, Jellystat’ten gelenler, Streamystats’ten gelenler. Yalnızca geçmişinizin gerçekten içerdikleri sunulur ve diğer her liste gibi kapsamlanır; böylece yalnızca kendi oynatmalarını görebilen biri yalnızca kendi geçmişinin nereden geldiğini öğrenir.
- **Etkinlik’teki filtreler, işaretleyerek seçtiğiniz açılır listelerdir**; böylece her birinden bir tane ya da hepsi yerine müzik olmadan filmler *ve* bölümler, iki oynatma yöntemi ya da aynı anda iki kişi isteyebilirsiniz. Medya türü artık birleştirilebildiği için **Diğer** seçeneğini kazandı.
- Kullanıcı filtresinin göründüğü her yerde (pano, oynatma, kütüphaneler, güvenlik ve diğerleri) aynı anda birkaç kişi seçilebilir; harita ve her istatistik buna uyar. Bir kullanıcı filtresi hâlâ yalnızca birinin görebileceklerini daraltır: *Herkesin etkinliğini görme* izni olmadan bir istek, adres kimi ve kaç kişiyi belirtirse belirtsin, isteği yapan kişiye sabitlenir.
- **Açık tema**: mürekkep renkli metin ve vermilyon vurgulu washi kâğıdı. FinStats, siz birini seçene kadar cihazın açık ya da koyu ayarına uyar.
- **Oturumu kapat düğmesinin yanında bir tema düğmesi** Cihaz, Açık ve Koyu arasında döner. Seçim bu tarayıcıda saklanır ve sayfa çizilmeden önce uygulanır, böylece diğer tema asla bir an bile görünmez.
- **Jellyfin’de aç**: her film, dizi ve bölüm sayfasında o içeriğin Jellyfin’deki sayfasını yeni sekmede açan bir düğme var.
- **Kullanıcılar için Jellyfin adresi**, Ayarlar → Jellyfin altında: FinStats’in bağlandığı adres bir tarayıcının ulaşabileceği bir adres olmadığında bu düğmelerin gittiği yer. Boş bırakılırsa FinStats’in bağlandığı adres kullanılır; yalnızca bir Jellyfin yöneticisi ayarlayabilir.

### Changed
- **FinStats’in yeni bir evi var**: `github.com/finstats/finstats`; imaj ise `ghcr.io/finstats/finstats`. Güncellemeleri almaya devam etmek için `docker run` komutunuzdaki ya da compose dosyanızdaki imaj adını değiştirin; veri klasörü olduğu gibi kalır.
- **Ayarlar’da ekranda aynı anda tek bir bölüm görünür.** Sayfa tek sütunda on dört kart ve altı yüz sözcük yardımdan oluşuyordu. Artık her bölümün kendi adresi var (`/settings/collection`), soldaki bir liste aralarında gezinir ve telefonda liste küçük düğmelere dönüşür. Eski çapalara giden bağlantılar hâlâ doğru yere iner.
- **Bir ayar tek satırdır**: ne olduğu, kuralı belirten tek bir satır, sağda denetim. Bir bölümdeki sayılar alttaki tek bir düğmeyle birlikte kaydedilir; anahtarlar hâlâ kendiliğinden kaydedilir. Durum kartları (Görevler, Giden bağlantılar, Veritabanı) ve lisans bağlantısı **Sistem** altında yer alır ve iki içe aktarıcının kendi bölümü vardır.
- **Bir ayarı başlıktaki kutudan bulun** (`/` ona odaklanır), adındaki herhangi bir sözcükle; ayarın hangi bölümde olduğunu söyler ve Enter satıra götürür.
- **Ayarlar oturum açan herkes içindir**: sunucuyu yöneten bölümler onu yönetebilenlerde kalır ve yalnızca bildirimlere izni olan biri sonunda kendi hedeflerine ulaşabilir.
- **Sunucu sayfasında da ekranda aynı anda tek bir bölüm görünür**, Ayarlar’daki gibi: Genel bakış (sistem ve depolama), Görevler, Cihazlar, Eklentiler, Günlük ve Denetim, her biri kendi adresinde. `/server#jobs` bağlantıları hâlâ doğru yere iner.
- **Sunucu günlüğü Sunucu sayfasının bir bölümüdür** (`/server/log`), menüde kendine ait bir sayfa değil; `/events` filtreleriyle birlikte oraya yönlendirir.
- Başarısız bir iş, başarısız bir yedek ya da yanıt vermeyi bırakan bir bağlantı hakkındaki bildirim, sayfanın en üstüne değil, onu barındıran ayar bölümüne bağlanır.
- Kütüphanenizin içe aktarılan bir oynatma hakkında söyleyebildikleri (ne tür bir şey olduğu, hangi bölüm olduğu, süresi, hangi kütüphaneye ait olduğu) yalnızca içe aktarma anında değil, **her** kütüphane okumasından sonra doldurulur. Geçmiş genellikle FinStats kütüphaneyi hiç okumadan önce içe aktarılır, bu yüzden o oynatmalar kalıcı olarak tahmini ya da bilinmeyen kalıyordu.
- Bir FinStats yedeğini geri yüklemek artık bir izleme aracından farklı bir kimlikle gelmiş bir oynatmayı eklemiyor ve aynı şeyin dakikalar arayla yapılmış iki gerçek izlenmesini hâlâ korur.
- Bir oynatmanın ayrıntıları, içe aktarılan her şeye Jellystat oynatması demek yerine onu hangi izleme aracının getirdiğini söyler.

### Performance
- **Bir yeniden başlatma, geçmiş ne kadar büyük olursa olsun neredeyse anlıktır.** Her başlatma tüm geçmiş üzerinden kimin birlikte izlediğini hesaplıyor ve hangi oynatmaların yerel olduğuna yeniden karar veriyordu, ikisi de hiçbir şeyi değiştirmemek için: on milyon oynatmada 24 sn ve 5,5 sn. Bir başlatma artık yalnızca son çalışmanın geride bıraktığını yapar; tamamını yalnızca bir güncellemeden sonra ya da arkasındaki ayar değiştiğinde yeniden yapar.
- **Aynı anda gelen bir kalabalık saniyeler içinde kaydedilir.** Her yeni oynatma, her ilerleme kaydı ve her cihaz ayrı ayrı yazılıyordu ve biten her oynatma, içeriğin tüm geçmişine ayrı bir bakışla kapatılıyordu; bir tur artık her türü tek seferde yazar. Birlikte başlayan 100.000 oynatma 3 sn sürer; eskiden birlikte biten 25.000 oynatma dört dakikadan fazla sürüyordu.
- **Şimdi oynatılıyor, binlerce kişi izlerken de hızlı kalır.** Kimin birlikte izlediğini bulmak her akışı diğer her akışla karşılaştırıyor ve her arkadaşı adıyla listeliyordu: 10.000 akışta kırk saniye ve bunun da ötesinde gigabaytlarca bellek. Akışlar artık bir içerik içinde karşılaştırılır ve her biri birkaç ad ile kaç kişinin daha olduğunu gösterir.
- Kimin birlikte izlediğini hesaplamak, FinStats her başladığında, içe aktardığında ya da bir oynatmanın bittiğini gördüğünde geçmişteki gruplanmış her oynatmayı yeniden yazıyordu; artık yalnızca değişeni yazar. Hangi oynatmaların bittiğini bulmak artık oynatılanların sayısının karesiyle yavaşlamıyor.
- **Geçmişte zaten olan bir oynatmayı tanımak bir dizin üzerinden yapılır**, böylece 150.000 oynatmalık bir içe aktarma saniyeler sürer ve bir kütüphane okumasından sonra yeniden bağlama, veritabanını tutarken her oynatmayı okumak yerine içerikler arasında adım adım ilerler.
- **Oynatma içgörüleri ve genel bakış tüm zamanlar için daha hızlı**: bağımsız kısımları yan yana hesaplanır, eş zamanlılık grafiği yerel günü oynatma başına bir kez yerine çeyrek saatte bir kez arar ve türler içerik başına sayılır. Bir milyon oynatmada: 4,1 sn’den 2,4 sn’ye ve 1,6 sn’den 1,1 sn’ye.

### Stability
- **Bir televizyon kütüphanesi artık içi boşaltılmış bir kütüphane gibi okunmuyor.** Bir kütüphaneyi okumak iki geçiş yapar (her öğe, ardından yalnızca filmlerin ve dizilerin oyuncuları ve ekibi için daha küçük bir geçiş) ve bir okumanın görmediğini kaldırma konusunda güvenilip güvenilemeyeceğine karar veren denetim ikinci geçişe bakıyordu. Bir televizyon ya da müzik kütüphanesinde iki sayı birbirine hiç yakın değildir, bu yüzden denetim her okumayı reddedip FinStats’i durduruyor, kurulum yeniden başlıyor ve aynısını tekrar yapıyordu. Hiçbir şey yanlışlıkla kaldırılmadı ama hiçbir kütüphane okuması tamamlanamadı. Diziler ya da Müzik kütüphaneniz yeni içerikleri almıyorduysa nedeni buydu.
- Bir içe aktarma, ilk okuması ile ilk yazması arasına başka bir yazma (yenilenen bir oynatma) girdiğinde “database is locked” hatasıyla başarısız olabiliyordu. İki içe aktarıcı da yazma kilidini başlarken alır ve veritabanının günlük dosyası artık şimdiye kadar yapılmış en büyük içe aktarma kadar büyük kalmıyor.
- **Hasarlı, yarım kalmış, başkasına ait ya da salt okunur bir veritabanı dosyası hemen reddedilir**; 30 sessiz saniye yerine dosyayı adlandırarak ve hiçbir şeyin değiştirilmediğini söyleyerek. Başka bir programın veritabanına asla yazılmaz.
- Yarıda kesilen bir yedek ya da güncelleme öncesi anlık görüntü artık geride bir dosya bırakmıyor; o dosya, saklanan üç anlık görüntü arasında sağlam bir anlık görüntünün yerini alabiliyordu.
- **FinStats çalışırken konum veritabanının üzerine yazmak artık onu çökertmiyor**: FinStats dosyanın özel bir kopyasından okur.
- Her seferinde yeni bir cihaz kimliği bildiren bir Jellyfin uygulaması, FinStats çalıştığı sürece belleğinin büyümesine yol açabiliyordu; beş dakikadır görülmeyen cihazlar artık unutulur.
- Bir kişi en fazla 30 açık oturum tutar; en eskisi bir sonraki oturum açmada kapatılır.

### Fixed
- **Bir öğe yeniden adlandırıldığında aynı akşam artık iki kez sayılmıyor.** Geçmişi Jellyfin’de yeniden eklenmiş bir içeriğe yeniden bağlamak, bir izleme aracından içe aktarılmış bir oynatmayı zaten orada olan bir oynatmanın kopyasına dönüştürebiliyordu, çünkü bunu yakalayacak denetim kimlik taşınmadan önce çalışıyordu. Artık kimliklerin yeniden bağlandığı her yerde yeniden çalışır. Yalnızca içe aktarılmış oynatmalar kaldırılır, FinStats’in kendisinin kaydettiği hiçbiri değil; kaç tanesinin gittiği günlükte yazar. 2.0’ın ilk başlatması, zaten oluşmuş olanlar için tüm geçmişi bir kez tarar.
- **Bir oturum artık kişiyi Jellyfin’deki şimdiki haliyle izler.** Jellyfin’in yetkisini düşürdüğü bir yönetici, 30 günlük oturumunun geri kalanında FinStats’e tam erişimini koruyordu ve Jellyfin’de devre dışı bırakılan ya da silinen biri oturumda kalıyordu. Artık FinStats Jellyfin’in kullanıcılarını okur okumaz (her 15 dakikada bir ya da Ayarlar’dan hemen) bunu kaybederler; API anahtarlarında hep olduğu gibi.
- Oturum açmak, Jellyfin’in kişi hakkında söylediklerini FinStats’in kullanıcı listesine yazar; böylece içe aktarılan bir izleme aracı yedeği, kullanıcıların ilk okuması bunu düzeltmeden önce birini yönetici yapamaz.
- Jellyfin’de silinen biri, kullanıcıların iki okuması aynı saniyeye denk gelse bile kaldırılmış olarak işaretlenir.
- **İndirme hataları asla bir indirme istemcisinin parolasını ya da bir dizinleyicinin API anahtarını göstermez**; Sonarr ve Radarr bunları hata iletilerinde aktarabilir.
- Bir kütüphane okuması sırasında yeniden adlandırılmış bir içeriğe yeniden bağlanan oynatmalar, bir sonraki yeniden başlatmaya kadar birlikte izleme açısından denetlenmiyordu; artık hemen denetlenir.

## [1.6.5] - 2026-09-24

### Added
- **Her yükseltmeden önce otomatik olarak tam bir yedek alınır.** Daha yeni bir FinStats veritabanınızı ilk kez açtığında, herhangi bir şeyi değiştirmeden *önce* veritabanının eksiksiz bir kopyasını (tüm geçmişiniz, ayarlar dahil her şey) `data/pre-update-backups/` klasörüne yazar. Bir yükseltme bir şeyi bozarsa verileriniz güvendedir: FinStats’i durdurun, kopyayı `finstats.db` yerine koyun, kullandığınız sürümü başlatın ve hatayı bildirin. Hiçbir şey kaybolmaz. En yeni birkaç kopya saklanır; disk alanı darsa kapatmak için `FINSTATS_SKIP_PREUPDATE_BACKUP=1` ayarlayın.

## [1.6.4] - 2026-09-24

### Added
- **Bildirim gönderilecek beş yer daha**: Telegram, Slack, Pushover, Pushbullet ve e-posta; zaten var olan webhook, Discord, ntfy ve Gotify hedeflerinin yanında. Her biri yalnızca ihtiyaç duyduğunu ister (bir sohbet kimliği, bir kullanıcı anahtarı, bir posta kutusu) ve tek bir hizmette yaşayan üçüne (Telegram, Pushover, Pushbullet) yalnızca orada ulaşılabilir, böylece bir belirteç benzer görünen bir ana makineye gönderilemez. Posta ilk bayttan itibaren TLS üzerinden ya da `STARTTLS` ile gider; üçüncü bir seçenek yoktur.
- **Lisanslar**, **Ayarlar** altında: FinStats’in kendi GPL lisansı ve üzerine kurulduğu her üçüncü taraf lisansı (253 Rust paketi, birlikte gelen iki yazı tipi, harita sınırları ve konum veritabanı), her metin eksiksiz, kendi projesinin yazdığı haliyle. Bir bileşen seçin, lisansı listenin yanındaki pencerede açılır; birkaç dosya içeren bir paket bunlar arasında yerinde geçiş yapar. Liste FinStats’in gerçekten derlendiği paketlerden üretilir, bu yüzden lisansı kaydedilmeden bir bağımlılık eklenemez.

## [1.6.3] - 2026-09-24

### Fixed
- Güvenlik koruması artık Jellyfin *hiç* kütüphane ya da *hiç* kullanıcı listelemediğinde de devreye giriyor (normal bir gün değil, bozuk bir okuma); 1.6.2’deki ayar bunu geçiriyordu. Bazı kütüphaneleri kaybetmek hâlâ olağan sayılır; hepsini bir anda kaybetmek sayılmaz.

## [1.6.2] - 2026-09-24

### Fixed
- 1.6.1’de eklenen veri bütünlüğü koruması fazla hevesliydi: öğelerinin çoğunu gerçekten kaybeden bir kütüphane (örneğin dosyaları silinen bir avuç klip) bozuk bir okuma sanılıyordu, bu yüzden FinStats onu güncellemeyi reddedip duruyordu. Artık yalnızca *açıkça* bozuk bir okumada devreye girer: önemli sayılacak kadar büyük bir kütüphanenin tamamen boş okunması ya da büyük bir kütüphanenin neredeyse hiçliğe inmesi. Gündelik değişiklikler eskisi gibi uygulanır ve “Jellyfin hiçbir şey döndürmedi” durumu hâlâ yakalanır.

## [1.6.1] - 2026-09-24

### Added
- **FinStats artık geçmişinizi silecek bir Jellyfin okumasına güvenmiyor.** FinStats’in binlerce öğe tuttuğu bir kütüphane birden boş ya da neredeyse boş okunursa (API’sini değiştiren bir Jellyfin yükseltmesi, boş sonuç kılığına girmiş bir hata), FinStats elindeki verileri korur, eşitlemeyi başarısız olarak işaretler (ve bildirir) ve her şeyi kaldırılmış olarak işaretlemek yerine açık bir iletiyle düzgünce durur. Jellyfin kendine geldiğinde yeniden başlatın ya da bir kütüphaneyi gerçekten boşalttıysanız `FINSTATS_ALLOW_LIBRARY_SHRINK=1` ayarlayın. Aynı koruma kütüphane listesini ve kullanıcı listesini de kapsar.

### Security
- Bir istemcinin gönderdiği `X-Forwarded-Host` başlığı yabancı bir `Origin` için kefil olabiliyor ve siteler arası bir yazmayı aynı köken korumasından geçirebiliyordu. Bu başlığa artık yalnızca güvenilen bir vekil arkasında güvenilir (`FINSTATS_TRUST_PROXY`); istemci adresinin belirlenmesinde zaten kullanılan kuralın aynısı. `SameSite=Lax` oturum çerezi ilk savunma hattıydı ve öyle kalıyor.
- Kişisel bir bildirim hedefi, bu makineye ulaşan `0.0.0.0` ya da `::` adresine yöneltilebiliyordu. Kişisel hedeflerin herkese açık adreslere yönelmesini sağlayan denetim artık bunları da, geri döngü ve özel aralıkların yanında, yerel sayar.

### Fixed
- **Oynatma** sayfası, yöntemi kaydedilmemiş bir oynatma içeren bir kütüphane için (sütundan daha eski bir yedekten geri yüklenmiş bir satır) artık yüklenememe hatası vermiyor; böyle bir oynatma basitçe doğrudan oynatma sayılır.
- Bir aramada ya da filtrede çok uzun bir sözcük artık sunucu hatası değil: SQLite 50.000 karakteri aşan bir `LIKE` desenini reddeder, bu yüzden filtre sözcükleri sorgudan önce makul bir uzunluğa kısaltılır.
- **Ayarlar** sayfası, bir kart bir geri yüklemenin sonucu gibi bütün bir cümle bildirdiğinde telefonda artık yana kaymıyor.

## [1.6.0] - 2026-09-24

Bildirimler

### Added
- Ayarlar altında **Bildirimler** kartı: bir hedef ekleyin, neleri duyması gerektiğini işaretleyin, Test’e basın. Hedef yoksa hiçbir şeye istek yapılmaz.
- Dört tür hedef: düz bir **webhook** (JSON, `docs/api.md` içinde belgelenmiş, isteğe bağlı `Authorization` başlığı), **Discord** (bir kanal webhook URL’si üzerinden bir embed), **ntfy** (ntfy.sh ya da kendinizinki, bir konuya) ve **Gotify** (kendi sunucunuz, bir uygulama belirteciyle). Film ve dizi adları bir HTTP başlığında değil, iletinin içinde taşınır.
- Her biri ayrı bir onay kutusu olan on bir tür olay: imkânsız seyahat, yeni bir ülkede görülen bir hesap, bir başarısız oturum açma dalgası, başarısız olan bir görev, yazılamayan bir yedek, yanıt vermeyi bırakan bir şey (Jellyfin, Sonarr, Radarr ya da Seerr) ve yeniden yanıt verdiği an, kütüphanedeki yeni içerikler (pano rafındaki gibi dizi ve gün başına tek girdide toplanmış), izlenebilir hale gelen bir istek ve başlayan ya da biten bir oynatma. Son ikisi varsayılan olarak kapalıdır.
- İzlenebilir hale gelen bir istek içeriği, kimin istediğini ve ne kadar beklediğini söyler.
- Kişisel hedefler: **Bildirim alma** iznini verin, o kişi kendi hedefini ekleyebilir; bu hedefe yalnızca o kişinin zaten görebildikleri gönderilir: kendi istekleri ve uyarıları her zaman, başkalarınınki *Herkesin etkinliğini görme* ile, adresler *Ağ ayrıntılarını görme* ile. Kişisel bir hedef herkese açık bir adresi göstermelidir; yalnızca bir yönetici ağın içini hedefleyebilir.
- **IP adreslerini ve konumları ekle**, hedef başına ve varsayılan olarak kapalı: aksi halde imkânsız seyahatle ilgili bir ileti yalnızca konumları adlandırır (“Oslo, Norveç ve Londra, Birleşik Krallık”).
- **Son gönderilenler**, aynı kartta: ne söylendiği, hangi hedefe ve ulaşmadığında karşı tarafın ne yanıt verdiği. Başarısız bir ileti yaklaşık bir saat içinde dört kez yeniden denenir ve hız sınırlayan bir hedef beklenir.
- **FinStats’in adresi**, yöneticiler için bir ayar, bir bildirimin bağlantısının neyden oluşturulduğudur. Boşsa iletiler bağlantı taşımaz.
- Sunucu sayfasında Jellyfin’in kendi zamanlanmış görevleri: her birinin ne yaptığı, kodunun adıyla değil sözle, ve ağır olanlardan biri olup olmadığı. Çalışan bir görev bir ilerleme çubuğu, bir yüzde ve ne kadar kaldığını gösterir; diğerleri bir sonraki ne zaman çalışacaklarını, en son ne zaman çalıştıklarını ve bunun iyi gidip gitmediğini gösterir. Jellyfin’in kendi kontrol panelinden gizlediği görevler de dahildir ve FinStats’teki hiçbir şey bir görevi başlatamaz ya da durduramaz.
- Bir görevin ne kadar kaldığı, yüzdesinin gerçekte ilerlediği hızdan ölçülür; tüm çalışmanın ortalamasına göre değil, bir anlam taşıyacak kadar geride kalan en son okumaya göre; böylece hızlanan ya da yavaşlayan bir görev şimdiki temposuyla tanımlanır.
- FinStats’in başka nedenlerle zaten okuduğu görev listeleri (her 5 dakikada bir kütüphane taraması denetimi, her 15 dakikada bir sunucu ayrıntıları) aynı ölçümü besler, böylece bir saattir çalışan bir görevin genellikle sayfa açıldığı anda bir tahmini olur. Bunun için Jellyfin’den fazladan hiçbir şey istenmez.
- Henüz hiçbir şey ölçülmemiş çalışan bir görev bir sayı yerine **Kalan süre** ve dönen bir üç nokta gösterir; yüzdesi yarım dakikadır ilerlemeyen bir görevin yanında “4 dk’dır %68” yazar. Ölçülmemiş bir tahminin yerine hiçbir şey konmaz.
- Sürüm notlarının yanında her sürümün listesi: tek tıkla herhangi bir sürüme gidilir, okunan sürüm işaretlenir ve dar bir ekranda notların üstünde serilerden oluşan bir satıra dönüşür.

### Changed
- Sürüm notları artık katlanmıyor. Her seri açık durur ve gezinmenin yolu sürüm listesidir; bir seferde tek bir grubun açık olması ve başka birine ulaşmak için tıklamak gerekmez.
- **Ayarlar → Giden bağlantılar**, Jellyfin ve hizmet bağlantılarının yanında her bildirim hedefini listeler: ne olduğu, ana makinesi, açık olup olmadığı ve en son ne zaman ileti aldığı.
- Toplama, geçmiş ya da ekrandaki herhangi bir rakamla ilgili hiçbir şey değişmiyor. FinStats hâlâ Jellyfin’den, Sonarr’dan, Radarr’dan ve Seerr’den yalnızca *okur*.

## [1.5.2] - 2026-09-22

### Changed
- Jellyfin’in oturum listesinin bir okuması ile onu isteyen saatin sıfırlanması tek bir çağrıdır, bu yüzden ikisi artık birbirinden ayrılamaz. 1.5.0 geliştirilirken bulunan, kontrolden çıkan isteklerle ilgili hatanın biçimi buydu.
- Bunu kanıtlamak gerçek bir sunucuya karşı yedi dakika yerine saniyenin bir kesrini alıyor, bu yüzden herhangi bir şey değiştiğinde her seferinde denetlenir. Aynı istekler, aynı geçmiş, ekranda aynı rakamlar.

## [1.5.1] - 2026-09-22

### Fixed
- Bir kod dönüştürme saniyede bir değil, bir kez kaydedilir. Bir oynatma kod dönüştürmeyle başlayıp doğrudan oynatmaya döndüğünde Jellyfin kod dönüştürme ayrıntılarını oturumda bırakır; her okuma FinStats’in az önce üzerine yazdığı bir değerle karşılaştırılıyordu, bu yüzden her okuma bir değişiklik gibi görünüyor ve Etkinlik sayfası altında “Doğrudan oynatma: …” yazan “Kod dönüştürme” satırlarıyla doluyordu.
- Zaten yazılmış satırlar başlangıçta temizlenir: aynı şeyi söyleyen bir dizi satırdan ilki tutulur. Kod dönüştürme gerektiren bir oynatma hâlâ kod dönüştürme sayılır.

## [1.5.0] - 2026-09-22

WebSocket ile oturum izleme

### Changed
- FinStats başladığı andan itibaren Jellyfin’e açık bir bağlantı tutar ve bir oynatmayı bu yolla duyar. 1.4.0 bunu gerçek sunucularda kendini kanıtlaması için varsayılan olarak kapalı bir ayar olarak getirmişti; artık her zaman açık. Aynı oynatmalar, duraklatmalar ve atlamalar, aynı kodla hesaplanıyor.
- Hiçbir şey oynatılmıyorken FinStats Jellyfin’e hiçbir şey sormaz. Kimsenin izlemediği bir akşam hiç istek harcamaz.
- Bir şey oynatılırken oturum listesi saniyede bir okunur. Bir duraklatma, sarma ya da parça değişikliği keskinliğini buradan alır ve istemcisi veda etmeden kaybolan bir oynatmayı bitiren de budur.
- Yüklü her şey art arda üç okumada duraklatılmışsa sorma durur ve haberleri bunun yerine bağlantı getirir; önceden duraklatılmış bir film her saniye aynı soruya, saatte yaklaşık 16 MB’a mal oluyordu. Devam ettirmeye ya da başka birinin bir şey başlatmasına yaklaşık bir saniye içinde yanıt verilir ve duraklatılmış süre hâlâ asla izleme süresi sayılmaz.
- Her giden okuma sıkıştırılmış yanıt ister ve onları açar (Jellyfin, Sonarr, Radarr ve Seerr istendiğinde hepsi sıkıştırır); bu, en büyük iki okumanın yarısından fazlasını kırpar: bir oynatma sırasındaki oturum listesi ve kütüphane okuması. Konum veritabanı hâlâ saklandığı haliyle indirilir.
- Ayarlar’daki Jellyfin kartı iki yarıdan hangisinin çalıştığını söyler ve canlı nokta, hangi yarıda olduğunuza değil, bağlantının veri taşıyıp taşımadığına uyar.
- Aynı anda ikisinden yalnızca biri çalışır: FinStats sorarken Jellyfin’e göndermeyi durdurmasını söyler ve son oynatma biter bitmez yeniden başlamasını ister. Bağlantı baştan sona açık kalır.
- Aboneliğe hiçbir şeyle yanıt vermeyen bir Jellyfin, canlı tutma iletilerine yanıt verdiği sürece güvenilir sayılır: FinStats bunu bir kez söyler, bu arada oturum listesini kendisi okur ve ilk gönderim durumu netleştirene kadar abone kalır. “Gönderemiyor” ile “sorduğumuzda yeniden başlıyordu” aynı görünür, bu yüzden sessizlik asla bir hüküm değildir.
- Geri dönüş yoluna dokunulmadı: kapanan, sessizleşen ya da bu protokolü konuşmadığı anlaşılan bir bağlantı hemen bir sonraki turda zamanlayıcıyla sormaya döner ve yeniden bağlanmayı sürdürür; sorma aralıkları **Biri izlerken şu aralıkla kontrol et** ve **Hiçbir şey oynatılmazken şu aralıkla kontrol et** ayarlarıdır.

### Added
- Toplayıcının ne yaptığı, `GET /api/status` üzerinde: dört durumdan hangisinde olduğu (`idle_socket`, `playing_poll`, `paused_socket`, `fallback`), bağlantının açık ve abone olup olmadığı, ne sıklıkla sorduğu, son dakikada kaç oturum okuması yapıldığı ve bunun en son ne zaman değiştiği. Bellekten yanıtlanır (istek yok, sorgu yok) ve hiçbir ad, içerik ya da sayı taşımaz.
- Bu alanlar her turda birbirlerine karşı denetlenir ve süren bir uyuşmazlık günlüğe yazılır.
- Süreçteki her oturum okumasının altında tek bir sınırlayıcı: saniyede en fazla 2 ve dakikada en fazla 70; bunun üstündeki her şey geciktirilir, sayılır ve dakikada en fazla bir kez günlüğe yazılır. Normal izleme bunun çok altında kalır.

### Fixed
- Canlı güncellemeler yoklamanın yerini gerçekten alıyor. Açıldığında 1.4.0 hâlâ her beş saniyede bir oturum listesini istiyor ve bağlantıyı her yirmi ila kırk saniyede yeniden açıyordu, çünkü bağlantıyı bir listenin ne kadar yakın zamanda geldiğine göre değerlendiriyordu; Jellyfin ise bir liste yalnızca bir şey *değiştiğinde* gönderir, arada hiçbir şey göndermez. Canlılık artık Jellyfin’in herhangi bir yanıt verip vermediğidir ve dakikada yaklaşık iki kez bir canlı tutma iletisiyle denetlenir.
- FinStats iki tur arasındayken gelen bir oturum listesi, bir buçuk saniye sonra bir başkasının geleceği varsayımıyla atılıyordu. Artık saklanır ve kullanılır.
- Ayarlar altındaki **Son kontrol** ve Giden bağlantılar kartı bağlantı sessizken artık eskimiyor: gösterdikleri, Jellyfin’in bir canlı tutma iletisine verdiği yanıttır.

### Removed
- Ayarlar → Toplama altındaki **Jellyfin oynatılanları bildirsin** ayarı ve ayarlar API’sindeki `live_socket`.
- Toplayıcı durumundan `socket_enabled`: artık “kapalı” diye bir durum yok.
- Toplayıcı durumundan `last_reconcile_at`; yerine `socket_live` geldi.

## [1.4.1] - 2026-09-22

### Changed
- Seerr yalnızca bir şey değiştiğinde tamamen okunur: FinStats her beş dakikada bir tek satır, en son değişen isteği ister ve onu zaten biliyorsa tur orada biter. Bu, elli satırlık bir sayfa için yaklaşık altmış kilobayta karşı yaklaşık bir kilobayttır. Hâlâ yolda olan isteklere her çeyrek saatte bir bakılır, her şey günde bir kez listelenir ve yeni bir istek hâlâ beş dakika içinde görünür.
- Sonarr ve Radarr kuyrukları, gün boyu her dakika yerine, bir sayfa onları gösterirken her beş saniyede bir, kuyrukta bir şey varken her dakika ve kuyruk boşken her beş dakikada bir okunur. Sayfayı açmak, bir hizmeti bağlamak ya da Seerr’deki yeni bir istek onları hemen yeniler.

## [1.4.0] - 2026-09-21

İsteğe bağlı WebSocket oturumları ve FinStats’in ulaşabildiği her yer

### Added
- Ayarlar → Toplama altında **Jellyfin oynatılanları bildirsin**, siz açana kadar kapalı: tek bir bağlantı oturum listesini taşır ve günde yaklaşık 17.000 istek neredeyse sıfıra iner. Oynatmalar, duraklatmalar, atlamalar ve gruplar, onları önceden hesaplayan kodun aynısıyla hesaplanır. Canlıyken FinStats *bir şey oynatılırken* dakikada bir sıradan istek yapmaya devam eder, hiçbir şey oynatılmıyorken hiç yapmaz. Kapanan, on beş saniye sessiz kalan ya da bu protokolü konuşmadığı anlaşılan bir bağlantı bir sonraki turda zamanlayıcıya döner ve yeniden bağlanmayı sürdürür.
- **Giden bağlantılar**, Ayarlar’da Jellyfin yöneticileri için bir kart: FinStats’in ulaşabildiği her hedef (Jellyfin’iniz, “IP adresim ne” hizmeti, DB-IP’nin veritabanı ve her Sonarr, Radarr ya da Seerr), ne için olduğu, açık olup olmadığı ve en son ne zaman yanıt verdiği. FinStats’in zaten bildiklerinden oluşturulur ve yalnızca adresleri gösterir: asla bir anahtar, asla bir temel yol.
- Ayarlar → Ev ağı altında **Şimdi sorgula**, herkese açık adresinizin değiştiği gün için.

### Changed
- “IP adresim ne” sorgusu her on beş dakikada bir yerine, FinStats’in bir adrese ilk ihtiyaç duyduğu anda bir kez çalışır.
- Ayarlar’daki Jellyfin kartı toplayıcının nasıl haber aldığını ve bağlantı açık ama veri taşımıyorken nedenini söyler.

## [1.3.0] - 2026-09-20

Sonarr, Radarr ve Seerr, tek sayfada

### Added
- Ayarlar altında **Bağlantılar**, Jellyfin yöneticileri için: **Sonarr**, **Radarr** ve **Seerr** (ya da Jellyseerr/Overseerr), her türden birkaç tane, her biri kaydedilmeden önce test edilir. İndirme istemcinizin ayrıca kurulması gerekmez, çünkü Sonarr ve Radarr zaten onunla konuşur ve FinStats onların bildiklerini okur.
- **İstekler**: kim ne istedi, gelmesi ne kadar sürdü ve isteyen onu hiç izledi mi. Bekleyen içerikler ve tipik bekleme süresi için kutucuklar, bunun aydan aya nasıl değiştiğini gösteren bir grafik ve haftalar önce gelip hiç oynatılmayanların ve en çok isteyenlerin listeleri.
- **Yakında**: Sonarr ve Radarr’dan yeni bölümler ve film çıkışları, gün gün bir ajanda olarak; *sizin* o diziyi izleyip izlemediğiniz ve herkesi görebilenler için başka kimin izlediği işaretli. Biri son dört ayda bir bölümünü oynattıysa bir dizi izleniyor sayılır. Panoda bir “Yakında” satırı, her profilde o kişinin izlediği diziler için bir tane ve bir içeriğin kendi sayfasında sıradaki.
- **İndirmeler**: her Sonarr ve Radarr’ın canlı kuyruğu (ne olduğu, ne kadar ilerlediği, ne kadar hızlı olduğu, içe aktarmada ne ters gittiği), yanında onu isteyen kişi. Bir sezon paketi kaç bölüm içerirse içersin tek satırdır ve ilgi isteyenler en üsttedir. Altında, bir hafta, bir ay ya da bir yıl içinde gelenler dizinleyiciye, kaliteye ve indirme istemcisine göre ve hangi indirmelerin başarısız olduğu. Pano aynı listeyi kısaltılmış olarak gösterir.
- **İndirilenleri görme**, yeni bir izin. Bu izin olmadan kişiler yine de *kendi* isteklerinin ne kadar ilerlediğini ve ne kadar kaldığını görür, başka hiçbir şeyi değil: sürüm adları yok, hızlar yok, başka indirmeler yok. Başka kişilerin istekleri *Herkesin etkinliğini görme* iznini gerektirir.

### Changed
- FinStats bu hizmetlere `GET` dışında hiçbir şey göndermez: içinde bir isteği onaylayabilecek, arama başlatabilecek ya da bir indirmeyi ekleyebilecek, duraklatabilecek veya kaldırabilecek hiçbir kod yoktur.
- Bu hizmetlerin API anahtarları FinStats’in kendi veritabanında saklanır; asla tarayıcıya geri gönderilmez, asla günlüğe yazılmaz ve asla yedeğin parçası olmaz. Hiçbir yönlendirme izlenmez, böylece bir anahtar girmediğiniz bir yere gidemez ve siz bir bağlantı için kapatmadığınız sürece sertifikalar doğrulanır.

## [1.2.2] - 2026-09-20

### Added
- Her film ve bölümde ses ve altyazı dilleri (“Ses: Japonca · İngilizce”).
- Bir dublajın nereye kadar gittiği: bir dizi ya da sezon, bölümlerinden kaçının her dile sahip olduğunu söyler (“İngilizce: 26 bölümden 13”) ve bölüm listesinde bir **Ses** sütunu vardır.
- Kütüphane sayfalarında **Ses dilleri** ve **Altyazı dilleri**: her birinde kaç dosyanın oynatılabileceği.
- FinStats dilleri almak için bu güncellemeden sonra kütüphaneyi bir kez yeniden okur. Asla “dublajlı” demez, çünkü Jellyfin bir içeriğin orijinal dilini vermez; dil etiketi olmayan bir parça “Bilinmiyor” olarak listelenir.

## [1.2.1] - 2026-09-20

### Changed
- README’deki resimler bu sürümde yeniden çekildi, aralarında **Güvenlik** sayfasının yeni bir resmi de var. Uygulamada hiçbir şey değişmiyor.

## [1.2.0] - 2026-09-20

İnsanların nereden izlediği, bir haritada

### Added
- Ağ ayrıntılarını ve herkesin etkinliğini görebilenler için **Güvenlik sayfası**: kullanıcılarınızın izlediği her yer için bir noktası olan bir dünya haritası; noktalar orada ne kadar şey olduğuna göre boyutlanır. Ev yeşil, dışarıdaki yerler mor, canlı akışlar nabız gibi atar ve dışarıdan gelen başarısız oturum açmalar kırmızıdır. Taşımak için sürükleyin, düğmelerle, tuşlarla ya da Ctrl + tekerlekle yakınlaştırın; üst üste binen noktalar siz yakınlaştırana kadar birleşir. Altında her yer kişileri, oynatmaları, izleme süresi ve oturum açmalarıyla, ülkeler de oynatmalara göre listelenir.
- **İmkânsız seyahat**: bir hesap hiçbir uçuşun bağlamadığı iki yerde ya da aynı anda birbirinden uzak iki yerde görüldüğünde bir uyarı; iki görülme, mesafe, aradaki süre ve bunun gerektireceği hızla. **Haritada göster** yolculuğu çizer.
- **Yeni ülke**: biri daha önce görülmediği bir ülkeden ilk kez oynattığında ya da oturum açtığında.
- Bir uyarıyı bir notla çözüldü sayın ya da o kişi için bir yer çiftini sessize alın; bir VPN ya da operatörü başkentte bulunan bir telefon için. Eski geçmişte, bir içe aktarmadan sonra ya da yeni bir veritabanında bulunan uyarılar listeyi doldurmak yerine çözülmüş olarak kaydedilir. Uyarılar ve onlar hakkında verdiğiniz kararlar yedeklerin parçasıdır.
- Konumlar yerel olarak okunan bir şehir veritabanından gelir: hiçbir adres hiçbir yere gönderilmez ve harita bir harita hizmetinden değil, FinStats ile birlikte gelen sınır çizgilerinden çizilir. DB-IP’nin ücretsiz veritabanını Güvenlik sayfasından ya da **Ayarlar → Güvenlik** bölümünden indirin, FinStats’in onu her ay yenilemesine izin verin (siz açana kadar kapalı) ya da kendi `.mmdb` dosyanızı (DB-IP, MaxMind GeoLite2-City) `data/geoip/` klasörüne bırakın. `FINSTATS_GEOIP_DB` başka bir yerdeki dosyayı belirtir.
- **Ayarlar → Güvenlik**: kullanılan veritabanı, aylık güncelleme ve iki görülmenin imkânsız seyahat sayılması için ne kadar hızlı (900 km/sa) ve ne kadar uzak (500 km) olması gerektiği. Şehir veritabanları çoğu zaman birkaç yüz kilometre şaşar; mesafe eşiği bunun içindir.

## [1.1.3] - 2026-09-20

### Added
- Ctrl+Space ayrıca kütüphanenizdeki her şeyin oyuncularını ve ekibini **Oyuncular ve ekip** altında arar; fotoğrafları, oyunculuk mu yönetmenlik mi yaptıkları ve içeriklerinizin kaçında yer aldıklarıyla. Birini seçmek sayfasını açar. İçerik araması kadar hoşgörülüdür (sözcükler herhangi bir sırada, aksanlar yok sayılır, bir parmak kayması affedilir) ve birkaçı eşit derecede eşleştiğinde, içeriklerinizin daha fazlasında yer alan önce gelir.

## [1.1.2] - 2026-09-20

### Added
- Panoda, Etkinlik grafiğinin üstünde **Son eklenenler**: en yeni 30 gelen, yana kaydırabileceğiniz bir poster sırası olarak; bilgisayarda oklarla. Her biri ne zaman geldiğini ve ne olduğunu söyler (bir film, bir albüm ya da “4. sezon · 3 bölüm”); tek bir yeni bölüm hangisi olduğunu, varsa sezonun kendi posteriyle söyler.
- **Shift + fare tekerleği** ve ok tuşları bir poster, Page Up ve Page Down bir ekran dolusu kaydırır, Home ve End iki uca gider ve parmakla kaydırma çalışır. Düz tekerlek hâlâ sayfayı kaydırır.
- Aynı dizinin aynı gün gelen yeni bölümleri tek bir girdiyi paylaşır ve bir kerede eklenen bütün bir dizi tek girdidir (“3 sezon · 60 bölüm”), böylece büyük bir içe aktarma diğer her şeyi listeden itmez.
- Sıra, oynatmalarla değil kütüphaneyle ilgili olduğu için panonun zaman aralığını ve kişisini yok sayar ve kütüphane boşken kendini gizler.

## [1.1.1] - 2026-09-20

### Added
- Sayfalar anında açılır: FinStats yüklendikten bir an sonra, büyük olasılıkla sırada açacaklarınızı (pano, profiliniz ve zaman çizelgeniz, Etkinlik, Kullanıcılar, Kütüphaneler ve her kütüphane, Oynatma, sunucu sayfaları, yıl özeti ve en etkin kişiler) sessizce getirir; böylece birini açmak onu hemen gösterir ve sayfa arka planda yine de yenilenir.
- Filmler, diziler ve oyuncular binlerce olduğu için yalnızca niyet belirdiğinde getirilir: işaretçiyi bir bağlantının üzerinde bekletmek, ona dokunmak ya da klavyeyle ulaşmak o tek sayfayı getirir.
- Açtığınız sayfanın yüklenmesi bitene kadar bekler, tarayıcı boştayken her seferinde tek bir sayfa üzerinde çalışır, arka plandaki bir sekmede durur, veri tasarrufu modunda ya da çok yavaş bir bağlantıda hiçbir şey yapmaz ve oturumu kapattığınızda unutulur.
- Durum çubuğunda sürümün yanında GitHub işaretli küçük bir **Depo** bağlantısı.

### Fixed
- **Zaman çizelgesi**, sayfa yeniden yüklendiğinde ya da bir yer iminden açıldığında, seçim adreste olmasına rağmen hangi kütüphanelerin kapatıldığını unutuyordu.

## [1.1.0] - 2026-09-20

İzlediğiniz her şeyin zaman çizelgesi

### Added
- Her profilde **Genel bakış** sekmesinin yanında **Zaman çizelgesi**: izlemeniz, bugünden FinStats’in bildiği ilk oynatmaya kadar tek bir patika olarak; bir akşamlık bölümler tek bir durağa katlanır. Her durak, neyin ne zaman izlendiğini gösteren bir posterdir (“2. sezon · 3-6. bölümler”, bir film ve birden fazla oturuşta izlenip izlenmediği ya da bir albüm ve kaç parça) ve içeriği açar.
- Patika pencerenin genişliğine uyar: geniş ekranda satır başına üç durak, daha dar olanında iki; her ikinci satır geriye doğru akar ve onu bir sonrakine bir dirsek bağlar; telefonda tek bir düz çizgiye iner. Her ayın ilk durağı ayın adını taşır.
- Daha eski duraklar siz kaydırdıkça, geçmişin başladığı yere kadar kendiliğinden yüklenir.
- Patikanın üstündeki **Kütüphaneler** anahtarları müzik gibi görmek istemediklerinizi dışarıda bırakır ve seçim adresin parçasıdır.
- Hâlâ oynatılan bir şeyde **Şimdi oynatılıyor** yazar.
- Kendi zaman çizelgenizi görürsünüz; başkalarınınki “herkesi görme” iznini gerektirir.

## [1.0.5] - 2026-09-20

### Added
- Sürüm düşürme koruması: veritabanı onu açmış en yeni FinStats sürümünü hatırlar ve daha eski bir sürüm, tam anlamadığı veriler üzerinde sessizce çalışmak yerine onunla başlamayı reddeder. İleti ne yapılacağını söyler. 1.0.4’e kadarki sürümler bu denetim yokken yayımlandı.

## [1.0.4] - 2026-09-20

### Fixed
- Bir oynatmanın ayrıntılarında, uzun bir **Cihaz kimliği** yanındaki kopyalama düğmesi kendi satırına düşmüştü.
- Oynatma sayfasında **Hangi istemciler kod dönüştürüyor** tablosunun son sütunu kesiliyordu (“Transc…”); iki sözcüklü başlıklar artık alt satıra geçer.

### Changed
- README’deki ekran görüntüleri yeniden güncel; hâlâ 0.5.0 sürümünü gösteriyorlardı.

## [1.0.3] - 2026-09-20

### Added
- Bir bölüm sayfası, dizinin posteri yerine o bölümün kendi resmini, yani Jellyfin’in bölüm listesinde gösterdiği resmi gösterir. Jellyfin’in resmi olmadığı bir bölüm posteri korur.

## [1.0.2] - 2026-09-20

### Fixed
- **Sürüm notları** bölümünde v0.9 ve v0.10 gruplarının sürümlerinin yanında manşet yoktu. Bir grubun manşeti artık her zaman ilk sürümünün ne hakkında olduğunu anlatan metnin ilk cümlesidir, böylece uzun bir giriş cümle ortasında kesilmez.

## [1.0.1] - 2026-09-20

### Fixed
- Yayımlanan imaj yeni bir makinede başlamıyordu: yarım dakika takıldıktan sonra “unable to open database file: /data/finstats.db”. Docker eksik bir `data` klasörünü root olarak oluşturur ve FinStats sıradan bir kullanıcı olarak çalışır. Konteyner artık klasörü kendine mal eder ve sonra o kullanıcıya geçer, böylece `docker run` ilk denemede çalışır. 1000 numaralı kullanıcıdan başka birine ait dosyalar için `PUID` ve `PGID` ayarlayın; `--user` eskisi gibi çalışır.
- Gerçekten yazılamayan bir veri klasörü 30 saniye sonra bir veritabanı hatası olarak değil, onu düzelten komutla birlikte hemen bildirilir.

## [1.0.0] - 2026-09-20

Herkes için hazır

### Added
- Hazır bir imaj, artık `ghcr.io/finstats/finstats` adresinde, 64 bit Intel/AMD ve ARM makineler için (Raspberry Pi 4 ya da 5 de çalışır): `:latest` en yeni sürümdür, bir sürüm etiketi olduğu yerde kalır ve `:edge` geliştirme sürümüdür.
- Her sürüm, bu sürüm notlarıyla birlikte projenin GitHub sürümler sayfasında yer alır.

### Changed
- Sürüm numaraları artık söylediklerini kastediyor: 1.x güncellemeleri verilerinizi, yedeklerinizi ya da ayarlarınızı bozmayacak.
- Docker Compose dosyası ve tüm talimatlar yayımlanan imajı kullanır. Kendi derlediğiniz bir `finstats:latest` için yalnızca imaj adını değiştirmeniz yeterli; `data` klasörü olduğu gibi taşınır.

## [0.10.2] - 2026-09-20

### Fixed
- Telefonda Ayarlar ve bir dizinin sayfası yana kayabiliyordu: yalnızca ekran okuyucular için olan bir etiket tablosundan taşıyordu, bölüm listeleri ve oyuncu satırı da aynısını yapıyordu. Artık kendi içlerinde kayarlar.
- Yıl özeti, bir bölümün arka plan sözcüğü sayfadan geniş olduğunda artık yana kaymıyor.
- **Şimdi oynatılıyor** altındaki uzun bir “… üzerinde izleniyor” satırı, harfin ortasında kesilmek yerine üç noktayla biter.
- Bir profildeki her dizinin önündeki poster, ekran okuyucular için ikinci, adsız bir bağlantıydı; artık onlardan gizlenir ve bağlantı yanındaki addır.

## [0.10.1] - 2026-09-20

### Fixed
- Bir yedeği indirmek tarayıcıda başarısız oluyordu (Firefox’ta “the source file could not be read”) ve sayfada nedenini söyleyen hiçbir şey yoktu: FinStats zaten sıkıştırılmış olan bir yedeği ikinci kez sıkıştırıyordu ve iki kez paketlenmiş aktarım sona varmadan kopuyordu. Yedekler artık oldukları gibi, boyutları önceden bildirilerek gönderilir, böylece tarayıcı gerçek ilerlemeyi gösterebilir.

## [0.10.0] - 2026-09-20

Yedekler ve yeni bir kuruluma taşınma

### Added
- Her hafta bir yedek, en yeni beşi saklanır: her oynatma duraklatma ve atlama zaman çizelgesiyle, izlendi işaretleri, izinler, ev adresleri, sunucu günlüğü ve ayarlarınız, tek bir küçük dosyada; binlerce oynatma için birkaç yüz KB. **Ayarlar → Yedekler** onları her biri için bir **İndir** düğmesiyle listeler, istendiğinde bir tane oluşturur ve ne sıklıkla ve kaç tane olacağını ayarlar. Veri dizininizin `backups` klasöründe dururlar. Bir yedek Jellyfin API anahtarı ve oturum içermez, ama IP adresleriyle birlikte eksiksiz bir izleme geçmişidir, bu yüzden onu gizli bir yerde saklayın.
- Geri yükleme üzerine yazmak yerine birleştirir: zaten orada olan oynatmalar atlanır, bu yüzden iki kez yapmak ya da bir süredir çalışan bir kuruluma yapmak güvenlidir. Yalnızca geçmiş için “Ayarları ve izinleri de geri yükle” seçeneğinin işaretini kaldırın. Yedekler sürümler arasında her iki yönde çalışır. Terminalden: `finstats backup` ve `finstats restore <file>`.

### Changed
- Toplama, biri izlerken her saniye, kimse izlemezken her 5 saniyede bir denetler; eskiden baştan sona her 5 saniyedeydi. Böylece duraklatmalar, atlamalar ve parça değişiklikleri saniyesine kadar kaydedilir. İkisi de **Ayarlar → Toplama** altındadır; değiştirilmiş eski bir aralığın yeniden ayarlanması gerekir.

### Fixed
- Bir oynatmanın ayrıntılarındaki IP adresi, adresin ortasından iki satıra bölünüyordu. Yerel/Uzak işareti yer olmadığında alta geçer.

## [0.9.1] - 2026-09-20

### Fixed
- Evdeki kişiler uzak olarak gösteriliyordu. Kendi ağınızdaki bir cihaz Jellyfin’e herkese açık adı üzerinden (ters vekil ya da alan adı gibi) ulaştığında evin herkese açık IP adresiyle gelir ve yalnızca özel adresler yerel sayılıyordu. FinStats artık herkese açık adresi öğrenir ve bu oynatmaları tüm geçmiş için yerel sayar; önceki adresleri de hatırlar.

### Added
- **Ayarlar → Ev ağı**: FinStats’in ev saydığı adresler; kendi adreslerinizi ekleme (önceki bir adres, ikinci bir ev, bir VPN çıkışı) ya da sorguyu kapatma. Sorgu her 15 dakikada bir düz bir “IP adresim ne” hizmetine sorar; Pi-hole gibi reklam engelleyen DNS’ler bunları çoğu zaman engellediği için birkaçını dener (Amazon, Cloudflare ve diğerleri) ve bunlardan biri hiç DNS gerektirmez. İstek sizin ya da sunucunuz hakkında hiçbir şey içermez ve anahtar kapalıyken FinStats Jellyfin’den başka hiçbir şeyle konuşmaz. `FINSTATS_PUBLIC_IP_URL` kendi hizmetinizi ayarlar.

## [0.9.0] - 2026-09-20

Her tablo, istediğiniz sırada

### Added
- Her tablo sıralanır: bir başlığa tıklayarak ona göre sıralayın, tersine çevirmek için yeniden, özgün sıra için üçüncü kez tıklayın. Hücrenin anlamı anlaşılır, böylece “3 gün 2 sa” süre, “1,4 GB” boyut olarak sıralanır ve boş hücreler sona gider. Kullanıcı listesini, izleyicileri, bölümleri, cihazları, eklentileri, zamanlanmış görevleri, her grafiğin tablo görünümünü, izin matrisini ve kendine ait ince bir başlık alan çubuk listelerini (codec’ler, çözünürlükler, istemciler ve diğerleri) kapsar.
- Etkinlik ve Sunucu günlüğü ekrandaki satırları değil tüm sayfalarını sıralar ve sıra adreste tutulur, böylece yeniden yüklemeden ya da paylaşılan bir bağlantıdan sonra da korunur.
- On ya da daha fazla satırlı tabloların üstünde küçük bir filtre alanı olur.

## [0.8.2] - 2026-09-20

### Changed
- Yıl özetinin yıl seçicisi, FinStats’teki diğer tüm aralık seçicileriyle aynı anahtardır (7 gün · 30 gün · 90 gün …).

## [0.8.1] - 2026-09-19

### Added
- Oyuncular ve yönetmenler için sayfalar; yıl özetindeki “En çok izlenen kişiler” bölümünden ya da herhangi bir film, dizi veya bölümdeki yeni **Oyuncular ve ekip** satırından açılır: sunucunuzda yer aldıkları her şey, ne kadarının kim tarafından izlendiği ve hâlâ bekleyenler. Seçtiğiniz zaman aralığına uyar, yalnızca kendi istatistiklerini görebilenler orada yalnızca kendi izlemelerini görür ve Esc sizi geri götürür.

## [0.8.0] - 2026-09-19

Yıl özetinin yeni bir görünümü ve söyleyecek daha çok şeyi var

### Added
- **En çok izlenen kişiler**: en çok zaman geçirdiğiniz oyuncular ve izlediklerinizin arkasındaki yönetmenler, bir portre sırası olarak. Oyuncular ve ekip kütüphaneyle birlikte okunur ve bu güncellemeden sonraki ilk kütüphane okumasından sonra görünür.
- **Yılınız, gün gün**: yılın her günü tek bir kare olarak, ne kadar çok izlediyseniz o kadar parlak; üstünde en uzun seriniz ve en yoğun gününüz. Ne içerdiğini görmek için bir günün üzerine gelin ya da ok tuşlarıyla takvimde gezinin.
- **Rakamlarla**: oynatmalar, izleme süresi, farklı içerikler, izlenen günler, en uzun seri ve ne kadarının **yeniden izleme** olduğu; ayrıca yılın bölümler, filmler ve müzik arasında nasıl bölündüğü.
- Tür bölümü kaç türe dokunduğunuzu ve en üsttekinin ne kadar pay aldığını söyler.

### Changed
- Yeni bir tasarım: yılınızı “yeniden oynatılmış” olarak açar; manşet, yılı dolduran posterlerin yanında, haftada bir çubuğu olan ve en yükseklerinin yandığı bir yıl dalga biçiminin üzerinde durur; yılı da hemen üstünden seçersiniz. Her bölümün anahtar sözcüğü mor olan bir manşeti, rakamları taşıyan bir cümlesi ve arkasında büyük ve soluk duran bölüm sözcüğü vardır. Filmlerin bittiği gibi jeneriklerle biter: başrolde, yöneten, gösterildiği yer, süre.
- Saatler, hafta günleri ve aylar, aralarında bir anahtarla tek bir **Etkinlik düzeni** bölümünü paylaşır ve manşet baktığınız şeye uyar (“Taç cumanın oldu”).

### Fixed
- Panodaki **Şimdi oynatılıyor**, kimse izlemezken boş bırakılıyordu, bu yüzden başlık altındaki filtrelere aitmiş gibi görünüyordu. “Şu anda hiçbir şey oynatılmıyor” kutusu geri geldi.

## [0.7.10] - 2026-09-19

### Added
- Kütüphane görselleri: her kütüphane kartı ve kütüphanenin kendi sayfası, Jellyfin’in o kütüphane için kullandığı resmi, posterler gibi FinStats’in resim vekili üzerinden gösterir. Resmi olmayan bir kütüphane simgesini korur.

## [0.7.9] - 2026-09-19

### Changed
- Türler kartı yeniden **liste** olarak açılır; radar anahtardaki ikinci seçenektir. Daha önce yapılmış bir seçim korunur.

## [0.7.8] - 2026-09-19

### Added
- Panoda ve profillerde Türler kartı için bir radar görünümü: tür başına bir kol; kartın köşesinde **Radar** ve **Liste** arasında geçilir ve seçim hatırlanır. Bir noktanın üzerine gelmek ya da ok tuşlarını kullanmak o türün izleme süresini ve payını gösterir.

## [0.7.7] - 2026-09-19

### Fixed
- Bir koleksiyona ait filmler “Artık kütüphanede değil” olarak gösteriliyordu. *group movies into collections* açıkken Jellyfin içindeki filmler yerine koleksiyonu verir, bu yüzden FinStats onları hiç görmüyordu; bir sunucuda bu, Filmler kütüphanesindeki 1.238 öğeden 556’sını gizliyordu. FinStats artık filmlerin kendilerini ister ve bu güncellemenin tetiklediği okuma onları geri getirip oynatmalarını yeniden bağlar.

## [0.7.6] - 2026-09-19

### Changed
- Profillerde **Diziler** kartı Türler’in hemen üstüne indi, böylece uzun bir dizi listesi izleme süresini, etkinlik grafiğini ve en çok izlenen içerikleri artık görüş alanının dışına itmiyor. Gün serileri en üstte kalır ve açtığınız bir dizi, zaman aralığını değiştirdiğinizde açık kalır.

## [0.7.5] - 2026-09-19

### Changed
- Arama tam ifadeyi gerektirmek yerine sözcük sözcük, herhangi bir sırayla ve aralarında başka şeyler olsa da eşleşir; böylece “alya hides” *Alya Sometimes Hides Her Feelings in Russian* içeriğini bulur. Büyük/küçük harf, aksanlar ve noktalama önemli değildir, baştaki “The” yok sayılır ve bir parmak kayması affedilir (“mentalsit” *The Mentalist* içeriğini bulur); üç ya da daha az harfli sözcükler hâlâ doğru yazılmalıdır. Daha iyi eşleşmeler önce sıralanır ve müzik sanatçısına göre de bulunur.
- Etkinlik ve Sunucu günlüğündeki arama kutuları tüm sütunlarda sözcük sözcük eşleşir: “alya opera”, o dizinin Opera’daki oynatmalarını bulur.

## [0.7.4] - 2026-09-19

### Changed
- Sürüm notları seriye göre gruplanır: her `0.x` serisi, sürümlerini barındıran ve serinin ne hakkında olduğunu, kaç sürüm içerdiğini ve ne zaman yayımlandığını gösteren katlanır bir bölümdür. Yalnızca en yenisi açık başlar ve kapalı bir bölüm hangi sürümü çalıştırdığınızı yine de gösterir.

## [0.7.3] - 2026-09-19

### Fixed
- Şimdi oynatılıyor saati birkaç saniyede bir duraksıyordu: 5 saniyelik yenileme onu da yeniden boyuyordu, bu yüzden sunucu bir kesir öndeyken sayı erken ilerliyor ve sonra bir saniye duruyordu. Artık onu yalnızca saniyede bir atan tik ilerletir, tam olarak bir saniye (999 ile 1001 ms arasında ölçüldü); yenileme ise bir duraklatmadan, bir atlamadan ya da gerçek bir kaymadan sonra yine düzeltir ve düzeltme tike denk gelir.

## [0.7.2] - 2026-09-19

### Added
- Canlı birlikte izleme: insanlar aynı şeyi birlikte izlerken her Şimdi oynatılıyor kartında “maria ile” yazar; eskiden bir grup ancak oynatmalar bittikten sonra tanınıyordu. Canlıyken kişiler aynı içerikteyse ve grup penceresi içinde başladılarsa ya da hemen hemen aynı konumdaysalar birlikte sayılır; bu, FinStats akış ortasında yeniden başlatıldıysa da çalışır.

## [0.7.1] - 2026-09-19

### Changed
- Şimdi oynatılıyor her saniyeyi sayar: saat tik tak eder ve çubuk, sunucuya her sorulduğunda sıçramak yerine aralarda kayar. Sunucuya hâlâ her 5 saniyede bir sorulur ve gerçeğin kaynağı o olarak kalır: ekran bir duraklatmada, bir atlamada ya da sunucu öndeyken yeniden eşitlenir, ama istemciler yalnızca on saniyede bir kadar bildirdiği için yalnızca birkaç saniye eskimiş bir konum için değil. Duraklatılmış akışlar yerinde durur.

## [0.7.0] - 2026-09-19

Kim birlikte izliyor

### Added
- Birlikte izleme: farklı kişiler aynı içeriği aynı anda başlattığında FinStats bunu birlikte izleme sayar. Panodaki **Birlikte izlendi** kartı grupları, paylaştıkları içerikleri ve birlikte geçen saatlerini gösterir; her profil o kişinin en çok kiminle izlediğini gösterir; paylaşılan oynatmalar Etkinlik’te küçük bir kişiler işareti taşır ve oynatma ayrıntıları kiminle izlendiğini söyler. İçe aktarılan oynatmalar dahil tüm geçmişinizde çalışır.
- Başlangıçların ne kadar yakın olması gerektiği için bir ayar, varsayılan olarak 60 saniye: gerçek grup oturumlarının yaklaşık üçte biri 6 ila 60 saniye arayla başlar. Kişilerin ayrıca birkaç dakika yan yana izlemeye devam etmesi gerekir, böylece aynı bölümü tesadüfen açan iki kişi bir grup sayılmaz.

### Changed
- Arama Ctrl/⌘ + K yerine **Ctrl + Space** ile açılır.

## [0.6.1] - 2026-09-19

### Added
- Yöneticiler, Yıl özeti sayfasının üstünden ya da kişinin profilindeki **Yıl özetini aç** ile başka bir kişinin yıl özetini açabilir. Özet o zaman “siz” yerine o kişiden adıyla söz eder.

### Changed
- Diğer herkes, hangi izinlere sahip olursa olsun hâlâ yalnızca kendi yıl özetini görür ve tüm sunucunun bir yıl özeti hâlâ yoktur.

## [0.6.0] - 2026-09-19

Profiliniz ve her dizide nerede olduğunuz

### Added
- Profillerde dizi ilerlemesi: dokunduğunuz her dizi, bölüm başına bir parçası olan bir çubuk olarak (izlendi, başlandı ya da henüz değil); sezon sezon açılabilir, bitirmeye ne kadar yakın olduğunuza göre sıralanır, bitirilen diziler ve her şey için ayrı görünümlerle.
- Kendi profilinizde bölüm, sezon ya da dizi başına **İzlendi olarak işaretle**; hiçbir şeyin kayıt yapmadığı dönemde izledikleriniz için. Jellyfin’in kendi oynatıldı işaretleri de kullanılır. İşaretler FinStats’te kalır ve Jellyfin’de asla hiçbir şeyi değiştirmez; kaydedilmiş bir oynatmanın işareti kaldırılamaz, kendi işaretleriniz kaldırılabilir.
- Profillerde gün serileri: her gün oynatma yapılan en uzun ardışık gün diziniz, şimdiki seriniz ve kaç gün bir şey izlediğiniz.
- Kenar çubuğunda **Profilim**.
- Etkinlik, ilerleme çubuğunun altında oynatmanın nerede durduğunu (örneğin `44:15 / 45:00`) ve “3 sa önce” altında tarih ve saati gösterir. Jellystat’ten içe aktarılan oynatmaların durma konumu yoktur, çünkü Jellystat bunu hiç kaydetmedi.

### Changed
- Eksik ya da henüz yayınlanmamış bölümler artık kütüphanenizin parçası sayılmıyor. Jellyfin bunları dosyası olmayan yer tutucular olarak listeler ve eskiden bölüm toplamlarına sayılıyorlardı.

## [0.5.1] - 2026-09-19

### Added
- Bir logo: üç grafik çubuğuna dilimlenmiş bir oynat düğmesi; kenar çubuğunda ve oturum açma ekranlarında keskin bir SVG olarak, isteyen tarayıcılar ve yer imi çubukları için çok boyutlu bir `favicon.ico` ve telefonlar ile tabletler için bir ana ekran simgesiyle.

## [0.5.0] - 2026-09-19

Kimin neyi göreceğine siz karar verin

### Added
- Ayarlar → Erişim altında herkes için birden ya da kişi kişi izinler: **Herkesin etkinliğini görme** (diğer kişilerin istatistikleri ve geçmişi, Kullanıcılar sayfası, her canlı akış), **Ağ ayrıntılarını görme** (IP adresleri, cihaz kimlikleri, yerel ya da uzak), **Sunucuyu görme** (Sunucu sayfası, sunucu günlüğü, başarısız oturum açmalar, dosya yolları) ve **FinStats’i yönetme** (ayarlar, görevler, içe aktarma, oynatmaları silme).
- Kişi başına oturum açma; FinStats’i her Jellyfin kullanıcısına açmadan birkaç kişiyi içeri almak için.
- Değişiklikler, erişimi geri almak dahil hemen uygulanır ve her şey düğmeleri gizleyerek değil, sunucu tarafından zorunlu kılınır.

### Changed
- Jellyfin yöneticileri hâlâ her şeye sahiptir ve izinleri değiştirebilen tek kişilerdir; FinStats’i yönetebilen biri ne kendisine ne de başkasına bir izin veremez.
- Yıl özeti, ne verilirse verilsin kişisel kalır.
- Siz bir şey verene kadar mevcut kurulumlar için hiçbir şey değişmez.

## [0.4.5] - 2026-09-19

### Changed
- FinStats **GNU General Public License v3.0** ile lisanslanmıştır. Daha önce hiçbir sürüm yayımlanmadı, bu yüzden farklı koşullar altında hiçbir şey yayımlanmadı. Birlikte gelen yazı tiplerinin lisansları onlarla birlikte gelir.

## [0.4.4] - 2026-09-19

### Added
- Esc geri adım atar: açılmış bir kütüphanede, kullanıcıda ya da içerikte sizi geldiğiniz yere, listeyi tam bıraktığınız haliyle döndürür. Açık olan her şey önce kapanır (oynatma ayrıntıları, arama, bir açılır liste, mobil menü) ve siz yazarken ya da bir grafiği klavyeyle kullanırken Esc’e dokunulmaz.

## [0.4.3] - 2026-09-19

### Changed
- FinStats her sayfanın en son ne gösterdiğini hatırlar: zaten baktığınız bir profile, kütüphaneye ya da zaman aralığına geri dönmek onu profil resmi dahil hemen çizer, taze rakamlar ise arkada yüklenir; sayfa yalnızca bir şey değiştiyse yeniden çizilir. Hatırlananlar tarayıcı sekmesinde yaşar ve oturumu kapattığınızda ya da herhangi bir şeyi değiştirdiğinizde atılır.
- Yükleme yer tutucuları yalnızca yükleme gerçekten yavaş olduğunda görünür. Önceden her sayfa onları en az saniyenin üçte biri kadar gösteriyordu.

### Fixed
- Jellyfin’de silinmiş ve hiç oynatması olmamış kütüphaneler artık listelenmiyor. Geçmişi olan silinmiş bir kütüphane hâlâ kaldırıldı olarak işaretlenmiş şekilde gösterilir.

## [0.4.2] - 2026-09-19

### Fixed
- Kullanıcılar sayfasında **Yönetici** etiketi kendi satırında duruyor ve o satırı diğerlerinden yüksek yapıyordu. Etiketler artık adın yanında durur.

## [0.4.1] - 2026-09-19

### Fixed
- Aralık ve ocak dışında, gösterilecek bir yıl özeti bandı yokken pano başlığının altında artık “null” sözcüğü görünmüyor.

## [0.4.0] - 2026-09-19

Uygulamadan çıkmadan neyin değiştiğini görün

### Added
- Her sürümü gösteren ve çalıştırdığınız sürümü işaretleyen **Sürüm notları** sekmesi.
- Durum çubuğundaki sürüm ona bağlanır ve sekme bir güncellemeden sonra siz bakana kadar bir nokta gösterir.

### Changed
- Geçmişi yeniden adlandırılmış öğelere yeniden bağlamak çok daha hızlı: 666 sahipsiz içerikle 18 saniyeden saniyenin üçte birine; böylece geçmiş büyüdükçe başlangıç anlık kalır.

## [0.3.3] - 2026-09-19

### Changed
- Kütüphane, Jellyfin’in **Scan Media Library** görevi bittikten hemen sonra yeniden okunur; asla ayrı bir zamanlayıcıyla ve asla bir tarama sürerken değil. FinStats Jellyfin’e karşı her zaman salt okunur oldu ve orada asla tarama başlatmaz.
- Haftalık bir güvenlik ağı okuması, tarama görevinin hiç çalışmayabileceği, gerçek zamanlı izlemeye dayanan sunucuları kapsar.
- Eski aralık artık yalnızca bir geri dönüş yoludur; taramayı izleme kapalıyken ya da sunucu bir tarama görevi bildirmediğinde kullanılır.

### Added
- Ayarlar → Toplama altında **Jellyfin’in kütüphane taramasını izle** anahtarı, varsayılan olarak açık.

## [0.3.2] - 2026-09-19

### Fixed
- Geçmiş yeniden adlandırmalardan sağ çıkar. Jellyfin yeniden adlandırılan bir dosyaya yeni bir kimlik verir; bu, önceki oynatmalarını ham klasör adı altında (`Title (2010) [tmdbid-…] [imdbid-…]`) postersiz bırakıyordu. Bu oynatmalar geçerli kütüphane girdisine yeniden bağlanır: eski ad bir TMDB/IMDb/TVDB kimliği taşıyorsa ona göre, değilse temizlenmiş ad ve yıla göre; bölümler de dizi ile sezon ve bölüm numarasına göre. Bir eşleşmenin tek anlamlı olması gerekir, bu yüzden gerçekten silinmiş içerikler olduğu gibi kalır.
- Jellystat’ten içe aktarılan Canlı TV kanalları film sayılıyordu, çünkü Jellystat öğe türü kaydetmez. Artık Canlı TV olarak tanınırlar ve yeni Canlı TV oynatmaları Jellyfin’in kendi türüyle kaydedilir.

### Changed
- Canlı TV yıl özetinin tamamen dışında bırakılır: üst listeler, toplamlar, izleme tarzı, rekorlar ve sıralama.

### Added
- Geçmişi istendiğinde yeniden bağlamak için `finstats relink`. Başlangıçta ve her kütüphane okumasından sonra da çalışır.

## [0.3.1] - 2026-09-19

### Changed
- Yıl özeti tamamen kişiseldir: yöneticiler dahil herkes yalnızca kendisininkini görür; “Herkes” görünümü, kullanıcı değiştirici ve sunucu bölümü kaldırıldı.
- Yıl özeti “hazır” olan yılda açılır: aralık boyunca içinde bulunulan yıl, diğer aylarda bir önceki yıl. Diğer yıllar ve “Son 12 ay” bir tık uzaklıkta kalır.

## [0.3.0] - 2026-09-19

Yılınızın özeti

### Added
- **Yıl özeti** sekmesi, bir yılın kaydırılarak okunan hikâyesi: izlenen saatler, en çok izlenen diziler, filmler, parçalar ve türler ve saat ve hafta günü grafikleriyle bir izleme tarzı (gece kuşu, erkenci kuş, hafta sonu savaşçısı, maraton izleyicisi, film kurdu, müzik tutkunu, alışkanlıklarına bağlı).
- Ay ay yıl, her ayı tanımlayan içerikle.
- Rekorlar: en yoğun gün, en büyük maraton, en uzun günlük seri, en uzun tek oynatma, en çok yeniden izlenen içerik, yılın ilk oynatması ve izlenen en eski içerik.
- Keşif: başlanan yeni diziler, bitirilen filmler ve bölümler ve tam olarak tek bölümü izlenen diziler.
- Aralık ve ocakta, bir yıl özeti hazır olduğunda panoda tek satırlık bir bant.

## [0.2.0] - 2026-09-19

Jellystat’in kaydettiğinden fazlası

### Added
- Her oynatma için bir zaman çizelgesi: başlama, duraklatma, devam etme, atlama, ses ve altyazı değişiklikleri, doğrudan oynatmanın kod dönüştürmeye dönüşmesi ve durma. Ayrıca duraklatma ve atlama sayıları, oynatmanın nereden devam ettiği ve izleyicinin yerel ağda olup olmadığı.
- **Sunucu** sekmesi: Jellyfin sürümü, bekleyen güncellemeler ve yeniden başlatmalar, kütüphane başına disk kullanımı (Jellyfin 10.11+), eklentiler, zamanlanmış görev sonuçları ve kayıtlı her cihaz.
- **Oynatma** içgörüleri: zaman içinde eş zamanlı akışlar, hangi istemcilerin kod dönüştürmeyi zorunlu kıldığı, insanların durmadan önce ne kadar ilerlediği, yerel ve uzak oynatmalar ve izleme davranışı.
- Kütüphanenizin neyden oluştuğu: çözünürlükler, codec’ler, dinamik aralık, kapsayıcılar, on yıl başına içerikler, ay başına eklemeler, en büyük içerikler ve hem FinStats’in geçmişine hem de Jellyfin’in kendi oynatıldı işaretlerine göre denetlenen, hiç kimsenin izlemediği içerikler.
- Pano: zirve eş zamanlı akış, tahmini aktarılan veri, uzak oynatmaların payı, izleme süresine göre türler ve başarısız oturum açmalar.
- Öğe sayfaları: IMDb ve TMDB bağlantıları, stüdyolar, bit derinliği ve kare hızı ve içeriği Jellyfin’de oynatıldı olarak işaretlemiş olanlar.
- Kullanıcı sayfaları: türler ve Jellyfin’in kayıtlı tuttuğu filmler, bölümler ve favoriler.

### Fixed
- Şimdi oynatılıyor kartındaki zaman etiketi artık kartın dışına taşmıyor.
- Oynatma sayfasındaki uzun etiketler fazla erken kesiliyordu.
- Kütüphaneden ayrılmış bir dizi hâlâ sezon ve bölüm listesini gösterir.

## [0.1.0] - 2026-09-19

İlk sürüm

### Added
- Jellyfin’in oturumlarından canlı toplama: kullanıcı, içerik, istemci, cihaz, IP adresi, oynatma yöntemi, kod dönüştürme nedenleri ve donanım hızlandırma, video, ses ve altyazı ayrıntıları ve duraklatılan süreye karşı izlenen süre.
- Jellyfin hesabınızla oturum açma. İki adımlı bir kurulum FinStats’in kendi API anahtarını oluşturur ve parolalar asla saklanmaz. Yönetici olmayan kullanıcılar isteğe bağlı olarak, yalnızca kendi istatistiklerini görmek için oturum açabilir.
- Pano, Etkinlik, Kullanıcılar, Kütüphaneler, Oynatma ve Sunucu günlüğü sayfaları, bir komut paleti (Ctrl/⌘ K) ve canlı bir durum çubuğu.
- Jellystat içe aktarması: büyük yedekler diske akıtılır ve saniyeler içinde içe aktarılır; aynı dosyayı iki kez içe aktarmak güvenlidir.
- Gömülü bir web arayüzü ve tek bir SQLite dosyasıyla tek bir küçük ikili dosya, ayrıca bir Docker imajı.
