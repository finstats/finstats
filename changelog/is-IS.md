# Útgáfulýsingar

Allt sem hefur breyst í FinStats, nýjast fyrst. Þessi skrá er felld inn í forritið og er það sem
flipinn **Útgáfulýsingar** í forritinu sýnir.

Snið: `## [version] - date`, eða `## [version] - first day to release day` fyrir útgáfu sem var unnin
á nokkrum dögum, síðan `### Added`, `### Changed`, `### Security`, `### Performance`, `### Stability`, `### Fixed`
eða `### Removed` (Bætt við, Breytt, Öryggi, Afköst, Stöðugleiki, Lagfært, Fjarlægt) með einum punkti fyrir hverja
breytingu. Öryggi er eitthvað sem einhver hefði getað misnotað, Afköst eru eitthvað sem varð fljótlegra eða léttara, Stöðugleiki eitthvað sem getur ekki lengur
hrunið, stöðvast, lekið eða glatað gögnum, og Lagfært eitthvað sem gaf rangt svar. Útgáfa `x.y.0` ber
stutta titillínu undir fyrirsögn sinni: þann titil notar forritið sem fyrirsögn allrar raðarinnar.

## [2.3.1] - 2026-10-26 to 2026-10-30

### Security
- **Uppsetningin biður um einnota kóða**: þar til FinStats hefur verið sett upp skrifar það kóða í atvikaskrá sína við ræsingu (`docker logs finstats`), og uppsetningarleiðsögnin biður um hann áður en hún prófar vistfang eða setur nokkuð upp. Annars gat hver sá sem komst fyrstur að gáttinni beint FinStats að eigin Jellyfin og orðið stjórnandi þess, eða notað prófunina til að skima um netið.
- **Aðgangur að söfnum fylgir Jellyfin**: hver og einn sér aðeins söfnin sem Jellyfin opnar fyrir honum, og í þeim aðeins titlana sem Jellyfin sýnir honum innan aldursmarka hans, merkja og reglna um óflokkað efni, á safnasíðum, í leit, leikaraliði, myndum, Nýlega bætt við, áhorfslistanum, Á næstunni, dagatalsstraumnum og tilkynningum um nýja titla. Allir innskráðir gátu skoðað öll söfn.
- Tengill gat sent einhvern frá móttökusíðunni yfir á annan vef strax eftir innskráningu. Nú er aðeins farið á síður FinStats sjálfs.
- Atvikaskrá þjóns og misheppnaðar innskráningar í innsýninni gáfu hverjum sem hafði *Sjá þjóninn* innskráningarvistföng og virkni allra. Línur annarra krefjast nú *Sjá virkni allra* og vistfang *Sjá netupplýsingar*, eins og alls staðar annars staðar.
- Persónulegur viðtakandi tilkynninga gat náð inn á staðarnetið gegnum nafn sem svaraði öðruvísi í annað sinn sem flett var upp í því. Nú er tengst við vistfangið sem var athugað, fleiri svið sem ná aldrei út á netið teljast staðbundin, opinberu vistfangi heimilisins sjálfs er líka hafnað, og nafni sem ekki er hægt að fletta upp er hafnað með sömu orðum og staðbundnu.
- Á bak við milliþjón sem bætir við eigin `X-Forwarded-For`-línu, eða frá IPv6-neti, gat hver innskráningartilraun komið frá nýju vistfangi og hámarkið gilti aldrei. IPv6 /48-net fær nú 50 tilraunir á fimm mínútum, fullt hámark hafnar nýju vistfangi í stað þess að gleyma gömlu, allur þjónninn stöðvast við 60 mistök á mínútu, og misheppnuð innskráning er skráð í endurskoðunarskrá einu sinni fyrir hvert vistfang og tímabil.
- `FINSTATS_TRUST_PROXY` tilgreinir milliþjónana (vistföng eða svið) sem treyst er fyrir `X-Forwarded-For`, `X-Forwarded-Host` og `X-Forwarded-Proto`, og `1` eða `true` treystir nú aðeins mótaðila á loopback eða einkaneti. Þegar kveikt var á þessu gat hvaða mótaðili sem var valið sér vistfang; slökkt á bak við milliþjón læstu tíu rangar ágiskanir alla úti.
- Þegar notendurnir hafa verið lesnir er innskráningu fyrir nafn sem er óþekkt, óvirkt, fjarlægt eða ekki hleypt inn í FinStats svarað sem röngu lykilorði án þess að spyrja Jellyfin, og ekki fyrr en Jellyfin hefði svarað. Slík nöfn gátu læst Jellyfin-aðgöngum, komist fram hjá reglu Jellyfin um fjartengingar eða falið sig á bak við vistfang FinStats.
- Sá sem Jellyfin heldur á heimanetinu (slökkt á *Allow remote connections*) skráir sig aðeins inn í FinStats þaðan, þar sem Jellyfin mat stillinguna út frá vistfangi FinStats sjálfs, og lestur notenda sem segir ekki hvort einhver sé óvirkur heldur þeim úti.
- Lotukaka er merkt Secure hvenær sem síðan sem skráð er inn á er á https, líka á bak við milliþjón sem segir það ekki.
- Innskráning sem tekst fyrirgefur ekki lengur ágiskanirnar á undan henni. Á https er lotukakan `__Host-`-kaka sem vefur á nálægu nafni getur ekki skipt út og er ekki tekin gild undir neinu öðru nafni, og útskráning lýkur öllum lotum sem vafrinn sendi.
- Aðeins dagatalslykill opnar dagatalsstrauminn úr vistfangi hans; allir aðrir lyklar fara í hausinn.
- API-lykill getur ekki lengur veitt heimildir, breytt því hver kemst inn eða hvað er opinbert, endurheimt afrit eða bætt við, breytt eða prófað þjónustu eða viðtakanda tilkynninga: hvert þeirra lifir lykilinn og krefst því innskráðs vafra.
- Hvert svar ber `Cross-Origin-Opener-Policy: same-origin` og `Permissions-Policy`, API-svör bera `Cross-Origin-Resource-Policy: same-origin` og eru `no-store` nema þau segi annað, og CSP leyfir ekki lengur `blob:`-myndir. Síða sem opnaði FinStats gat síðar breytt flipa þess í eigin innskráningarsíðu.
- Þar sem FinStats veit hvaða samskiptareglu það er náð með (`X-Forwarded-Proto` trausts milliþjóns, eða opinbera vistfangið), er skrifum frá annarri reglu hafnað sem cross-site.
- `/auth/me`, síða titils og Heilbrigði safns tengja inn í Jellyfin á opinberu vistfangi þess; vistfangið sem FinStats sjálft les Jellyfin á fer aðeins til stjórnenda.
- Mynd er aðeins sótt frá Jellyfin fyrir eitthvað sem FinStats hefur, veggspjöld og spjöld opinbers prófíls meðtalin, svo ekki er hægt að biðja um myndir annarra safna eftir auðkenni.
- Lyklar lokaðs rekjara eru líka afmáðir úr villum niðurhalsbiðraðarinnar, eftir lögun sinni jafnt sem nafni, og sá sem hefur aðeins *Sjá hvað er verið að sækja* fær að vita hvaða þjónusta brást og hvernig, aldrei innra vistfang hennar eða hvert hún vísar áfram.
- Síðan Þjónn nefnir síðasta notanda tækis aðeins með *Sjá virkni allra*, og Ótengt efni krefst þess auk *Stjórna FinStats*. Án hennar telur hlutinn Verkefni aðeins eigin spilanir og streymi, fjarlægt safn er sýnt eftir eigin spilunum, og ekki er hægt að hætta við Tautulli-töflu sem bíður.
- Stillingarnar sýna þjónustuna fyrir opinbert vistfang aðeins eftir hýsilnafni hennar og vistfang Jellyfin og gagnamöppuna aðeins stjórnendum, og titli sem safnið missti er aðeins lýst út frá spilunum sem lesandinn má sjá.
- Tautulli-afrit þar sem töflurnar voru ekki venjulegar töflur gat látið öll önnur skrif bíða þar til FinStats var endurræst. Slíkri skrá er hafnað, og SQLite les afritið með varúð.
- Upphleðslu yfir 4 GB er hafnað, og þjappað Tautulli-afrit er ekki afþjappað umfram þá stærð sem það segist hafa; hvort tveggja gat fyllt diskinn sem gagnagrunnur FinStats sjálfs er á.
- Svar frá Jellyfin, Sonarr, Radarr, Seerr, uppflettingu opinbers vistfangs eða GeoIP-niðurhalinu er ekki lesið umfram hámarkið sem það má vera, talið eftir afþjöppun.
- Hvaða Jellyfin-notandi sem var gat stöðvað skráningu FinStats á spilunum allra með því að tilkynna þúsundir stuttra spilana á einum titli. Nú er fundið út hverjir horfðu saman í einni umferð, og enginn lás er haldinn á meðan.
- Spilanir sem skráðar eru hver á eftir annarri, eða einn titill spilaður til skiptis af einu tæki eftir annað, tengja ekki lengur ókunnuga saman í einn hóp, og sá sem má aðeins sjá sjálfan sig fær aðeins að vita af þeim sem eigin spilun hans var samhliða, og aðeins þann tíma sem hún var það.
- Tengill gat enn sent einhvern frá móttökusíðunni yfir á annan vef eftir innskráningu, gegnum slóð eins og `/.//`.
- Sá sem má stjórna FinStats getur ekki lengur breytt því hve mörg afrit eru geymd eða hve oft þau eru skrifuð, sameiningarglugganum (nú í mesta lagi klukkustund) eða hvenær notendur, atvikaskráin og afrit eru lesin, og afrit slíks aðila ýta aldrei afritum stjórnanda út; heimilisvistföng og ferðareglurnar krefjast þess sama og viðvaranirnar, og innflutningur sögu krefst líka *Sjá virkni allra*.
- Vistaður lykill, tóki eða lykilorð fer aðeins til þjónsins sem það var vistað fyrir, og viðtakanda tilkynninga er aðeins hægt að búa til eða breyta í innskráðri lotu, aldrei með API-lykli. Stjórnandi má slökkva á eða fjarlægja persónulegan viðtakanda annars, en ekki færa hann, slökkva á vottorðaathugun hans eða lesa ntfy-efni hans eða Pushover-notandalykil.
- Discord- og Slack-vefkrókar verða að vera `https`, þar sem vistfang þeirra er tóki þeirra.
- Flóð misheppnaðra innskráninga getur ekki lengur falið raunverulega innskráningu eða kaffært viðvaranirnar: nöfn sem eru ekki aðgangur gefa eina tilkynningu á korters fresti sem nefnir engan, viðvaranir fara fyrst út, og Slack og Discord sýna efni skilaboða sem texta og pinga engan.
- Sætið í Uppgjöri, Afkastageta, tilkynning og opinber prófíll segja ekki meira en síðurnar: hvorki sæti né hlutdeild í tíma allra til þess sem má aðeins sjá sjálfan sig, annatímar aðeins sem dagar með þremur streymum eða fleiri án *Sjá virkni allra*, ekkert misheppnað verk eða tenging til þess sem má ekki stjórna FinStats, og opinber prófíll breytist einu sinni á dag.
- Tautulli-afriti með reiknuðum dálki er hafnað, og mynd er aðeins sótt fyrir innflutta spilun þegar safnið hefur titil hennar.
- Endurheimt opnar FinStats aldrei meira en það er: stillingar koma aðeins aftur þegar beðið er um það, innskráning, heimildir og opinberir prófílar verða það sem bæði þessi uppsetning og afritið leyfa, og opinberu vistföngin haldast þessarar uppsetningar. `finstats restore <file>` færir aðeins söguna aftur, eins og síðan, og stillingarnar líka með `--with-settings`.
- Endurheimt sögunnar einnar færir ekki lengur aftur heimilisvistföng, þögguð viðvörunarboð, endurskoðunarfærslur sem þykjast vera þessarar uppsetningar eða staðsetta titla sem safnið vantar, og engin endurheimt færir aftur spilanir, línur í atvikaskrá eða tæki dagsett fyrir 1990 eða eftir morgundaginn. Endurheimt spilun telst innflutt hvaða uppruna sem hún segist hafa, svo endurræsing heldur henni aldrei áfram.
- Gagnamappan tilheyrir aðeins eigin notanda FinStats (0700, skrár hennar 0600, `umask 077` í myndinni). FinStats neitar að keyra sem root yfir gagnamöppu sem annar notandi á og nefnir hvaða `docker exec -u` skuli nota, og entrypoint breytir eiganda aðeins möppunnar og eigin færslna FinStats.
- Nafn viðtakanda má ekki innihalda stýristafi, og nafn tækis, manneskju eða titils getur ekki lengur falsað línu í atvikaskránni.
- Leiðarbreyta tengils getur ekki lengur náð annarri API-slóð en eigin slóð síðunnar, og einingasmiður FinUI setur vistfang aðeins þegar það er http(s) eða afstætt, aldrei `srcdoc`.
- Leit, opinberir prófílar og verk ræst handvirkt geta ekki lengur haldið FinStats uppteknu: leit stöðvast við 200 stafi, prófíll er geymdur fyrir hvert tungumál, spjöld bíða í stuttri biðröð, og verk, uppfletting vistfangs, niðurhal staðsetningargagnagrunnsins og prufuskilaboð ræsast aðeins með hæfilegu millibili hvernig sem beðið er um þau, og prufuskilaboð eru talin fyrir hvern sendanda. Að sækja staðsetningargagnagrunninn krefst líka þess sem lestur síðunnar Öryggi krefst.
- Meginmál beiðni verður að halda áfram að berast þegar fyrstu 15 sekúndurnar eru liðnar, svar sem enginn les er gefið upp eftir 30 sekúndur, og eitt opinbert vistfang heldur í mesta lagi 64 tengingum, en traustir milliþjónar, jafnvel með opinber vistföng, og vistföng á þessari vél eða í einkaneti eru undanþegin. Nokkur hundruð beiðnir sem sendu aldrei meginmál sitt gátu komið í veg fyrir að FinStats svaraði nokkrum.
- Póstþjónn er ekki lesinn lengra en nokkurt svar getur verið (1 MB fyrir heilt bréf), og prófunarskilaboð stöðvast eftir jafnlangan tíma og skilaboð í biðröð mega taka. Þjónn sem endaði aldrei línu gat látið FinStats halda sífellt meira minni þar til það var stöðvað.
- Lestur safns er metinn út frá titlunum sem lestrar hafa vistað, ekki þeim sem innflutningur kom með, og Jellystat-titill í safni sem þegar hefur verið lesið kemur inn sem horfinn. Innflutningur uppdiktaðra titla gat látið hvern lestur safns stöðva FinStats, við hverja ræsingu.
- Mörkin um 60 misheppnaðar innskráningar á mínútu fyrir allan þjóninn telja aðeins tilraunir sem sendar eru til Jellyfin, líka þær sem bíða enn svars. Nöfn sem FinStats hafnaði sjálft leyfðu hverjum sem var með fáein vistföng að stöðva innskráningu allra, og margar ágiskanir sendar í einu náðu allar til Jellyfin.
- Sá sem Jellyfin heldur við heimanetið er haldið þar við hverja beiðni, seta og lyklar meðtalin, ekki aðeins við innskráningu; heima er einkanet eða opinbert vistfang heimilisins eins og uppflettingin fann það, aldrei vistfang sem slegið var inn sem heima, og á bak við milliþjón sem `FINSTATS_TRUST_PROXY` nefnir ekki er viðkomandi haldið úti. Vafrakaka eða lykill sem fór út úr húsi opnaði allt hvaðan sem var, og stjórnandi gat opnað húsið fyrir vistfangi að eigin vali.
- API-lykill getur ekki lengur slökkt á því sem myndi taka eftir misnotkun hans, né afritunum sem myndu afturkalla það sem hann gerði: hversu mörg afrit eru geymd og hve oft, sameiningarglugganum, heimilisföngum og ferðareglum, áætlunum notenda, virkniskrár og afrita, þöggun eða lausn allra viðvarana, afriti í ruslið eða að fjarlægja tilkynningastað. Hvert og eitt entist lengur en lykillinn þegar hann var afturkallaður.
- Veggspjöld frá Sonarr og Radarr og næsti þáttur á óskalista fylgja líka aðgangi að söfnum: þáttaröð sem Jellyfin felur fyrir einhverjum hefur hvorki veggspjald né dagsetningu fyrir viðkomandi. Að telja auðkenni þjónustanna sýndi hvaða faldar þáttaraðir voru á leiðinni, og listi nefndi næsta þátt faldrar þáttaraðar.
- Kaflinn Saman í ársyfirlitinu nefnir aðeins þá sem spiluðu samhliða þér, eins og síðan Saman gerir. Hópur sem þú lentir í gegnum einhvern annan nefndi fólk sem þú horfðir aldrei með.
- Innflutningur úr Jellystat eða Streamystats les alla skrána áður en hann tekur skriflás gagnagrunnsins og skrifar svo aðeins línurnar sem hann heldur. Lesin undir lásnum gat skrá með fáeinum raunverulegum línum og gígabætum af hverju sem er öðru látið öll önnur skrif bíða, spilanir og innskráningar meðtaldar.
- Endurheimt heldur einni heimild á mann, fyrir ekki fleiri en heimili margfalt, og leitar að tvítekningu færslu í atvikaskrá án þess að lesa hverja færslu sem hún var að skrifa. Smíðað afrit gat haldið minni og skriflási gagnagrunnsins lengur og lengur meðan það var endurheimt.
- Endurheimt með stillingum kveikir á uppflettingu opinbers vistfangs og niðurhali staðsetningargagnagrunns aðeins þar sem þessi uppsetning hefur kveikt á þeim líka, og geymir aldrei færri afrit né skrifar þau sjaldnar en þessi uppsetning. Afrit sem geymdi eitt gat látið næstu keyrslu fjarlægja öll hin.
- Skráarstærð sem enginn diskur gæti rúmað er lesin sem engin stærð, og stærðir eru lagðar saman svo engin summa geti flætt yfir. Ein innflutt stærð lét safnsíðurnar svara með villu fyrir fullt og allt.
- Þegar FinStats veit að það er sótt um https (það er beðið um það á opinberu https-vistfangi þess, eða traustur milliþjónn segir það) er seta aðeins tekin úr `__Host-`-vafrakökunni, og hver ný seta þar er https-seta. Innskráning send án `Origin` í gegnum milliþjón sem sagði ekkert bjó til venjulega setu sem nágrannavefur gat komið fyrir í vafra einhvers.
- Myndin er smíðuð með hverja GitHub-aðgerð festa við commit og báðar grunnmyndir við samantekt, sem aðeins Dependabot færir, og útgáfa er smíðuð án sameiginlega smíðaskyndiminnisins. Fært merki á aðgerð einhvers annars hefði getað birt hvaða mynd sem er sem FinStats.
- Stílblað FinUI, teiknað í eigin útliti gestsins, mega aðeins síður FinStats sjálfs hlaða. Síða á annarri gátt sama hýsils gat lesið úr því hvort gesturinn var innskráður og hvaða forstillingu hann valdi.
- Hverri hafnaðri innskráningu er svarað á einu augnabliki talið frá því hún var send, hvort sem FinStats eða Jellyfin hafnaði henni, og nafn þekkist óháð há- eða lágstöfum, líka utan A til Ö. Hversu fljótt höfnun barst sagði hvaða nöfn voru einhver, og „Åse“ sem sló inn `åse` var hafnað án þess að spyrja Jellyfin.
- Villur við niðurhal fela vistföng biðlara fyrir öðrum en stjórnendum. Þeir sem gátu skoðað biðröðina gátu áður séð innri hýsla, gáttir og slóðir.
- Mörkum fyrir lykla og áfangastaði tilkynninga er framfylgt í einni ódeilanlegri aðgerð. Samtímis beiðnir gátu áður farið yfir þau.

### Performance
- Opinber prófíll er reiknaður út í mesta lagi einu sinni á mínútu, ekki aftur fyrir hverja mynd og hvert spjald á honum.
- Saga einstaklings er skimuð eftir öryggisviðvörunum einu sinni í einu, hversu margar spilanir sem hefjast.
- Innskráning eða útskráning, afturköllun lykils eða breyting á heimildum eins einstaklings biður aðeins rauntímatengingarnar sem málið varðar að líta aftur.
- Tautulli-tengitaflan les söguna einu sinni, ekki einu sinni fyrir hvern Plex-notanda.
- Mynd sem vantar í skyndiminnið er sótt með einni beiðni á meðan hinar bíða, og ekki er beðið aftur í 30 sekúndur um mynd sem uppruninn gat ekki gefið.
- Skrár vefviðmótsins eru gzip-þjappaðar einu sinni og geymdar, og letur er ekki þjappað aftur.

### Stability
- Afrit á eldra sniði Jellystat er lesið röð fyrir röð í stað þess að vera lesið í heild, og línu sem er lengri en í nokkru afriti er hafnað.
- Tvær Tautulli-upphleðslur samtímis skrifa ekki lengur í sömu skrána.
- Síðunúmer langt umfram enda titla safns eða Heilbrigðis safns er tóm síða, ekki villa.
- Innflutningur eða endurheimt þar sem upphleðslan rofnaði lætur ekki lengur hafna öllum öðrum fram að endurræsingu, og hvorki hún né innflutningur sem var hætt við áður en hann hófst skilur eftir sig skrá.
- Ein röð afrits er ekki lesin lengra en 8 MB, og eitt Tautulli-afrit kemur í einu.
- Spilun geymir í mesta lagi 2.000 atburði, og innflutningur sleppir spilun sem hefst ekki og lýkur á milli 1990 og morgundagsins.
- Eitt þáttarnúmer langt undir núlli lætur ekki lengur hvern lestur safnsins hanga: Heilbrigði safns (samantektin líka), Afkastageta, endurtenging og staðsetning leggja saman og bera saman tölur án þess að fara hring, og útgáfusmíð athugar yfirflæði, svo það sem gleymist stöðvar eitt verkefni í stað þess að lenda í lykkju.
- Þáttaraða-, þáttar- eða lokanúmer er aðeins geymt þar sem Jellyfin gæti rúmað það (0 til 2.147.483.647), jafnt frá Jellyfin, Jellystat og Tautulli.
- Tautulli-afriti þar sem auðkenni endurtaka sig er hafnað, þar sem nokkur MB af röðum undir einu auðkenni tæmdu minni ferlisins, og afrit er opnað óbreytanlegt og skilur enga `-wal` eða `-shm` eftir.
- Tenging verður að senda beiðni sína innan 15 sekúndna og í mesta lagi 512 er þjónað í einu: nokkur hundruð hálfopnar tengingar tóku FinStats úr sambandi. Rammi á rauntímarásinni er takmarkaður við 4 KB.
- Nokkur kílóbæti frá Sonarr, Radarr eða Seerr geta ekki lengur orðið að gígabætum af minni: listi geymir ekki meira en FinStats les af honum.
- Viðtakandi tilkynninga sem svarar aldrei tefur aðeins sjálfan sig: sent er til viðtakenda samhliða, og sá sem rennur sífellt út á tíma er látinn í friði í tíu mínútur.
- Eftirstandandi tími og heildarhraði biðraðar, tölur endurheimtrar spilunar og dagurinn sem spilun fer úr ruslinu geta ekki lengur flætt yfir.
- Endurheimt les alla skrána áður en hún tekur skrifalásinn, hafnar skrá sem afþjappast í miklu meira en stærð hennar, og skrifar í lotum, svo söfnunin heldur áfram að skrá á meðan.
- Afrit eru nefnd í UTC og aldrei skrifuð yfir skrá sem er til, og aðeins eigin afrit FinStats frá því fyrir uppfærslu eru grisjuð.
- Lína í atvikaskrá þjóns dagsett í framtíðinni stöðvar ekki lengur lestur virkniskrárinnar: hún er lesin frá nýjustu línu sinni sem er ekki í framtíðinni.
- Beiðnir sem berast samtímis á opinberan tengil reikna svar hans út einu sinni, og spjöld hans eru teiknuð á eigin braut, svo handhafi tengilsins getur ekki lengur látið Uppgjörsspjöld eigandans svara 503.
- Spilanir safns sem vantar á lista Jellyfin yfir söfn bíða í tvo daga áður en þær eru færðar á afrit sama titils í öðru safni. Einn listi sem sleppti safni færði þær strax, og þær voru áfram færðar þegar safnið kom aftur.
- Diskur sem Jellyfin segir stærri en nokkur tala rúmar lætur ekki lengur hvern lestur á upplýsingum þjónsins mistakast.
- Lestri frá Jellyfin lýkur þegar síða endurtekur sig án nýrra auðkenna.

### Fixed
- Misheppnaðar innskráningar á aðgang eru ein viðvörun á hvert tímabil sem verður að viðvörunarboði við tuttugu, og nafn telst gegn aðgangi aðeins þegar það er slegið inn í heild ("x alice" telst ekki lengur gegn alice).
- Discord- og Slack-skilaboð eru fyrst varin (escaped) og svo stytt: tækisnafn fullt af Markdown lét Discord hafna hverri viðvörun sem nefndi það.
- Opinbert IPv6-vistfang er samþykkt fyrir persónulegan viðtakanda.
- Á bak við milliþjón sem skráir fólk sjálfur inn (nginx `auth_basic`) er köku FinStats ekki lengur hafnað.
- Birtingarnafn sem inniheldur `{{url}}` er sýnt eins og það er skrifað í forskoðun opinberrar síðu.
- Samantekt fyrir árið 9999 sýnir sjálfgefna árið í stað villu.
- Lifandi síða sem missti heimild og fékk hana aftur meðan hún var opin helst aftur uppfærð án endurhleðslu.

### Removed
- **Synnmørsk** er ekki lengur eitt af tungumálum FinStats: vafri sem valdi það les nýnorsku, ritmálið sem það var gert eftir.

## [2.3.0] - 2026-10-07 to 2026-10-26

Afkastageta, rauntímarás, Einfaldur hamur og tungumál

### Added
- **Tungumál**: FinStats talar 23 tungumál. Breska og bandaríska ensku og norskt bókmál og nýnorsku skrifar umsjónarmaðurinn og fer sjálfur yfir; danska, þýska, spænska, finnska, franska, hindí, íslenska, ítalska, japanska, kóreska, hollenska, pólska, portúgalska (Brasilía), rússneska, sænska, tyrkneska, úkraínska og kínverska (einfölduð og hefðbundin) eru þýðingar sem enginn hér hefur farið yfir, og segja á ensku það sem þær hafa ekki enn. Hver síða í báðum framsetningunum, opinberi prófíllinn, útgáfulýsingarnar og það sem þjónninn svarar í orðum (villuboð, spjöld Uppgjörsins, hvað verkefni í Jellyfin gerir) birtist á tungumálinu sem er valið undir Stillingar → Útlit → Tungumál, fyrir hvern vafra; þar til eitt er valið ræður tungumál vafrans sjálfs. Tölur, dagsetningar, einingar og upptalningar eru skrifaðar eins og það tungumál skrifar þær. Forskrift getur beðið um tungumál með `Accept-Language`.
- **Einfaldur hamur**: önnur framsetning fyrir þá sem koma frá Plex, sett upp eins og klassískur tölfræðiþjónn fyrir Plex. Stika þvert yfir efst í stað valmyndarinnar, og aðeins tölfræðisíðurnar: Forsíða, Söfn, Notendur, Ferill, Gröf og Stillingar, með tannhjóli fyrir prófílinn þinn, Stillingar, Útgáfulýsingar, þemað og útskráningu. Valinn fyrir hvern vafra undir Stillingar → Útlit → Framsetning. Hann hleður aðeins sínum helmingi af FinStats og ræsist því hraðar; Uppgjör, Saman, Á leiðinni, Öryggi og síður þjónsins eru áfram í framsetningu FinStats. Hann ber eigin liti FinStats, ljósa og dökka, og ekkert í honum hreyfist.
- **Forsíða í Einföldum ham**: það sem er í spilun, spjald fyrir hvert streymi (hvernig það spilast, heiman frá eða utan heimanets, hvenær því lýkur), áhorfstölfræðin sem röðuð spjöld eftir fjölda spilana eða spilunartíma síðustu N daga, hvað hvert safn geymir og hvað kom nýlega.
- **Gröf í Einföldum ham**: spilanir eftir tegund efnis (dag frá degi, eftir vikudegi og klukkustund, tíu efstu forritin og notendurnir), eftir því hvernig þær spiluðust (dag frá degi, flest samtímis streymi hvern dag, upplausn skrárinnar og sú sem var send, forrit og fólk) og mánuð fyrir mánuð, eftir fjölda spilana eða spilunartíma, fyrir alla eða þá sem eru valdir.
- **Söfn, Notendur og titlar í Einföldum ham**: töflur yfir hvert safn og hverja manneskju með því sem síðast var spilað, á hverju og hvaðan (vistfangið aðeins með *Sjá netupplýsingar*), og síða fyrir hvert safn, hverja manneskju og hvern titil með tölum þess og eigin ferli.
- Fyrir forskriftir: `/api/stats/split` (spilanir eftir forriti, manneskju, upplausn, degi eða mánuði, skipt eftir tegund efnis eða eftir því hvernig þær spiluðust); topplistar eftir fjölda ólíkra áhorfenda (`sort=users`); virkni þrengd að spilunum sem teljast áhorf (`watched=1`); innsýn aðeins í þá hluta sem beðið er um (`parts=`); mesta samtímaálagið skipt í umkóðanir, beint streymi og beina spilun, líka fyrir hvern dag; hvað hvert safn geymir og hvað var síðast spilað þar; nýlega bætt við eftir tegund (`kind=`); hitakortið eftir tegund efnis; síðasta spilun manneskju; hvort streymi spilast heiman frá.
- **Rauntímarás**: yfirlitið, niðurhalið á síðunni Á leiðinni og stöðustikan halda sér uppfærðum yfir eina WebSocket-tengingu á hvern flipa (`/api/live`), svo það sem er í spilun og það sem verið er að sækja breytist á skjánum um leið og það gerist, ekki við næstu athugun. Á bak við öfugan milliþjón (reverse proxy) sem hleypir WebSocket-uppfærslum ekki í gegn, eða á tengingu sem slitnaði án þess að segja orð, spyrja síðurnar á nokkurra sekúndna fresti eins og áður. `/api/now-playing`, `/api/downloads` og `/api/summary` haldast óbreytt, fyrir forskriftir.
- **Þjónn → Afkastageta**: ræður þessi þjónn við álagið, miðað við það sem FinStats hefur séð? Annasömustu stundirnar sem skráðar eru, og þær síðustu 90 daga: streymi samtímis, og hve mörg þeirra fengu myndina umkóðaða með vélbúnaði, án hans, eða án þess að nokkuð væri gefið upp (ferill sem er fluttur inn úr Tautulli er ferill Plex og er ekki tekinn með). Hver diskur sem söfnin eru á og hve hratt það sem Jellyfin bætti við á sex mánuðum fyllir hann (diskur sem var fylltur nýlega er mældur frá mánuði eftir fyrstu skrána á honum, því fyrsti mánuðurinn er áfyllingin sjálf), og, þar sem Jellyfin 10.11 eða nýrra gefur upp laust pláss, hve lengi það endist á þeim hraða. FinStats segir aldrei hversu mörg streymi þjónninn gæti ráðið við.
- **Flytja út .ics** á Á leiðinni → Væntanlegt: listinn eins og þú síaðir hann, sóttur sem dagatalsskrá til að flytja inn einu sinni. Hann uppfærist ekki sjálfur; dagatalslykill úr Stillingar → API-lyklar gefur enn dagatal sem gerir það.
- **Vert að skoða**: Afkastageta bendir á þegar hver einasta umkóðun sem Jellyfin sagði frá á 90 dögum keyrði án vélbúnaðarhröðunar þótt umkóðun sé algeng, þegar safnskönnun Jellyfin er í gangi fjórðung tímans eða meira, og þegar diskur á minna en sex mánaða pláss eftir. Hverju atriði fylgja tölurnar að baki, og það má leggja til hliðar eins og athugasemdirnar í Heilbrigði safns.
- **Úr hverju safnið þitt er** má þrengja að einu safni eða fleirum, undir Söfn og á síðu hvers safns (`library_id=a,b`).
- **Snið dagsetninga og tíma** í Stillingar → Útlit: dagsetning eins og 31.01.2026, 31/01/2026, 31-01-2026, 01/31/2026, 2026-01-31, 2026.01.31 eða 2026/01/31, og tími í 24 eða 12 tímum, með eða án núlls á undan, eða hvort tveggja eins og tungumálið skrifar það. Geymt í þessum vafra.
- **Móttaka**: í fyrsta sinn sem einhver skráir sig inn, í hvaða vafra sem er, og í lok uppsetningar, spyr FinStats um tungumál, snið dagsetninga og tíma og útlit, með mynd af hverju. Sleppa heldur breskri ensku, 31/01/2026, 13:05 og FinStats.

### Changed
- Merkið inni á síðu er bert táknið við hlið nafnsins, sett í Inter hvaða letur sem forstilling velur; þar sem það stendur eitt (vefsíðutáknið) heldur það ávala reitnum sínum.
- Spilun sem var umkóðuð í hugbúnaði er skráð sem slík þar sem Jellyfin segir það; áður leit hún eins út og spilun frá þjóni sem segir það ekki. Síðurnar kalla hvort tveggja enn Hugbúnaður.
- Uppfærsla keyrir nú, einu sinni og elstu útgáfuna fyrst, það sem útgáfa þarf að gera við gagnagrunn sem eldra FinStats skrifaði, auk skemabreytinga sinna. Sú fyrir 2.3.0 les upplýsingar þjónsins við fyrstu ræsingu, svo Afkastageta er til staðar strax.

### Performance
- **Síða sem stendur opin biður ekki um neitt**: yfirlitið sendi 30 beiðnir á mínútu til að haldast uppfært og niðurhalið á Á leiðinni 18; með rauntímarásinni sendir hvorugt neina. Hversu margir flipar sem eru opnir biður FinStats Jellyfin, Sonarr og Radarr ekki um neitt umfram það sem áður var, og falinn flipi leyfir niðurhalsröðinni enn að hægja á sér.
- Síða manneskju les aðeins spilanir þeirrar manneskju: heildartölur hennar og síðasta spilun voru reiknaðar yfir allar spilanir á þjóninum og síðan þrengdar að henni, um fjórar sekúndur á milljón spilunum.
- Síða safns les aðeins titla og spilanir þess safns, þar sem hún taldi áður öll söfn og hélt einu eftir: þriðjungur tímans á milljón spilunum.
- Þegar klukka Jellyfin var á undan klukku FinStats las *Á eftir safnskönnun Jellyfin* allt safnið aftur á hverri mínútu þar til munurinn var liðinn; nú er hverri skönnun fylgt einu sinni.
- GeoIP-skrá sem ekki var hægt að opna var afrituð aftur, hundrað megabæti og meira, og hvert vistfang staðsett aftur, á hverjum fjórðungi klukkustundar; nú er hún aðeins reynd aftur þegar hún breytist.
- Heilbrigði safns opnast strax eftir að bendillinn hefur hvílt á tenglinum: það sem var sótt fyrir fram var geymt þar sem síðan leit aldrei.

### Stability
- Uppfærsla sem mistókst eftir að skemabreyting hafði keyrt, og Docker endurræsti strax, tók nýtt afrit af hálfuppfærða gagnagrunninum við hverja ræsingu, og þrjár ræsingar grisjuðu burt eina afritinu sem fyrri útgáfan gat opnað. Afritið frá því fyrir uppfærsluna er nú geymt.
- Titill sem var eytt í Jellyfin meðan safnið var lesið (skrá sem Radarr skipti út) gat valdið því að lesturinn missti af öðrum titli sem var enn til staðar og merkti hann fjarlægðan, og spilanir hans gátu þá færst yfir á annað eintak af honum. Nú er fyrst spurt eftir auðkenni um það sem lestur sá ekki.
- Endurheimt öryggisafrits sem var tekið áður en skrá var endurnefnd eða uppfærð endurheimti aftur hverja spilun sem síðan hafði færst yfir á nýju skrána, svo hún var tvítalin fyrir fullt og allt.
- Beiðni í svari Seerr sem FinStats gat ekki lesið (reitur sem var endurnefndur í uppfærslu á Seerr) var merkt fjarlægð við daglega heildarlesturinn.
- Verkefni sem lenti á villu var „í gangi“ fyrir fullt og allt, og engin kveikja ræsti það aftur fyrr en FinStats var endurræst; nú lýkur því með villu.
- Skemmd zip-skrá frá Tautulli stöðvaði upphleðsluna áður en skránni var eytt, en hún geymir tóka allra Plex-notenda, og þar lá hún fram að sex klukkustunda hreinsuninni.
- Innflutningur eða endurheimt sem var stöðvuð meðan skráin var að berast skildi þá skrá eftir í gagnamöppunni fyrir fullt og allt, vistföng og allt; henni er nú eytt við næstu ræsingu.
- Mynd gat verið afgreidd hálfskrifuð meðan verið var að setja hana í skyndiminni, og vafrinn hélt henni þá þannig.
- Afturkölluðum API-lyklum var haldið að eilífu, hversu margir sem voru búnir til og afturkallaðir.
- Lestur á merkjum einhvers um spilað og eftirlæti sem kom tóm eða ólæsileg til baka hreinsaði þau öll fram að næsta lestri; henni er nú hafnað, eins og tómum lestri safns.
- Þáttur með lokanúmer sem enginn þáttur getur haft (dagsetning skrifuð þar) gat haldið Heilbrigði safns uppteknu endalaust eftir lestur safnsins.

### Fixed
- Útgáfulýsing með kóða inni í feitletrun (í 2.0.2 og 2.0.3) sýndi bakkommurnar í stað þess að birta kóðann.
- Graf yfir fjölda gat merkt ásinn sinn 0, 3, 5, 8, 10 eða jafnvel 0, 0, 1, 1, 1: þrep upp á 2,5 skrifuð námunduð. Fjöldi hækkar nú í heilum þrepum.
- Dagatalslykli sem einhver bjó til sem má ekki stjórna FinStats fylgdi tengill á eigið vistfang vafrans í stað opinbers vistfangs FinStats, svo hann virkaði aðeins innan heimanetsins; síðan bað líka um stillingar sem henni var neitað um.
- Síðasta spilun manneskju gat verið nokkrum sekúndum of stutt til að teljast spilun annars staðar, og nefndi þá titil sem Ferill hennar sýnir ekki.
- Endurheimt úr Nýlega eytt sem mistókst sagði ekkert og lét hnappinn sinn halda áfram að snúast; sama gerði misheppnað *Sækja aftur* í Heilbrigði safns og misheppnað *Opna aftur* eða *Leysa allar* í Öryggi.
- Dagatalsstraumurinn heitir FinStats í dagatalsforriti, ekki finstats.
- Niðurhalsröðin sagði hverjum þeim sem mátti sjá hana hver hefði beðið um hvert niðurhal; beiðni einhvers annars krefst nú *Sjá virkni allra*, eins og á Á leiðinni.
- Þegar *Fá tilkynningar* var tekið af manneskju var ekki hætt að senda til viðtakenda hennar, og hún gat ekki lengur slökkt á þeim.
- Spilun sem gert var hlé á og haldið áfram innan samrunabilsins var tilkynnt stöðvuð einu sinni, en framhald hennar og raunveruleg lok aldrei.
- Spilanir lags gátu færst yfir á lag annars flytjanda með sama nafni („Intro“) þegar plata þess hvarf úr safninu.
- Viðtakandi tilkynninga sem var skrifaður sem IPv4-vistfang inni í IPv6 (NAT64, 6to4) taldist opinber þótt hann næði inn á staðarnetið.
- Sonarr- eða Radarr-tenging sem var beint að öðrum þjóni hélt áfram að sýna veggspjöld gamla þjónsins í viku.
- Viðvaranirnar á síðunni Öryggi sögðu að ekkert þyrfti að skoða þegar tvær manneskjur eða fleiri voru valdar, og síða sem tæmdist við að leysa síðustu viðvörunina á henni átti enga leið til baka.
- Hver API-lykill sem var ekki útrunninn sagðist hafa runnið út „rétt í þessu“.
- Svargluggi stóð áfram yfir síðunni eftir að farið var til baka, og ekki var hægt að fletta síðunni fyrr en honum var lokað.
- Endurskoðunarskráin sýndi átta tegundir færslna (spilun eða afrit sótt aftur, ruslið tæmt, opinberir prófílar, útliti breytt) með auðkennum sínum, og ekkert í Nánar.
- Þegar titill var settur á áhorfslistann úr samhengisvalmynd hans mistókst það án orðs ef þjónninn neitaði.
- *Horft saman* á yfirlitinu fór með nokkrar manneskjur sem voru valdar í einu eins og eina manneskju.
- Uppgjörið skrifaði klukkustundir og daga undir tíu með tugakommu vafrans, við hliðina á eigin þúsundaskilum sem voru líka kommur.
- Afkastageta sagði að diskur fylltist "á hraða síðustu 0 mánaða".
- Á síðunni Öryggi töldu vistföng án staðar spilanir allra frá upphafi, sama hvaða manneskju og tímabil síðan var þrengd að.
- Einhver með *Stjórna FinStats* en ekki *Sjá virkni allra* gat sett spilun hvers sem er í ruslið.
- Spjald úr Uppgjöri eða opinberum prófíl gat komið úr skyndiminni vafrans á tungumálinu sem valið var áður.
- Niðurhölin sem beina rásin sendi voru á breskri ensku, sama á hvaða tungumáli síðan var.
- Innskráning sem breytti því hvað einhver má sjá náði til hinna opnu flipa viðkomandi aðeins við næstu athugun eftir hálfa mínútu.
- Flipi sem lét beina tengingu sína eftir nýrri flipa tók eina aftur á næstu síðu, svo flipar sömu manneskju lokuðu sífellt hver á annan.
- Töflur röðuðu stærðum frá 1.000 til 1.023 einingum sem texta í tungumálum sem flokka tölustafi með bili, og tyrkneskum prósentum líka.
- Titill með staðgengli í nafninu, svo sem "Big {device} Bunny", var fylltur út í skilaboðum þjónsins.
- Töflur röðuðu tímalengdum og fjölda af heild (5 af 26) sem texta á öllum tungumálum nema ensku og þeim norrænu.
- Flipi sem var enn opinn á einu tungumáli fékk svör þjónsins á öðru um leið og annar flipi hafði valið það.
- Orð við hlið tölu fékk ranga mynd á pólsku, rússnesku og úkraínsku ("5 wieczory", "2 потоков").
- Afkastageta taldi fyrstu fyllingu safns sem vöxt og deildi því sem hvert safn bætti við með sex mánuðum, sama yfir hve langan tíma var mælt; skrá sem tók stað annarrar (uppfærsla úr Radarr eða Sonarr, eða skrá sem Jellyfin gaf nýtt auðkenni) taldist öll ný, þegar aðeins mismunurinn var nýr.
- Þau sem horfðu saman með SyncPlay voru ekki hópur þegar annað bættist við nokkrum mínútum á eftir hinu; spilanir sem enda samtímis á sama stað eru saman núna, og ferillinn er flokkaður aftur einu sinni.
- Einhver án *Sjá virkni allra* gat lesið úr *Aldrei horft á* í safni að einhver annar hefði spilað titil; nú telur það aðeins eigið áhorf.

## [2.2.0] - 2026-10-01 to 2026-10-07

Heilbrigði safns og FinUI

### Added
- **Þín eigin valmynd í tölvu**: Stillingar → Útlit býður upp á átta: hliðarstikuna eins og hún var, hliðarstiku flokkaða eftir því til hvers síðurnar eru, stiku af táknum sem opnast í hliðarstikuna undir bendlinum, hliðarstiku sem byrjar á leit, dokku, skipanastiku, síður sem þú festir efst, og litaðar flísar. Hliðarstikurnar geta verið hægra megin. Dokkan rennur burt meðan þú flettir niður og kemur aftur þegar þú flettir upp eða bendir á neðri brún gluggans; með henni færist stöðustikan efst.
- **Þín eigin valmynd í síma**: flipastika með Meira (nýja sjálfgefna valið), blað sem opnast til hálfs, allur skjárinn, síðurnar í blævæng kringum þumalinn, eða veffangastika sem vex í valmyndina. Hver þeirra leitar í síðunum sínum og sendir önnur orð áfram í leitina.
- Vafrinn sem valið var í geymir hvort tveggja, svo sími og tölva halda hvort sínu; hvort tveggja eru FinUI-einingar, `desktop-nav` og `mobile-nav`.
- **Samhengisvalmynd**: hægrismelltu á titil, manneskju eða safn hvar sem er í FinStats (veggspjald á yfirlitinu, línu í lista, nafn í töflu, kvikmynd á spjaldinu Á næstunni sem er ekki enn í safninu), eða haltu fingri á því, eða ýttu á valmyndartakkann, og valmynd vex út frá punktinum sem þú ýttir á: opna það, opna það í nýjum flipa, setja kvikmynd eða þáttaröð á áhorfslistann þinn, opna það í Jellyfin eða á TMDB og TVDB, afrita tengil þess. Shift með hægrismelli gefur enn valmynd vafrans sjálfs. `context-menu` í FinUI.
- **Leit án svarglugga**: leitin vex út úr valmyndinni þinni (hliðarstika breikkar í hana, dokkan rís upp í hana, nafn skipanastikunnar breytist í hana, valmynd símans heldur henni) eða, ef það er valið í Stillingar → Útlit, tekur sæti síðunnar meðan valmyndin stendur. Ctrl+Space opnar hana eins og áður; glugginn yfir öllu er horfinn. `search` í FinUI.
- **Allt í safni**: spjaldið Nýlega bætt við á síðu safns býður upp á *Allt sem er í því*, hverja kvikmynd, þáttaröð eða plötu sem safnið geymir, sem veggspjöld, eftir nafni, ári, dagsetningu viðbótar eða stærð, þrengt meðan þú skrifar. Fleiri bætast við þegar þú flettir.
- **Þjónn → Heilbrigði safns**: FinStats ber hverja skrá saman við nágranna sína og telur upp það sem sker sig úr, hvert atriði með tölunum að baki. Það gefur aðeins skýrslu; engu er breytt eða eytt í Jellyfin og ekkert skannað aftur.
- **Göt**: þættir sem vantar milli tveggja skráa í seríu („Sería 2: þætti 4 og 5 vantar milli 3 og 6“). Skrá sem geymir tvo þætti telst sem báðir; aukaþættir og lok seríu eru látin vera.
- **Seríur sem skera sig úr** í upplausn, HDR eða kóðun frá restinni af þáttaröðinni, og **stakir þættir**: ein skrá ólík seríunni sinni, svo sem SD-þáttur í 1080p-seríu. Upplausn fer eftir breidd eða hæð, svo skorin 1920×800 kvikmynd er 1080p.
- **Tvítök og útgáfur**: sama kvikmynd eða þáttur tvisvar, eftir auðkennum veitenda sem áhorfslistinn og Á leiðinni styðjast þegar við. Tvö eintök í sömu upplausn sýna plássið sem er geymt tvisvar; 4K-eintak við hlið 1080p-eintaks er talið upp sem tvær útgáfur og aldrei talið sóun.
- **Rýrar skrár**: bitahraði langt undir því sem upplausn og kóðun skrárinnar þurfa venjulega („4K HEVC á 1,1 Mb/s“); rýrir þættir seríu eru ein athugasemd.
- **Talsetning sem hættir**: hljóðtungumál sem sumar seríur hafa í hverjum þætti og aðrar í engum.
- **Aldrei borin kennsl á**: kvikmyndir og þáttaraðir sem Jellyfin paraði án auðkennis frá TMDB, TVDB eða IMDb.
- **Hunsa** athugasemd sem er viljandi eins og hún er, með skýringu. Hún kemur aftur af sjálfu sér þegar skrárnar að baki breytast. Þeir sem mega stjórna FinStats og sjá þjóninn geta hunsað; hunsanir eru skráðar í endurskoðunarskrána og eru hluti af öryggisafriti.
- Síða safns segir hve margt er þar að skoða, og vísar á það.
- **FinUI**, einingarnar sem FinStats er byggt úr, er sjálfstætt verkefni (`github.com/finstats/finui`): hver hnappur, spjald og stýring í hverju ástandi og báðum þemum á `finui.finstats.no`, og **FinUI create** til að breyta litum, rúnnun, þéttleika og fleiru meðan þú horfir á og taka það heim með einni skipun. **Stillingar → Kerfi** vísar þangað.
- **Stillingar → Útlit**: hver og einn velur hvernig FinStats lítur út fyrir sig, með kóða FinUI-forstillingar: grunnlitur, áherslulitur, litir grafa, birtuskil, rúnnun, þéttleiki, rammar, spjöld, hnappar, reitir, töflur, áhersla, hreyfing, tákn, valmyndin, síðan á bak við allt saman, fókushringurinn, hvernig fyrirsagnir eru settar, og letrið fyrir texta, fyrirsagnir og tölur. Val þitt er þitt eitt; síðan sýnir hvað kóðinn þýðir og vísar á FinUI create til að breyta honum.
- **Stílar**: ellefu heildarútlit til að byrja á með einu vali í FinUI create, og Stillingar → Útlit tekur við kóða þeirra. Þeir eru Washi (FinStats eins og það kemur), Studio, Noir, Matinee, Console, Pastille, Ledger, Gazette, Arcade, Canopy og Harbour. FinStats sjálft kemur með eitt útlit og býður engan lista af stílum.
- **Tákn sem hreyfast**: hvert tákn í tengli, hnappi eða flipa gerir sína eigin hreyfingu einu sinni þegar bent er á það (endurhleðslutáknið snýst, ör niðurhalsins fellur ofan í bakkann, ruslafatan lyftir lokinu), og endurhleðsla sem ýtt var á heldur áfram að snúast þar til svarið er komið. Kyrr með minnkaðri hreyfingu, og með Hreyfing: Af.
- **Hreyfing, með FinMotion**: einingar FinUI hreyfast nú: hnúð rofa er kastað yfir, undirstrik flipa mjakast að næsta flipa, reitur sem er hafnað hristir höfuðið, afritun er hökuð við í einu striki, tilkynningar hlaðast upp eins og spil á hendi og breiðast út þegar bent er á þær, blað í síma gefur eftir eins og teygja þegar togað er í það, línur raðaðrar töflu færast á sinn stað, tala sem breytist rúllar eins og kílómetramælir og gröf teikna sig sjálf í fyrsta skipti sem þau sjást. Allt fylgir vali Hreyfingar í Stillingar → Útlit og stendur kyrrt með minnkaðri hreyfingu. FinMotion er sjálfstætt verkefni (`github.com/finstats/finmotion`), sett ofan á FinUI.
- **Fleiri tákn í FinUI**: 239 núna, en voru 71, hvert með sína eigin hreyfingu (hljóð upp og niður, heyrnartól og heyrnartól með slökkt á hljóði, blýantur sem skrifar, bjalla sem hringir, sími sem titrar), og hvert þeirra má sækja úr sýningarsafni FinUI sem SVG, kyrrt eða hreyfanlegt.
- **Letur**: þrjátíu og fjórar leturfjölskyldur með opnu leyfi til að velja úr (Geist, Roboto, Manrope, IBM Plex Sans, Montserrat, Lora, Playfair Display, Fira Code og fleiri), innbyggðar eins og Inter, aldrei sóttar annað, og aðeins hlaðnar þegar þær eru valdar.
- **Fleiri grunnlitir og áherslulitir**: Nord, Solarized, Gruvbox, Catppuccin, Rosé Pine, Tokyo Night, Dracula, Everforest og Ocean; sextán áherslulitir í viðbót, tíu þeirra einn litur að degi og annar að nóttu. Hver grunnlitur hefur nú sinn eigin pappír að degi: gráu litirnir litu eins út í ljósa þemanu.
- **Hægt er að afturkalla eyðingu í 30 daga**: spilun eða öryggisafrit sem er eytt bíður í ruslinu, og **Afturkalla** á tilkynningunni sem fylgir sækir það aftur. Virkni og Stillingar → Öryggisafrit telja hvor um sig upp það sem var nýlega eytt, og hvenær það hverfur fyrir fullt og allt, hvenær sem eitthvað er þar. Spilun í ruslinu er utan allra heildartalna, grafa, uppgjöra og prófíla, og innflutningur eða endurheimt sækir hana ekki aftur sem nýja; sá sem horfði á hana með einhverjum telst hafa verið einn þar til hún snýr aftur.

### Changed
- Dark Reader og sambærilegar viðbætur láta síður FinStats í friði (`darkreader-lock`): FinStats teiknar bæði þemun sjálft, og endurmáluð síða barðist við þau.
- Í síma rennur valmyndin ekki lengur inn frá vinstri: flipastikan situr neðst nema annað sé valið.
- FinStats les safnið þitt einu sinni eftir þessa uppfærslu, til að læra hvar skrár sem geyma nokkra þætti enda.
- Á síðu titils er hnappurinn **Bæta á áhorfslista** jafnhár og **Opna í Jellyfin** við hliðina; hann var einni stærð minni.
- Síurnar fyrir tímabil og fólk haldast efst á skjánum meðan síðan flettist, svo hægt er að breyta 7 d í Allt neðst á langri síðu; sama gildir um síurnar fyrir beiðnir og væntanlegt á Á leiðinni.
- Hnappurinn sem fer efst á síðuna heldur sig frá hvaða valmynd sem tekur neðri hluta gluggans: lágt í horninu við hlið dokkunnar, rétt fyrir ofan flipastiku, þumalhnapp eða veffangastiku símans, og annars fyrir ofan stöðustikuna. Hann er `to-top` í FinUI.
- Að eyða spilun eða öryggisafriti færir það í ruslið í stað þess að fjarlægja það strax (`PUT` með `{"deleted": true}`; `DELETE` er horfið úr API-inu). Öryggisafritum umfram fjöldann sem er geymdur er enn eytt fyrir fullt og allt.
- **Útgáfulýsingar** eru ein örk í stað stafla af spjöldum: hver lýsing er sín eigin lína með tegund sinni (Bætt við, Breytt, Lagfært og hinar) og tákni þeirrar tegundar í dálki fyrir framan, undir fyrirsögn fyrir hverja röð. Útgáfulistinn er áfram til hægri.

### Performance
- Þegar FinStats er opnað er aðeins sótt það sem breyttist: stílsnið, forskrift eða letur sem vafrinn hefur þegar er staðfest með nokkrum bætum í stað þess að vera sent aftur. Hver heimsókn sótti áður öll stílsnið og JavaScript-einingar FinStats í heild, þótt hver bæri merki til að greina þau í sundur.

### Stability
- Endurheimt öryggisafrits les stillingarnar aðeins úr stillingalínunni: skrá frá nýrra FinStats sem geymir meira þar missti stillingarnar sínar án orðs.
- Opinn flipi stækkar ekki lengur með hverri síðu sem er heimsótt: hver síða sem farið var af sat eftir í minni á bak við þá sem var á skjánum (um 2.600 stök í hverri heimsókn), því listi FinMotion yfir það sem það hreyfir og flettihlustari síustikunnar héldu í hana, þar til flipanum var lokað.
- Rangt slegin skipun (`finstats bakcup`), eða `restore` án skrárinnar, býr ekki lengur til eða uppfærir gagnagrunn áður en hún segist ekki vita hvað hún eigi að gera.

### Fixed
- Lítið graf í reit rennur ekki lengur út fyrir reitinn þegar talan við hliðina er breið („56 d 23 klst“), eða þegar fjórir reitir deila þrengri röð: grafið víkur og talan heldur breidd sinni.
- Í ljósa þemanu varð aðalhnappur fjólublár undir bendlinum; hann heldur nú lit innsiglisins.
- Í Stillingum situr rofi í sömu hæð og nafn stillingarinnar og hjálpartextinn; hann hékk efst í línunni.
- Í röðuðum lista með ekkert undir nafni, svo sem Virkustu notendurnir á yfirlitinu, sat nafnið efst í línunni við hlið miðjaðrar myndar; það er nú miðjað.
- Röð af tölfræðireitum á breidd milli síma og tölvu (fjórir reitir, þrír í röð) skildi þann síðasta eftir einan í eigin röð; fjórir reitir raðast nú fjórir, tveir eða einn í röð, og þrír raðast þrír eða einn.
- Lestur þjónsins taldi tæki sem enginn skráði sig inn á, sem hann geymir ekki: verkefnið sagði þrjú tæki þar sem síðan Þjónn taldi upp tvö.
- Ein sekúnda, ein mínúta, ein klukkustund: viðvaranir sögðu „1 seconds apart“ og „in 1 minutes“, og verkefni í Jellyfin á einnar mínútu bili keyrði „every 1 minutes“.
- `finstats --help` nefnir hverja breytu sem FinStats les; `FINSTATS_PUBLIC_IP_URL`, `FINSTATS_ALLOW_LIBRARY_SHRINK` og `FINSTATS_SKIP_PREUPDATE_BACKUP` vantaði.
- Uppsetningarálfurinn sagði um vistfang sem var ekki vistfang að hann „couldn’t connect“, í setningu sem rann saman við þá næstu; nú segir hann að vistfangið sé ekki vistfang.
- Í Stillingar → Öryggisafrit hvarf spurning sem var borin upp meðan verið var að skrifa öryggisafrit þegar afritinu lauk.
- Með valmyndina sem stiku sem opnast sýndi lokun leitarinnar opna leitina og lokuðu stikuna samtímis og skildi síðan stikuna eftir opna þar til bendillinn kom og fór. Esc skilar nú lyklaborðsfókusnum þangað sem hann var þegar leitin opnaðist, og sleppir fókus sem smellur skildi eftir.
- Í síma hélst síðan dempuð eftir að leitinni var lokað þar til leitin var horfin, og varð svo björt í einum ramma; dempunin dofnar nú með leitinni.
- Skref Tautulli-innflutningsins sögðu að öryggisafritinu væri hlaðið niður þegar ýtt væri á **Backup Database**. Tautulli vistar það á eigin vél: skrefin segja nú hvar (í `backups`-möppunni þess, `/config` með Docker), hvaða skrá á að taka, og að afrita hana yfir.
- Leit sem var lokað áður en hún hafði opnast að fullu skildi stikuna sem opnast eftir án hreyfingar: upp frá því opnaðist hún og lokaðist samstundis.
- Stikan sem opnast hélst opin eftir að síða var valin úr henni þar til smellt var annars staðar, og í sumum vöfrum, eða með aðdrætti, teiknaði hún skrunstiku undir táknunum. Nú opnast hún aðeins fyrir bendil og lyklaborð, og teiknar enga skrunstiku meðan hún er lokuð.
- Á síðunni Öryggi telst vistfang sem gagnagrunnurinn staðsetur í landi en ekki á kortinu með því landi, fremur en með vistföngunum sem hafa engan þekktan stað.

## [2.1.2] - 2026-10-01

### Added
- **Innflutningur úr Tautulli**: taktu Plex-ferilinn þinn með þér. Hladdu upp öryggisafriti af gagnagrunni Tautulli (`.db`-skránni, eða `.zip`-skránni sem geymir hana) undir **Stillingar → Innflutningur**; spilanir þaðan fá sinn eigin stað í síunni *Skráð af* á Virkni.
- **Tengiborð til að segja hver er hver**: Plex-notendur öðrum megin, Jellyfin-notendur hinum megin. Dragðu snúru frá hverjum Plex-notanda til þess sem hann er núna, eða smelltu á annan og svo hinn, eða notaðu lyklaborðið. Þeir sem eru ótengdir eru ekki fluttir inn, og tveir Plex-aðgangar geta farið í eina manneskju.
- Kvikmyndir og þættir eru paraðir við safnið þitt eftir nafni (kvikmynd eftir titli og ári, þáttur eftir þáttaröð og númeri), og titill sem er ekki enn á þjóninum þínum er geymdur og tengdur þegar hann kemur. Áhorf sem haldið var áfram er ein spilun; tónlist er ekki flutt inn.
- Öryggisafritið geymir aðgangstóka Plex: FinStats les þá aldrei, og eyðir skránni um leið og þú flytur inn eða byrjar upp á nýtt.
- **Ótengt efni** undir **Stillingar → Gögn**: hver titill frá Tautulli, Jellystat eða Streamystats sem passar ekki við neitt í safninu þínu, með því hvaðan hann kom og hvar hann er líklegast þegar að finna. **Finna** tengir hann út frá bestu ágiskunum eða leit (kvikmynd má finna sem aukaþátt þáttaraðar), og FinStats man valið, svo sami ferill tengir sig sjálfur ef hann er fluttur inn aftur. Valin eru hluti af öryggisafriti.

### Performance
- **Uppgjörið opnast þrisvar til fimm sinnum hraðar**: leikararnir og leikstjórarnir sem mest var horft á á árinu voru reiknaðir út með því að skanna árið aftur einu sinni fyrir hverja manneskju í því. Mælt á raunverulegum ferli fór ár úr 0,29 s í 0,09 s og síðustu 12 mánuðir úr 0,53 s í 0,12 s.

### Fixed
- **Innfluttur ferill finnur titla sem ganga undir öðru nafni**: spilun parast líka við nafn titils á frummálinu, svo þáttaröð sem Plex eða Tautulli þekkti sem 오징어 게임 tengist Squid Game, og *Im Westen nichts Neues* tengist *All Quiet on the Western Front*. Upprunalegir titlar safnsins þíns eru lesnir við næstu skönnun þess.
- Nöfn eru borin saman eins og leitin ber þau saman (hástafir, broddstafir, greinarmerki, ártal skrifað inn í nafnið, „The“ fremst), og útgáfuárinu má skeika um eitt, svo „Star Wars: Episode I - The Phantom Menace“ finnur „… – The Phantom Menace“ og Kingsman frá 2014 finnur þá frá 2015. Á titil sem gengur undir allt öðru nafni er samt aldrei giskað.
- Þegar innflutningur eða endurheimt úr Streamystats mistókst var það nefnt í tilkynningum með innra heiti sínu.
- Hnappurinn *Deila árinu sem spjöldum* í uppgjörinu og sæti áhorfandans við hliðina höfðu ekkert bil á milli sín.

## [2.1.1] - 2026-10-01

### Changed
- **Áhorfslistinn er í boði hvar sem titill er**: á spjöldunum Á næstunni á yfirlitinu, eins og á Á leiðinni → Væntanlegt, og á síðu þáttar eða seríu, sem setur þáttaröðina á listann.
- Á veggspjaldi á yfirlitinu birtist hnappur áhorfslistans meðan þú bendir á spjaldið eða nærð því með lyklaborðinu, í stað þess að hylja horn hvers veggspjalds; á snertiskjá er hann alltaf sýnilegur.

### Stability
- Ef Sonarr-, Radarr- eða Seerr-tenging var vistuð meðan einhver hafði **Stillingar → Tengingar út á við** opið gat þetta tvennt beðið hvort eftir öðru að eilífu.
- **Stillingar sem ekki er hægt að lesa stöðva FinStats við ræsingu**, og nefna stillinguna sem veldur, í stað þess að endurstilla hljóðlega hverja stillingu á sjálfgefið gildi.
- **Tvær breytingar á stillingunum samtímis** (til dæmis stillingasíðan og áætlun verkefnis) afturkalla ekki lengur hvor aðra.
- Hlé, spólun eða skipti á rás sem sást meðan gagnagrunnurinn var upptekinn er skrifað í næstu umferð í stað þess að glatast.
- Lestur safns mistekst ekki lengur á áhorfslista sem geymir nokkrar færslur sem reynast vera einn titill.

### Fixed
- **Skrá sem var skipt út er ekki „nú á þjóninum“**: þegar skrá kvikmyndar á áhorfslista einhvers er uppfærð er viðkomandi ekki lengur sagt að hún sé nýkomin.
- Einn misheppnaður lestur á viðbótum, áætluðum verkefnum eða geymslurými Jellyfin tæmir ekki lengur þann lista á síðunni Þjónn fram að næsta lestri.
- **Spilun úr Streamystats stöðvast þar sem hún stöðvaðist, á hverri síðu**: brottfallsferill titils og „klárað“ þáttar fóru í staðinn eftir lengd hennar, og töldu hana áætlaða.

## [2.1.0] - 2026-09-29 to 2026-09-30

Áhorfslisti

### Added
- **Þinn eigin áhorfslisti**: settu kvikmynd eða þáttaröð á hann af síðu hennar, úr Væntanlegt, úr Nýlega bætt við eða úr leitinni (Ctrl+Enter), og finndu hann undir **Áhorfslisti** á þínum eigin prófíl.
- **Hver færsla segir hvar hún stendur núna**: á þjóninum, „5 af 26 þáttum“, beðið um, „Á næstunni á föstudag“, ekki á þjóninum, horfin úr safninu, eða horft. Það er reiknað í hvert sinn sem þú lítur á listann, með sama skilningi á „séð“ og framvinda þáttaraðanna þinna. Titlar sem horft hefur verið á haldast, í sérstökum hópi, þar til þú tekur þá af.
- **Titill sem er ekki enn á þjóninum þínum** getur farið beint á listann úr því sem Sonarr eða Radarr bíða eftir. Færslan tengist titlinum sjálfkrafa þegar hann kemur, og endurnefnd skrá tekur færsluna sína með sér.
- **„Á áhorfslistanum þínum: nú í boði“**: tilkynning til þinna eigin viðtakenda þegar eitthvað sem þú beiðst eftir kemur. Slökkt þar til þú hakar við.
- **Enginn annar sér áhorfslistann þinn**, ekki heldur stjórnandi, og tilkynningin um hann fer aðeins til þinna viðtakenda. Ekkert er skrifað í Jellyfin, Seerr, Sonarr eða Radarr.
- Áhorfslistar eru hluti af öryggisafriti, og endurheimt sameinar þá án þess að tvöfalda neitt.

### Changed
- Færslur í `/api/upcoming` bera TVDB- og TMDB-auðkenni titilsins síns, og `/api/auth/me` segir hvort þjónninn heldur áhorfslista.

## [2.0.4] - 2026-09-29

### Added
- **Tímasetning verkefna, eins og Jellyfin gerir það**: hvert verk sem FinStats vinnur sjálft hefur kveikjur (daglega á tilteknum tíma, vikulega á tilteknum degi, með föstu bili, við ræsingu, eða á eftir safnskönnun Jellyfin), hver með valfrjálsum tímamörkum. Smelltu á verkefni til að sjá og breyta áætlun þess; *Bæta við kveikju* opnar sama svarglugga og Jellyfin hefur.
- **Verkefni eru sérstakur hluti** í Stillingum, með síðustu keyrslu hvers verks í orðum („Keyrði síðast fyrir 38 mínútum, tók 6 mínútur“) og Keyra-hnapp; þau voru áður efst í Kerfi.
- **Lýsigagnabreytingar**: á hverri klukkustund les FinStats það sem Jellyfin hefur vistað frá síðustu athugun (nöfn, yfirlit, tegundir, einkunnir, leikara og aðstandendur, upplýsingar um skrár og myndir), svo breyting í Jellyfin birtist án þess að bíða þurfi eftir safnskönnun.
- Öryggisafrit og staðsetningargagnagrunninn má líka keyra úr Verkefnum; Esc fer til baka úr áætlun verkefnis á listann.

### Changed
- **Þemað er rofi með þremur stöðum**, Ljóst · Tæki · Dökkt, þar sem hnappurinn sem fór í hring milli þeirra var áður: smelltu á þriðjung hans, dragðu hnúðinn eða notaðu örvatakkana. Hnúðurinn ber tákn valsins, sem hreyfist inn þegar það breytist.
- **Útskráning er undir Stillingar → Aðgangur**, sem segir hver er skráður inn hér; hliðarstikan heldur þemarofanum.
- **Gömlu tímasetningarstillingarnar eru nú kveikjur**: *Fylgja safnskönnun Jellyfin* með varabilinu sínu, *Taka afrit á N daga fresti* og GeoIP-rofinn *Halda gagnagrunninum uppfærðum* urðu að samsvarandi kveikjum á verkefnum sínum, með gildunum sem þú hafðir. Söfnun, Öryggisafrit og Öryggi vísa á áætlunina í staðinn.
- Ræsing keyrir ekki lengur hvert verk aftur: hvenær hverju þeirra lauk síðast er geymt.
- Verkefni í Jellyfin sem hefur ekkert mat enn sýnir „um … eftir“, þar sem punktarnir fara í hring þar sem tíminn kemur, í stað „ETA…“.

### Performance
- **Kortið í Öryggi hikstar ekki lengur**: þegar dregið er og þysjað hreyfist kortið sem þegar er teiknað og er teiknað aftur einu sinni þegar hreyfingunni lýkur, og púlsinn fyrir það sem er í spilun lætur ekki lengur allt kortið endurmálast 120 sinnum á sekúndu meðan enginn snertir það. Drag fór úr 34 í 60 ramma á sekúndu á hægri tölvu.

### Fixed
- **Veggspjald eða andlitsmynd sem er skipt út í Jellyfin birtist í FinStats**: myndir voru geymdar í skyndiminni í viku á þjóninum og aðra viku í vafranum, hvað sem Jellyfin hafði síðan. Nú eru þær geymdar undir myndamerki Jellyfin, svo ný mynd er ný skrá, og vafrinn kannar hvort afritið hans sé nýjasta útgáfan í hvert sinn sem hann sýnir mynd. Þegar andlitsmynd er skipt út er aðeins manneskjan vistuð aftur, svo Lýsigagnabreytingar lesa líka fólkið.
- **Þjónn → Verkefni sýnir mat um leið og síðan opnast**: tíminn sem var eftir var aðeins mældur út frá mælingum sem síðan sjálf gerði meðan hún var opin, svo hver heimsókn byrjaði á „ETA…“ þar til hún hafði séð hlutfallið hreyfast. FinStats tímamælir nú sjálft verkefni sem eru í gangi í Jellyfin, á 10 sekúndna fresti meðan eitthvað keyrir, og síðunni er afhent matið. Verkefni sem Jellyfin segir standa í 0% allan tímann (fyrsti fasi safnskönnunar) heldur punktunum sínum þar til það hreyfist: ekki er giskað á neitt.
- **Sveimtextar á kortinu birtast á punktinum**: í Firefox gat sveimtexti punktsins fyrir það sem er í spilun lent langt í burtu, og þar sem punktar skarast (einhver að spila heima) réð einn díll því hvaða sveimtexti birtist. Einn sveimtexti nefnir nú allt sem er undir bendlinum.
- **Stöðustikan hefur eina sveimlögun**: hver tengill (streymin, Kóði, útgáfan) lýsist upp sem sami litli kassinn, og punktarnir milli atriðanna sitja fyrir miðju bilanna. Þegar bendlinum var haldið yfir útgáfunni lýstust punkturinn og bilið á undan henni líka upp.
- **Skráarslóð tekur alla breidd spjaldsins síns**: á Skrá-spjaldi titils, og fyrir möppur Jellyfin á síðunni Þjónn, var slóð kreist inn í einn mjóan reit og rann niður níu línur við hlið auðs rýmis.
- Svargluggi sem er að dofna út gleypir ekki lengur smell á síðuna undir honum.

## [2.0.3] - 2026-09-28

### Security
- Síðan Þjónn gaf auðkenni hvers tækis hverjum þeim sem hafði *Sjá þjóninn*; auðkenni krefjast líka *Sjá netupplýsingar*, eins og alls staðar annars staðar.
- Ef tveimur heimildarrofum var snúið áður en sá fyrri hafði vistast var sami upphafslistinn sendur tvisvar, og sá seinni afturkallaði þann fyrri: heimild sem var tekin af hélst veitt, og hluti opinbers prófíls sem slökkt var á hélst birtur, meðan báðir rofarnir sýndu það sem var valið.

### Stability
- **Aðeins einn innflutningur eða endurheimt í einu**: tvær upphleðslur sem voru sendar saman stóðust báðar athugunina, og endurheimt leit aldrei á innflutninga; sú seinni beið eftir gagnagrunninum þar til tíminn rann út og mistókst eftir margra mínútna upphleðslu.
- **Vistfang með brotinni `%`-kóðun opnar síðuna sem fannst ekki**, í stað þess að skilja eftir hleðsluhjól að eilífu.
- **Hátt blaðsíðunúmer í lista gefur tóma síðu**: það flæddi yfir og kom aftur sem fyrsta síðan.

### Fixed
- **Töflur raða tölum rétt í þýskum eða norskum vafra**: „1,204“ (eitt þúsund tvö hundruð og fjórir) var lesið sem tugabrot, og allt yfir milljón sem texti.
- **Töfluyfirlit grafs raðar dagsetningum, vikudögum og stöðum eftir tíma**, ekki eftir stöfum þeirra.
- **Niðurhalsferillinn nefnir hvaðan innfluttar skrár komu**: Sonarr og Radarr nefna leitarþjóninn (indexer) og samskiptaregluna aðeins þegar útgáfan er gripin, svo þær sundurliðanir voru alltaf tómar. Misheppnaður seríupakki er ein villa, ekki ein fyrir hvern þátt.
- **Þegar spilun sem horft var á saman er eytt** finnast hópar þess titils aftur, í stað þess að skilja eftir „hóp“ eins manns.
- **Tími saman, fyrir eina manneskju, er aldrei lengri en hún staldraði við**: síðan Saman eignaði henni allt kvöldið.
- **Kvöld í félagsskap sem nær yfir miðnætti á gamlárskvöld** (eða yfir mörk „síðustu N daga“) tilheyrir allt tímabilinu sem það byrjaði í, í stað þess að verða að tveimur kvöldum ein.
- **Innfluttar spilanir á þáttum eru nefndar eftir nafni þáttarins** þar sem Jellystat geymdi aðeins nafn þáttaraðarinnar.
- **Staðsetning finnur borgagagnagrunninn við hlið Country- og ASN-gagnagrunnanna** sem `geoipupdate` skrifar, í stað þess að slökkva á sér.
- **Heitasti bletturinn í Oftast spólað til baka fylgir síum síðunnar**, eins og talningar hennar.
- **„Árið þitt er tilbúið“** fer aðeins til þeirra sem eiga eitthvað í uppgjörinu sínu.
- **Þegar þögguð viðvörun er leyst aftur helst parið hennar þaggað**; aðeins enduropnun afturkallar þöggun.
- **Útrunninn API-lykill telst ekki lengur með í tuttugu lykla hámarkinu.**
- **Lotuhámarkið sleppir elstu lotunni** jafnvel þegar margar innskráningar berast á sömu sekúndu.
- **Heimilisvistfang sem uppflettingin fann helst** þegar það sama er tekið af handskrifaða listanum.
- **Myndir í endurskoðunarskránni og lyklalistanum**, þar sem aðeins voru upphafsstafir.
- **Tilkynningalisti stjórnanda segir hver á persónulegan viðtakanda**, í stað „Þinn“ á öllum; **Nýlega sent** sýnir alla viðtakendur síðasta atburðar.
- **Esc á titli eða manneskju fer aldrei út úr FinStats**, og lokar aðeins efsta svarglugganum.
- **Síða er teiknuð jafnvel þegar vistun stöðvar forsóknina sem hún beið eftir.**
- **Í spilun núna sýnir áhorfanda í hléi sem spólar** þar sem hann er núna, ekki þar sem hann ýtir á spila.
- **Þegar tenging er fjarlægð uppfærist hvaða síður eru til**, eins og þegar tenging er vistuð.
- **Útgáfudagsetning staðsetningargagnagrunnsins heldur ártalinu sínu** í bandarískri ensku.

## [2.0.2] - 2026-09-28

### Security
- Á bak við milliþjón taldi innskráningarhámarkið fyrsta `X-Forwarded-For`-vistfangið, sem er það sem biðlarinn sendir: nýtt uppdiktað vistfang í hverri tilraun, og hámarkið gilti aldrei. Nú er það það síðasta, það sem milliþjónninn skrifaði.
- *Taka með IP-vistföng og staði* sendi vistföng til persónulegs viðtakanda þar sem eigandann vantar *Sjá netupplýsingar*, svo engin síða myndi sýna honum þau. Misheppnaðar innskráningar annarra krefjast nú líka *Sjá þjóninn*, eins og á síðunni Öryggi.

### Stability
- **Stöðvun glatast aldrei**: þegar lestri safns var hafnað á fyrstu augnablikunum eftir ræsingu, áður en þjónninn hlustaði eftir henni, var höfnunin skráð og FinStats hélt áfram í stað þess að stöðvast.
- **Jellyfin sem svarar á formi sem FinStats getur ekki lesið er hafnað**, ekki trúað: hver notandi, eða hver titill safns, sem barst undir lyklum sem FinStats þekkir ekki (`id` í stað `Id`) komst í gegnum vörnina með fullri talningu og var svo merktur fjarlægður.
- **Lok spilunar falla aldrei niður**: þegar gagnagrunnurinn var upptekinn lengur en biðtími hans (innflutningur hélt honum), stóð spilun sem lauk á því augnabliki merkt í spilun fram að næstu endurræsingu. Nú er hún skrifuð í næstu umferð.
- **Stillingar sem endurheimt getur ekki lesið eru látnar vera**: öryggisafrit með stillingum sem pössuðu ekki lengur við þessa útgáfu endurstillti hverja stillingu við endurheimt: samrunabil, heimildir, heimilisvistföng, áætlun.
- **Stýristafur í nafni eða titli brýtur ekki lengur spjöldin**: prófílspjaldið, hvert söguspjald sem sýndi þann titil og ZIP-skráin með þeim svöruðu með villu.

### Fixed
- **Tilkynningar hætta fyrir þá sem eru gerðir óvirkir eða eytt í Jellyfin**: persónulegir viðtakendur þeirra fengu áfram allt sem gömlu heimildirnar þeirra leyfðu.
- **Tilkynning í biðröð er athuguð aftur áður en hún er send**: viðtakandi sem slökkt hafði verið á, eða eigandi sem hafði misst heimild, fékk samt það sem beið eftir að vera reynt aftur.
- **Viðtakanda sem svarar alltaf `Retry-After` er gefist upp á** eftir fimm tilraunir eins og öðrum, í stað þess að reyna aftur í þrjátíu daga.
- **Auðkenni tækja á síðu manneskju** eru aðeins sýnd með *Sjá netupplýsingar*, eins og alls staðar annars staðar.
- **Birt ár bíður sólarhring með allt**: kláraðar og hættar þáttaraðir þess fylgdu hverri spilun, svo ef ýtt var á spila á þáttaröð sem hafði verið hætt við breyttist opinbera síðan innan nokkurra sekúndna.
- **Þáttaröð sem beðið var um telst áhorfð** þegar einn þáttur hennar er það: hver þáttaröð sem beðið var um var lesin sem aldrei áhorfð, í kaflanum Beiðnir og á spjaldinu Beðið um.
- **Kaflinn Saman í uppgjörinu sleppir sjónvarpi í beinni**, eins og restin af árinu: sjónvarpsrás sem var látin ganga meðan horft var saman gat orðið efsti titillinn, og hlutfallið gat farið yfir 100%.
- **Dagaspjaldið fyrir „síðustu 12 mánuði“ endar á nýjasta deginum**: hnitanetið sleppti nýjustu þremur eða fjórum vikunum.
- **Nýir þættir þáttaraðar eru tilkynntir einu sinni á kvöldi**: þættir sitt hvorum megin við miðnætti UTC voru tilkynntir tvisvar á þjóni austan eða vestan við það.
- **Þegar Seerr er tengt eru gamlar beiðnir ekki tilkynntar tilbúnar til áhorfs**: beiðni sem var þegar tiltæk við fyrsta lestur, án annarrar skráningar um hvenær, var lesin sem nýkomin á þeirri stundu.
- **Titlar safns sem er eytt í Jellyfin fara með því**: þeir sátu áfram í leit, á hillunni Nýlega bætt við og í heildartölum.
- **Remux helst beint streymi** þegar mæling missir umkóðunarupplýsingar sínar um stund (við spólun), í stað þess að verða umkóðun það sem eftir er spilunarinnar.
- **„Jellyfin liggur niðri“ þýðir að lestrar mistókust í röð**: stöku tímamörk með daga millibili, á þjóni sem var kyrr á WebSocket-tengingunni, lögðust saman í það.
- **Innflutningur úr Streamystats sýnir framvindu sína á eigin spjaldi**, ekki yfir síðustu niðurstöðu Jellystat-innflutningsins.
- **Vistfang þar sem hýsillinn byrjar á „web“** (`http://web:8096`) er lesið sem hýsill, ekki stytt eins og slóð vefbiðlara Jellyfin.
- **Villa í biðröð afmáir allar innskráningarupplýsingar** vistfangs jafnvel þegar lykilorðið inniheldur skástrik.

## [2.0.1] - 2026-09-28

### Changed
- **Tungumál titils eru ekki fyrir**: hausinn nefnir fyrstu þrjú tungumál hljóðs og skjátexta, eina línu fyrir hvort, og nýtt spjald, **Tungumál**, telur upp hvert þeirra: þau fullkomnu saman, þau ófullkomnu með því hve margir þættir hafa þau. Tveir tugir skjátextatungumála í langri þáttaröð ýttu áður myndefninu niður um heilan skjá.

### Fixed
- **Veggspjald titils helst við hlið nafnsins**: þegar textinn við hliðina var lengri en veggspjaldið (langur söguþráður, mörg kvikmyndaver) sökk veggspjaldið neðst í hausinn, undir auðu myndefni.
- **Útgáfa sem FinStats getur ekki lesið er kölluð það**: `app_version` sem var breytt handvirkt í eitthvað ólæsilegt lét FinStats neita að ræsast sem „eldra FinStats 2.0.1“, sem það var ekki. Það neitar enn, og segir nú hvaða gildi það gat ekki lesið og hvernig á að skrifa það.

## [2.0.0] - 2026-09-25 to 2026-09-28

Uppgjör 2026, opinberir prófílar og samáhorf

### Added
- **Með hverjum þú horfðir**, í uppgjörinu: kvöld í félagsskap, tíminn sem fór í það, titillinn sem leiddi fólk oftast saman, og allt að þrír félagar, nafngreindir í forritinu og með tengli á prófíla sína.
- **Þáttaraðir séðar til enda**, í uppgjörinu: síðasti þáttur hverrar sem var séður það ár, með sama skilningi á „séð“ og framvindustikur prófílsins, og líka þær sem var byrjað á og lagðar til hliðar, minna en hálfséðar og ekkert spilað í tvo mánuði.
- **Það sem þú baðst um**, í uppgjörinu: Seerr-beiðnir sem gerðar voru á árinu, hve margar komu og á hve margar þú horfðir svo. Aðeins þar sem beiðnir eru til.
- **Árið borið saman við árið á undan**: klukkustundir, spilanir og dagar, og hve miklu meira eða minna.
- **Árið sem saga**: hver kafli sem 1080×1920 spjald, lögunin sem sögur í símum nota (árið, tölurnar, efstu þáttaraðir, kvikmyndir og tónlist, tegundir, áhorfspersónuleikinn, klukkustundir og dagar, hver dagur ársins, met, félagsskapur, kláraðar þáttaraðir, beiðnir og árið á undan). Vistaðu eitt, eða öll sem ZIP-skrá, úr „Deila árinu sem spjöldum“.
- **Spjald nefnir aldrei neinn annan**: spjöld eru teiknuð úr afriti af árinu sem hefur engan reit fyrir aðra manneskju, sæti eða forrit. Í forritinu ber spjald alls ekkert nafn.
- **Ár þjónsins**, fyrir stjórnendur Jellyfin: hver titill og heildartala og áhorfspersónuleiki heimilisins, þar sem enginn fær sæti eða er nafngreindur. Rofi á síðu uppgjörsins, `scope=server` í API-inu.
- **„Árið þitt í hnotskurn er tilbúið“**: tilkynning í desember, einu sinni til hverrar manneskju sem horfði það ár.
- **Opinberir prófílar**: manneskja getur birt hluta af prófílnum sínum á tengli sem hver sem er getur opnað án aðgangs (heildartölur og efstu titlar, hrinur og hvenær hún horfir, allt árið sem saga í spjöldum, síðustu tíu spilanir), og slökkt er á hverjum hluta þar til eigandinn kveikir á honum undir Stillingar → Opinber prófíll. Tæki, vistföng, forrit, spilunaraðferðir, skráarslóðir og annað fólk eru aldrei birt, hvað sem kveikt er á, og nafnið sem sést er nafn sem eigandinn slær inn, aldrei innskráningarnafn hans í Jellyfin.
- **Ekkert á birtri síðu er nýrra en sólarhrings gamalt**, nýlegar spilanir og allar heildartölur jafnt, svo síða sem einhver fylgist með segir aldrei hver er að horfa akkúrat núna.
- **Spjöld til að deila**: hverjum birtum prófíl fylgir 1200×630 mynd, af prófílnum og af árinu, sem Discord, Signal, Mastodon og hin sýna þegar tengillinn er límdur inn, og sem má sækja til að birta. Teiknuð á þjóninum úr því sem síðan sýnir og engu öðru.
- **Tengill sem þú getur tekið til baka**: 22 handahófskenndir stafir, endurstilltir með einum smelli; gamli tengillinn svarar þá nákvæmlega eins og tengill sem aldrei var til.
- **Slökkt er á opinberum prófílum þar til stjórnandi leyfir þá**: einn rofi fyrir allan þjóninn, listi yfir hver birtir hvað, og leið til að taka prófíl niður. Birtingar, breytingar, endurstillingar og niðurtökur eru í endurskoðunarskránni, og lykill getur ekki gert neitt opinbert.
- **Saman**, sérstök síða: hver horfir með hverjum, hve mikið, og hvernig það hefur breyst. Tími saman, kvöld og hlutfall áhorfstímans í félagsskap, hvert borið saman við tímabilið á undan; klukkustundir í félagsskap á móti klukkustundum einn á dag eða viku; hvert par með kvöldum sínum, tíma saman, hvenær þau horfðu síðast og á hvað þau horfa mest; titlarnir sem mest er horft á saman; nýleg kvöld; og, fyrir hvern sem má sjá alla, tími hverrar manneskju ein á móti tíma hennar í félagsskap.
- Hvert par telst: kvöld þriggja telst fyrir hvert af þremur pörum sínum, þar sem spjaldið taldi áður aðeins nákvæmlega sama hópinn. Tími pars saman er styttri dvölin af þeim tveimur.
- Spjaldið **Horft saman** á yfirlitinu og á prófíl vísar á síðuna, og spjald prófíls segir hve stór hluti áhorfstíma viðkomandi var í félagsskap.
- `GET /api/stats/groups` fær `pairs`, `series`, `people`, `previous`, `bucket`, og `watch_s` og `share` í heildartölunum; lyklarnir sem það hafði eru óbreyttir.
- **Hvar fólk hættir**, á hverri síðu kvikmyndar og þáttar: hlutfall spilana sem eru enn í gangi á hverri mínútu, frá öllum í byrjun til þeirra sem náðu til enda, með spólunum til baka og kveikingum á skjátextum í þeim spilunum merktum á sama ás. Spjaldið segir hve mörg stopp voru mæld (spilanir sem FinStats skráði) og hve mörg áætluð (innfluttar spilanir, út frá því hve lengi þær stóðu), og þarf þrjár spilanir áður en það teiknar nokkuð.
- **Hver heldur áfram að horfa**, á síðu þáttaraðar: hve margir byrjuðu á hverjum þætti, í röð, og dálkurinn **Klárað** fyrir hvern þátt. „Allir hætta í þætti þrjú“ er fólk sem ýtir aldrei á spila á þætti fjögur.
- **Skrár sem spilast aldrei**, á Spilun: kvikmynd eða þáttur sem byrjað var á þrisvar eða oftar en komst aldrei lengra en þrjátíu sekúndur, með því hver reyndi og í hvaða forritum. Atvikaskrá Jellyfin geymir engar spilunarvillur, svo spilanirnar eru vitnið; lágmarkslengd spilunar gildir ekki hér, því hún er einmitt það sem þessar spilanir ná aldrei.
- **Oftast spólað til baka** og **Kveikt á skjátextum**, á Spilun: titlum raðað eftir spólunum aftur á bak á hverja spilun ásamt mínútunni sem þær safnast um, og eftir hlutfalli spilana þar sem kveikt var á skjátextum innan fyrstu tíu mínútnanna. Allir þrír listarnir krefjast *Sjá virkni allra*. Stökk styttri en 20 sekúndur voru aldrei skráð, svo stutt endurspilun á einni setningu sést ekki, og það er viljandi.
- Spólun geymir hvaðan hún kom sem tölu við hlið merkisins síns (`from_s` í tímalínu spilunar). Hver spólun sem þegar var skráð fær tölu sína úr merkinu, og það gerir endurheimt öryggisafrit frá því fyrir þessa útgáfu líka.
- `GET /api/stats/files` fyrir listana þrjá; `GET /api/items/{id}` fær `insights`, og þættir þáttaraðar fá `users` og `finished`.
- **API-lyklar**: hver sem er skráður inn getur búið til lykla fyrir sig undir Stillingar → API-lyklar, fyrir forskrift, mælaborð eða síma. Lykill er sendur sem `Authorization: Bearer fs_…`, er sýndur einu sinni, geymdur sem tætigildi, getur haft gildistíma, og er afturkallaður með einum smelli; stjórnandi sér lykla allra og getur afturkallað þá. Lykill vísar á sömu manneskju með sömu heimildir og innskráning, lesnar beint við hverja beiðni, svo þegar réttur til að skrá sig inn tapast hætta allir lyklar að virka í einu. Lykill getur hvorki búið til né afturkallað lykla.
- **Dagatalsstraumur**: `GET /api/calendar.ics?key=…` er það sem Sonarr og Radarr eiga von á, sem iCalendar sem sími getur gerst áskrifandi að. Lykill með umfanginu *dagatal* opnar strauminn og ekkert annað; straumurinn les lykilinn úr vistfanginu því dagatal í áskrift getur ekki sent haus, aldrei úr vafrakökunni, og nefnir engan.
- **Endurskoðunarskrá** undir Þjónn → Endurskoðun, fyrir stjórnendur Jellyfin: hver innskráning og misheppnuð tilraun með vistfangi sínu, hver stilling eða heimild sem var breytt og í hvað, hver lykill sem var búinn til, notaður eða afturkallaður, hver tenging, viðtakandi, öryggisafrit og innflutningur, hver spilun sem var eytt og viðvörun sem var leyst, með því hver, hvaðan, með hvaða lykli og hvort það tókst. Geymd í ár, hluti af öryggisafritum. `GET /api/audit` flettir í gegnum hana.
- **Innflutningur úr Streamystats**, sérstakur hluti undir Stillingum við hlið Jellystat. Taktu skrána úr **Settings → Backup & Import → Download Backup** þar og slepptu henni hér: ferill margra ára berst á nokkrum sekúndum, og ekkert sem þú hefur þegar er talið aftur. `finstats import-streamystats <file>` gerir það án viðmóts, `POST /api/import/streamystats` í gegnum API-ið; einn innflutningur keyrir í einu, af hvorri tegundinni sem er.
- **Notaðir þú bæði tölfræðiforritin? Flyttu inn báðar skrárnar.** FinStats þekkir spilun sem það hefur þegar á auðkenni tölfræðiforritsins fyrir hana og, ef það dugar ekki, á því að sama manneskja horfi á sama atriði og annar hvor endi spilunarinnar sé nálægt enda spilunar sem er þegar í ferlinum, svo Jellystat-útflutningur og Streamystats-útflutningur sömu kvölda geta báðir verið fluttir inn, í hvaða röð sem er, og kvöld sem FinStats fylgdist sjálft með er ekki flutt inn ofan á það úr hvorugu. „Nálægt“ er samrunabilið (*Endurræsing telst sama spilun ef bilið er undir*) undir Stillingar → Söfnun.
- Innfluttar línur segja hvað þær geta ekki sagt. Spilun sem Streamystats fylgdist sjálft með geymir ekkert um skrána (enga kóðun, upplausn, bitahraða eða ílát), svo þeir reitir haldast tómir frekar en að vera fylltir með ágiskun, og spjöldin sem myndu sýna þá fela sig. Spilun sem það hafði tekið frá Jellystat ber alla lotuna, og hún er lesin í staðinn: dagsett eftir augnablikinu þegar henni lauk, eina augnablikinu sem hvort forritið um sig geymdi, og lýst sem umkóðun aðeins þegar lotan var það í raun.
- Línur sem Streamystats skrifaði vegna þess að Jellyfin sagði atriði áhorft, fyrir áhorf sem það sá aldrei, eru taldar og þeim sleppt: þær eru jafnlangar allri kvikmyndinni og yrðu áhorfstími á kvöldi sem enginn horfði. Niðurstaðan segir hve margar þær voru.
- Hvernig gögn úr Streamystats eru túlkuð, reit fyrir reit, lært af raunverulegum útflutningi frekar en af skjölun: `docs/streamystats-import.md`.
- **Úr hvaða tölfræðiforriti hver spilun kom**, sem sía á Virkni: það sem FinStats skráði sjálft, það sem kom frá Jellystat, það sem kom frá Streamystats. Aðeins þau sem ferillinn þinn geymir í raun eru í boði, og sían er afmörkuð eins og hver annar listi, svo sá sem má aðeins sjá eigin spilanir fær aðeins að vita hvaðan hans eigin ferill kom.
- **Síurnar á Virkni eru fellilistar sem þú hakar við**, svo þú getur beðið um kvikmyndir *og* þætti án tónlistar, tvær spilunaraðferðir, eða tvær manneskjur í einu, í stað eins af hverju eða allra. Tegund efnis fékk **Annað** nú þegar hægt er að sameina.
- Nokkrar manneskjur í einu, hvar sem notendasían birtist (yfirlitið, spilun, söfn, öryggi og hitt), og kortið og öll tölfræði fylgja. Notendasía þrengir samt alltaf aðeins það sem einhver má sjá: án *Sjá virkni allra* er beiðni fest við þann sem biður, hvern sem vistfangið nefnir og hversu marga.
- **Ljóst þema**: washi-pappír, með bleksvörtum texta og sinnóberrauðum áherslulit. FinStats fylgir ljósri eða dökkri stillingu tækisins þar til þú velur.
- **Þemahnappur við hliðina á Skrá út** fer í hring milli Tæki, Ljóst og Dökkt. Valið er geymt í þessum vafra og því beitt áður en síðan er teiknuð, svo hitt þemað blikkar aldrei.
- **Opna í Jellyfin**: hver síða kvikmyndar, þáttaraðar og þáttar hefur hnapp sem opnar síðu titilsins í Jellyfin, í nýjum flipa.
- **Vistfang Jellyfin fyrir fólk** undir Stillingar → Jellyfin: hvert þessir hnappar vísa, ef vistfangið sem FinStats tengist er ekki vistfang sem vafri nær til. Autt heldur vistfanginu sem FinStats tengist; aðeins stjórnandi í Jellyfin getur stillt það.

### Changed
- **FinStats á nýtt heimili**: `github.com/finstats/finstats`, og Docker-myndin er `ghcr.io/finstats/finstats`. Breyttu nafni myndarinnar í `docker run` eða compose-skránni þinni til að halda áfram að fá uppfærslur; gagnamappan helst eins og hún er.
- **Stillingar eru einn hluti á skjánum í einu.** Síðan var fjórtán spjöld í einum dálki og sex hundruð orð af hjálpartexta. Nú hefur hver hluti sitt eigið vistfang (`/settings/collection`), listi til vinstri færir á milli þeirra, og í síma verður listinn að röð af hnöppum. Tenglar á gömlu akkerin lenda enn á réttum stað.
- **Stilling er ein lína**: hvað hún er, ein lína sem segir regluna, stýringin til hægri. Tölur í hluta vistast saman með einum hnappi neðst; rofar vista sig enn sjálfir. Stöðuspjöldin (Verkefni, Tengingar út á við, Gagnagrunnur) og tengillinn á leyfin eru undir **Kerfi**, og innflutningsforritin tvö hafa sinn eigin hluta.
- **Finndu stillingu** úr reitnum í hausnum (`/` setur fókus á hann) með hvaða orði sem er í nafni hennar; hann segir í hvaða hluta stillingin er, og Enter lendir á línunni.
- **Stillingar eru fyrir alla sem eru skráðir inn**: hlutarnir sem stjórna þjóninum eru áfram hjá þeim sem mega stjórna honum, og manneskja sem mátti aðeins fá tilkynningar kemst loksins að eigin viðtakendum.
- **Síðan Þjónn er einn hluti á skjánum í einu**, eins og Stillingar: Yfirlit (kerfi og geymslurými), Verkefni, Tæki, Viðbætur, Atvikaskrá og Endurskoðun, hver á eigin vistfangi. Tenglar á `/server#jobs` lenda enn á réttum stað.
- **Atvikaskrá þjónsins er hluti af síðunni Þjónn** (`/server/log`) frekar en sérstök síða í valmyndinni; `/events` vísar þangað með síunum sínum.
- Tilkynning um misheppnað verkefni, misheppnað öryggisafrit eða tengingu sem hætti að svara vísar á þann hluta stillinganna sem geymir það, frekar en efst á síðuna.
- Það sem safnið þitt getur sagt um innflutta spilun (hvers konar efni það var, hvaða þáttur, lengd, hvaða safni það tilheyrir) er fyllt út eftir **hvern** lestur safns, ekki aðeins á augnabliki innflutnings. Ferill er yfirleitt fluttur inn áður en FinStats hefur nokkurn tíma lesið safnið, svo þær spilanir héldust áður ágiskaðar eða óþekktar fyrir fullt og allt.
- Endurheimt öryggisafrits FinStats bætir ekki lengur við spilun sem barst frá tölfræðiforriti undir öðru auðkenni, og heldur samt tveimur raunverulegum áhorfum á sama efni með mínútna millibili.
- Upplýsingar spilunar nefna það tölfræðiforrit sem kom með hana, frekar en að kalla allt innflutt Jellystat-spilun.

### Performance
- **Endurræsing er nánast samstundis, á hvaða ferli sem er.** Hver ræsing reiknaði út hver horfði saman yfir allan ferilinn og ákvað aftur hvaða spilanir voru á heimaneti, hvort tveggja án þess að breyta neinu: 24 s og 5,5 s á tíu milljón spilunum. Ræsing gerir nú aðeins það sem síðasta keyrsla skildi eftir, og allt aftur aðeins eftir uppfærslu eða þegar stillingin að baki breyttist.
- **Fjöldi sem kemur í einu er skráður á sekúndum.** Hver ný spilun, hver vistun framvindu og hvert tæki var skrifað sér, og hverri spilun sem lauk var lokað með eigin athugun á öllum ferli titilsins; umferð skrifar nú hverja tegund í einu lagi. 100.000 spilanir sem byrja saman taka 3 s, þar sem 25.000 sem lauk saman tóku áður yfir fjórar mínútur.
- **Í spilun núna helst fljótt með þúsundir að horfa.** Til að finna hver horfir saman var hvert streymi borið saman við hvert annað og hver félagi talinn upp með nafni: fjörutíu sekúndur við 10.000 streymi, og gígabæti af minni þar fyrir ofan. Streymi eru nú borin saman innan titils, og hvert sýnir nokkur nöfn og hve margir aðrir eru.
- Útreikningurinn á því hver horfði saman endurskrifaði hverja hópaða spilun ferilsins í hvert sinn sem FinStats ræsti, flutti inn eða sá spilun ljúka; nú skrifar hann aðeins það sem breyttist. Leitin að spilunum sem hafði lokið hægist ekki lengur í hlutfalli við fjölda spilana í gangi í öðru veldi.
- **Leitin að spilun sem ferillinn hefur þegar fer í gegnum vísi**, svo innflutningur 150.000 spilana tekur sekúndur, og endurtenging eftir lestur safns fer titil fyrir titil í stað þess að lesa hverja spilun meðan gagnagrunninum er haldið.
- **Innsýn í spilun og yfirlitið eru fljótari yfir allan tímann**: óháðir hlutar þeirra eru reiknaðir hlið við hlið, samtímagrafið flettir upp staðardeginum einu sinni á hvern fjórðung klukkustundar í stað einu sinni á hverja spilun, og tegundir eru taldar á hvern titil. Á milljón spilunum: 4,1 s í 2,4 s, og 1,6 s í 1,1 s.

### Stability
- **Sjónvarpssafn lítur ekki lengur út eins og safn sem hefur verið tæmt.** Lestur safns fer tvær umferðir (hvert atriði, síðan minni umferð fyrir leikara og aðstandendur kvikmynda og þáttaraða eingöngu), og athugunin sem ákveður hvort treysta megi lestri til að fjarlægja það sem hann sá ekki leit á þá seinni. Í sjónvarps- eða tónlistarsafni eru talningarnar tvær hvergi nærri hvor annarri, svo athugunin hafnaði hverjum lestri og stöðvaði FinStats, og uppsetningin endurræstist og gerði það sama aftur. Ekkert var nokkurn tíma ranglega fjarlægt, en enginn lestur safns gat klárast. Ef safnið þitt fyrir þáttaraðir eða tónlist hefur ekki tekið upp nýja titla, er þetta ástæðan.
- Innflutningur gat mistekist með „database is locked“ þegar önnur skrif (spilun sem verið var að uppfæra) lentu milli fyrsta lestrar hans og fyrstu skrifa. Bæði innflutningsforritin taka skriflásinn þegar þau byrja, og dagbók gagnagrunnsins helst ekki lengur jafnstór og stærsti innflutningur sem gerður hefur verið.
- **Gagnagrunnsskrá sem er skemmd, stýfð, í eigu annars forrits eða aðeins lesanleg er hafnað strax**, með nafni skrárinnar og þeim orðum að engu hafi verið breytt, í stað 30 hljóðra sekúndna; aldrei er skrifað í gagnagrunn annars forrits.
- Öryggisafrit eða skyndimynd fyrir uppfærslu sem er rofin hálfnuð skilur ekki lengur eftir skrá, skrá sem gat tekið sæti góðrar skyndimyndar meðal þeirra þriggja sem eru geymdar.
- **Ef staðsetningargagnagrunnurinn er yfirskrifaður meðan FinStats keyrir hrynur það ekki lengur**: FinStats les úr einkaafriti af skránni.
- Jellyfin-forrit sem tilkynnti nýtt auðkenni tækis í hvert sinn gat látið minnisnotkun FinStats vaxa svo lengi sem það keyrði; tækjum sem hafa ekki sést í fimm mínútur er nú gleymt.
- Ein manneskja heldur í mesta lagi 30 innskráðum lotum; sú elsta er skráð út við næstu innskráningu.

### Fixed
- **Sama kvöld er ekki lengur talið tvisvar þegar atriði hefur verið endurnefnt.** Endurtenging ferils við titil sem var bætt aftur við í Jellyfin gat gert spilun sem var flutt inn úr tölfræðiforriti að tvítaki spilunar sem var þegar til staðar, því athugunin sem hefði átt að grípa það keyrði áður en auðkennið færðist. Nú keyrir hún aftur hvar sem auðkenni eru endurtengd. Aðeins innfluttum spilunum er nokkurn tíma eytt, aldrei spilun sem FinStats skráði sjálft, og hve margar fóru stendur í annál FinStats. Fyrsta ræsing 2.0 fer einu sinni yfir allan ferilinn eftir þeim sem þegar eru til staðar.
- **Lota fylgir nú manneskjunni eins og Jellyfin hefur hana núna.** Stjórnandi sem Jellyfin lækkaði í tign hélt fullum aðgangi að FinStats það sem eftir var af 30 daga lotunni sinni, og sá sem var gerður óvirkur eða eytt í Jellyfin hélst innskráður. Nú missa þeir aðganginn um leið og FinStats les notendur Jellyfin (á 15 mínútna fresti, eða strax úr Stillingum), eins og API-lyklar hafa alltaf gert.
- Innskráning skrifar það sem Jellyfin segir um manneskjuna í notendalista FinStats, svo innflutt öryggisafrit úr tölfræðiforriti getur ekki lengur gert einhvern að stjórnanda áður en fyrsti lestur notenda leiðréttir það.
- Sá sem var eytt í Jellyfin er merktur fjarlægður jafnvel þegar tveir lestrar notenda lenda á sömu sekúndu.
- **Villur við niðurhal sýna aldrei lykilorð niðurhalsforrits eða API-lykil leitarþjóns**, sem Sonarr og Radarr geta vitnað í í þeim.
- Spilanir sem voru endurtengdar við endurnefndan titil meðan safn var lesið voru ekki athugaðar fyrir samáhorf fyrr en við næstu endurræsingu; það er nú gert strax.

## [1.6.5] - 2026-09-24

### Added
- **Fullt öryggisafrit er tekið sjálfkrafa fyrir hverja uppfærslu.** Í fyrsta skipti sem nýrra FinStats opnar gagnagrunninn þinn skrifar það heilt afrit af honum (allan ferilinn þinn, stillingar og allt) í `data/pre-update-backups/` *áður* en það breytir nokkru. Ef uppfærsla brýtur einhvern tíma eitthvað eru gögnin þín örugg: stöðvaðu FinStats, settu afritið í stað `finstats.db`, ræstu útgáfuna sem þú varst á, og tilkynntu villuna. Ekkert glatast. Nýjustu afritin eru geymd; settu `FINSTATS_SKIP_PREUPDATE_BACKUP=1` til að slökkva á þessu ef diskpláss er af skornum skammti.

## [1.6.4] - 2026-09-24

### Added
- **Fimm staðir í viðbót til að senda tilkynningar á**: Telegram, Slack, Pushover, Pushbullet og tölvupóstur, við hlið vefkróks, Discord, ntfy og Gotify sem voru fyrir. Hver biður aðeins um það sem hann þarf (spjallauðkenni, notandalykil, pósthólf), og þá þrjá sem búa hjá einni þjónustu (Telegram, Pushover, Pushbullet) er aðeins hægt að ná þar, svo aldrei er hægt að senda tóka á hýsil sem líkist þeim. Póstur fer út yfir TLS frá fyrsta bæti eða með `STARTTLS`, og þriðji kosturinn er ekki til.
- **Leyfi**, undir **Stillingum**: eigið GPL-leyfi FinStats og hvert leyfi þriðju aðila sem það er byggt á (253 Rust-pakkar, letrin tvö sem fylgja með, útlínur kortsins og staðsetningargagnagrunnurinn), hver texti í heild, eins og verkefnið sjálft skrifaði hann. Veldu hluta og leyfi hans opnast í glugganum við hlið listans; pakki sem kemur með nokkrar skrár skiptir á milli þeirra á staðnum. Listinn er búinn til úr pökkunum sem FinStats er í raun byggt úr, svo ekki er hægt að bæta við pakka sem FinStats er háð án þess að leyfi hans sé skráð.

## [1.6.3] - 2026-09-24

### Fixed
- Öryggisvörnin grípur nú líka inn í þegar Jellyfin telur upp *engin* söfn eða *enga* notendur yfirhöfuð (bilaður lestur, aldrei venjulegur dagur), sem stillingin í 1.6.2 hafði hleypt í gegn. Að missa einhver söfn er enn talið venjulegt; að missa þau öll í einu er það ekki.

## [1.6.2] - 2026-09-24

### Fixed
- Gagnaheilleikavörnin sem bættist við í 1.6.1 var of áköf: safn sem missti í raun flest atriði sín (handfylli myndskeiða sem skrárnar hurfu af, til dæmis) var tekið fyrir bilaðan lestur, svo FinStats neitaði að uppfæra það og stöðvaðist. Nú grípur hún aðeins inn í *greinilega* bilaðan lestur: safn sem er nógu stórt til að skipta máli og les aftur alveg tómt, eða stórt safn sem er skorið niður í nánast ekkert. Hversdagslegar breytingar gilda eins og áður, og tilvikið „Jellyfin skilaði engu“ er enn gripið.

## [1.6.1] - 2026-09-24

### Added
- **FinStats treystir ekki lengur lestri úr Jellyfin sem myndi þurrka út ferilinn þinn.** Ef safn sem FinStats geymir þúsundir atriða fyrir les skyndilega aftur tómt eða næstum tómt (Jellyfin-uppfærsla sem breytti API-inu sínu, villa sem lítur út eins og tóm niðurstaða), heldur FinStats gögnunum sem það hefur, merkir samstillinguna misheppnaða (og sendir tilkynningu), og stöðvast snyrtilega með skýrum skilaboðum frekar en að merkja allt fjarlægt. Endurræstu þegar Jellyfin er orðið eðlilegt aftur, eða settu `FINSTATS_ALLOW_LIBRARY_SHRINK=1` ef þú tæmdir safn í raun. Sama vörn nær yfir safnalistann og notendalistann.

### Security
- `X-Forwarded-Host`-haus sem biðlari sendi gat vottað fyrir framandi `Origin` og komið skrifum milli vefsvæða framhjá vörninni um sama uppruna. Þeim haus er nú aðeins treyst á bak við traustan milliþjón (`FINSTATS_TRUST_PROXY`), sama regla og uppfletting á vistfangi biðlarans notar þegar. `SameSite=Lax`-lotukakan var, og er áfram, fyrsta varnarlínan.
- Persónulegum viðtakanda tilkynninga mátti beina að `0.0.0.0` eða `::`, sem ná til þessarar vélar. Athugunin sem heldur persónulegum viðtakendum á opinberum vistföngum telur þau nú staðbundin, ásamt loopback-vistföngum og einkasviðunum.

### Fixed
- Síðan **Spilun** hleðst nú inn fyrir safn sem inniheldur spilun án skráðrar spilunaraðferðar (línu sem var endurheimt úr öryggisafriti sem er eldra en dálkurinn); slík spilun telst einfaldlega bein spilun.
- Mjög langt orð í leit eða síu veldur ekki lengur villu á þjóninum: SQLite hafnar `LIKE`-mynstri yfir 50.000 stöfum, svo síuorð eru stytt í hæfilega lengd fyrir fyrirspurnina.
- Síðan **Stillingar** flettist ekki lengur til hliðar í síma þegar spjald skilar heilli setningu, eins og niðurstöðu endurheimtar.

## [1.6.0] - 2026-09-24

Tilkynningar

### Added
- Spjaldið **Tilkynningar** undir Stillingum: bættu við viðtakanda, hakaðu við það sem hann á að frétta af, ýttu á Prófa. Enginn viðtakandi þýðir að engin beiðni er send neitt.
- Fjórar tegundir viðtakenda: einfaldur **vefkrókur** (JSON, skjalfest í `docs/api.md`, valfrjáls `Authorization`-haus), **Discord** (embed í gegnum vefkrók rásar), **ntfy** (ntfy.sh eða þinn eigin þjónn, á tiltekið efni) og **Gotify** (þinn eigin þjónn, með forritstóka). Titlar ferðast í skilaboðunum frekar en í haus.
- Ellefu tegundir atburða, hver með sitt hak: ómögulegt ferðalag, aðgangur sem sést í nýju landi, hrina misheppnaðra innskráninga, verkefni sem mistókst, öryggisafrit sem ekki var hægt að skrifa, eitthvað sem hætti að svara (Jellyfin, Sonarr, Radarr eða Seerr) og þegar það svarar aftur, nýir titlar í safninu (teknir saman eftir þáttaröð og degi, eins og hillan á yfirlitinu tekur þá saman), beiðni sem varð tilbúin til áhorfs, og spilun sem hefst eða lýkur. Slökkt er á tveimur síðustu sjálfgefið.
- Beiðni sem varð tilbúin til áhorfs nefnir titilinn, hver bað um hann og hve lengi viðkomandi beið.
- Persónulegir viðtakendur: veittu **Fá tilkynningar** og einhver getur bætt við sínum eigin, sem fær aðeins það sem viðkomandi má þegar sjá: eigin beiðnir og viðvaranir alltaf, annarra með *Sjá virkni allra*, vistföng með *Sjá netupplýsingar*. Persónulegur viðtakandi verður að vísa á opinbert vistfang; aðeins stjórnandi má beina honum inn á netið.
- **Taka með IP-vistföng og staði**, fyrir hvern viðtakanda og slökkt sjálfgefið: skilaboð um ómögulegt ferðalag nefna annars aðeins staði („Ósló, Noregur og London, Bretland“).
- **Nýlega sent**, í sama spjaldi: hvað var sagt, við hvaða viðtakanda, og hverju hin hliðin svaraði þegar skilaboðin bárust ekki. Misheppnuð skilaboð eru reynd aftur fjórum sinnum á um klukkustund, og beðið er eftir viðtakanda sem takmarkar hraðann.
- **Vistfang FinStats**, stilling fyrir stjórnendur, er það sem tengill tilkynningar er byggður úr. Autt þýðir að skilaboð bera engan tengil.
- Áætluð verkefni Jellyfin sjálfs á síðunni Þjónn: hvað hvert þeirra gerir í orðum frekar en eftir heiti kóðans, og hvort það er eitt af þeim þungu. Verkefni í gangi sýnir framvindustiku, hlutfall og hve mikið er eftir; hin sýna hvenær þau keyra næst, hvenær þau keyrðu síðast og hvort það gekk vel. Verkefni sem Jellyfin felur á eigin stjórnborði eru tekin með, og ekkert í FinStats getur ræst eða stöðvað verkefni.
- Hve mikið er eftir af verkefni er mælt út frá hraðanum sem hlutfall þess hefur raunverulega hreyfst á, miðað við nýjustu mælingu sem er nógu langt aftur til að segja eitthvað, frekar en meðaltal allrar keyrslunnar, svo verkefni sem hraðar á sér eða hægir á sér er lýst eftir hraðanum sem það hefur núna.
- Verkefnalistarnir sem FinStats les þegar af öðrum ástæðum (athugun á safnskönnun á 5 mínútna fresti, upplýsingar þjónsins á 15 mínútna fresti) mata sömu mælingu, svo verkefni sem hefur keyrt í klukkustund hefur yfirleitt mat um leið og síðan opnast. Ekkert aukalega er beðið um frá Jellyfin vegna þessa.
- Verkefni í gangi sem ekkert hefur verið mælt fyrir sýnir **ETA** og punkta sem fara í hring í stað tölu, og það sem hefur ekki hreyfst í hálfa mínútu sýnir „68% í 4 mín“ við hliðina. Ekkert kemur í stað mats sem var ekki mælt.
- Listi yfir allar útgáfur við hlið útgáfulýsinganna: einn smellur fer á hvaða útgáfu sem er, hann merkir þá sem verið er að lesa, og á mjóum skjá verður hann að línu af útgáfuröðum ofan við lýsingarnar.

### Changed
- Útgáfulýsingar fellast ekki lengur saman. Hver röð stendur opin og útgáfulistinn er leiðin til að komast um, í stað eins hóps opins í einu og smells til að ná í hvern annan.
- **Stillingar → Tengingar út á við** telur upp hvern viðtakanda tilkynninga ásamt Jellyfin og tengingum við þjónustur: hvað hann er, hýsil hans, hvort kveikt er á honum, og hvenær hann tók síðast við skilaboðum.
- Ekkert varðandi söfnun, feril eða nokkra tölu á skjánum breytist. FinStats *les* enn aðeins frá Jellyfin, Sonarr, Radarr og Seerr.

## [1.5.2] - 2026-09-22

### Changed
- Lestur á lotulista Jellyfin og endurstilling klukkunnar sem bað um hann eru eitt kall, svo þau geta ekki lengur farið hvort í sína áttina. Það var lögun bilunarinnar með stjórnlausu beiðnirnar sem fannst meðan verið var að smíða 1.5.0.
- Að sanna það tekur brot úr sekúndu í stað sjö mínútna á móti raunverulegum þjóni, svo það er athugað í hvert sinn sem eitthvað breytist. Sömu beiðnir, sami ferill, sömu tölur á skjánum.

## [1.5.1] - 2026-09-22

### Fixed
- Umkóðun er skráð einu sinni, ekki einu sinni á sekúndu. Þegar spilun byrjar með umkóðun og fer svo aftur í beina spilun skilur Jellyfin umkóðunarupplýsingarnar eftir á lotunni; hver mæling var borin saman við gildi sem FinStats hafði rétt í þessu skrifað yfir, svo hver mæling leit út eins og breyting og síðan Virkni fylltist af „Umkóðun“-línum sem á stóð „Bein spilun: …“ fyrir neðan.
- Línur sem þegar voru skrifaðar eru hreinsaðar við ræsingu: þar sem margar í röð segja það sama er sú fyrsta geymd. Spilun sem þurfti umkóðun telst enn umkóðun.

## [1.5.0] - 2026-09-22

Lotur raktar með WebSocket

### Changed
- FinStats heldur einni tengingu opinni við Jellyfin frá því það ræsist, og þannig fréttir það af spilun. 1.4.0 kom með þetta sem stillingu, slökkt sjálfgefið, til að sanna sig á raunverulegum þjónum; nú er alltaf kveikt á því. Sömu spilanir, hlé og stökk, reiknuð út af sama kóða.
- Ekkert í spilun: FinStats spyr Jellyfin einskis. Kvöld þar sem enginn horfir kostar engar beiðnir.
- Eitthvað í spilun: lotulistinn er lesinn einu sinni á sekúndu. Þaðan fá hlé, spólun eða skipti á rás nákvæmni sína, og það er það sem lýkur spilun þar sem biðlarinn hvarf án þess að kveðja.
- Allt sem er hlaðið hefur verið í hléi í þrjár mælingar í röð: spurningarnar hætta og tengingin tekur við, þar sem kvikmynd í hléi kostaði áður sömu spurninguna á hverri sekúndu, um 16 MB á klukkustund. Framhaldi, eða því að einhver annar byrjar á einhverju, er svarað innan um sekúndu, og tími í hléi telst samt aldrei áhorfstími.
- Hver lestur út á við biður um þjöppuð svör og afþjappar þau (Jellyfin, Sonarr, Radarr og Seerr þjappa öll þegar beðið er um það), sem tekur meira en helminginn af tveimur stærstu lestrunum: lotulistanum meðan á spilun stendur, og lestri safnsins. Staðsetningargagnagrunnurinn er enn sóttur eins og hann er geymdur.
- Jellyfin-spjaldið í Stillingum segir hvor helmingurinn er í gangi, og lifandi punkturinn fylgir því hvort tengingin flytur gögn frekar en því í hvorum helmingnum þú ert.
- Aðeins annar þeirra keyrir í einu: meðan FinStats spyr segir það Jellyfin að hætta að senda, og biður það að byrja aftur um leið og síðustu spilun lýkur. Tengingin helst opin allan tímann.
- Jellyfin sem svarar áskriftinni með engu er trúað meðan það svarar lífsmerkjum: FinStats segir það einu sinni, les lotulistann sjálft á meðan, og helst í áskrift þar til fyrsta sendingin sker úr. „Getur ekki sent“ og „var að endurræsa þegar við spurðum“ líta eins út, svo þögn er aldrei dómur.
- Varaleiðin er óbreytt: tenging sem lokast, þagnar eða reynist ekki tala þetta fellur aftur í að spyrja eftir tímastilli í næstu umferð og heldur áfram að tengjast aftur, með bilunum **Bil milli athugana meðan einhver horfir** og **Bil milli athugana meðan ekkert er í spilun**.

### Added
- Hvað gagnasöfnunin er að gera, á `GET /api/status`: hvert af fjórum ástöndum hún er í (`idle_socket`, `playing_poll`, `paused_socket`, `fallback`), hvort tengingin er opin og í áskrift, hve oft hún spyr, hve margir lestrar lotulistans fóru út síðustu mínútu, og hvenær það breyttist síðast. Svarað úr minni (engin beiðni, engin fyrirspurn), og það ber engin nöfn, titla eða talningar.
- Þessir reitir eru bornir saman hver við annan í hverri umferð, og ósamræmi sem varir er skrifað í annálinn.
- Einn takmarkari undir hverjum lestri lotulistans í ferlinu: í mesta lagi 2 á sekúndu og 70 á mínútu, og allt umfram það er tafið, talið og skráð í annálinn í mesta lagi einu sinni á mínútu. Venjulegt áhorf helst vel undir því.

### Fixed
- Lifandi uppfærslur koma nú í raun í stað fyrirspurnanna. Þegar kveikt var á þeim spurði 1.4.0 samt um lotulistann á fimm sekúndna fresti og opnaði tenginguna aftur á tuttugu til fjörutíu sekúndna fresti, því það dæmdi tenginguna eftir því hve nýlega listi hafði borist, en Jellyfin sendir lista þegar eitthvað *breytist* og ekkert þess á milli. Lífsmark er nú hvort Jellyfin svarar yfirhöfuð, athugað með lífsmerki um það bil tvisvar á mínútu.
- Lotulista sem barst meðan FinStats var milli umferða var hent, í þeirri trú að annar kæmi einni og hálfri sekúndu síðar. Hann er nú geymdur og notaður.
- **Síðast athugað** undir Stillingum, og spjaldið Tengingar út á við, úreldast ekki lengur meðan tengingin er hljóð: svar Jellyfin við lífsmerki er það sem þau sýna.

### Removed
- Stillingin **Láta Jellyfin senda hvað er í spilun** undir Stillingar → Söfnun, og `live_socket` úr stillinga-API-inu.
- `socket_enabled` úr stöðu gagnasöfnunarinnar: það er ekki lengur til neitt „slökkt“.
- `last_reconcile_at` úr stöðu gagnasöfnunarinnar, í stað þess kemur `socket_live`.

## [1.4.1] - 2026-09-22

### Changed
- Seerr er aðeins lesið í heild þegar eitthvað hefur breyst: á fimm mínútna fresti biður FinStats um eina línu, þá beiðni sem breyttist síðast, og umferðinni lýkur þar ef það þekkir hana þegar. Það er um eitt kílóbæti á móti um sextíu fyrir síðu með fimmtíu. Beiðnir sem eru enn á leiðinni eru skoðaðar á fjórðungs klukkustundar fresti, allt er talið upp einu sinni á dag, og ný beiðni birtist samt innan fimm mínútna.
- Biðraðir Sonarr og Radarr eru lesnar á fimm sekúndna fresti meðan síða sýnir þær, á mínútu fresti meðan eitthvað er í biðröðinni og á fimm mínútna fresti meðan hún er tóm, í stað þess að vera lesnar á mínútu fresti allan sólarhringinn. Þegar síðan er opnuð, þjónusta tengd eða ný beiðni berst í Seerr eru þær uppfærðar strax.

## [1.4.0] - 2026-09-21

Valfrjálsar WebSocket-lotur, og allir staðir sem FinStats getur náð til

### Added
- **Láta Jellyfin senda hvað er í spilun** undir Stillingar → Söfnun, slökkt þar til þú kveikir á því: ein tenging flytur lotulistann, og um 17.000 beiðnir á dag verða nánast engar. Spilanir, hlé, stökk og hópar eru reiknuð út af nákvæmlega sama kóða og áður. Meðan tengingin er virk sendir FinStats samt eina venjulega beiðni á mínútu *meðan eitthvað er í spilun*, og enga þegar ekkert er það. Tenging sem lokast, þagnar í fimmtán sekúndur eða reynist ekki tala þetta fellur aftur á tímastillinn í næstu umferð og heldur áfram að tengjast aftur.
- **Tengingar út á við**, spjald í Stillingum fyrir stjórnendur Jellyfin: hver áfangastaður sem FinStats getur náð til (Jellyfin-þjónninn þinn, „hver er IP-talan mín“-þjónustan, gagnagrunnur DB-IP, og hvert Sonarr, Radarr eða Seerr), með því til hvers hann er, hvort kveikt er á honum, og hvenær hann svaraði síðast. Byggt úr því sem FinStats veit þegar, og það sýnir aðeins vistföng: aldrei lykil, aldrei grunnslóð.
- **Fletta upp núna** undir Stillingar → Heimanet, fyrir daginn sem opinbera vistfangið þitt breytist.

### Changed
- Uppflettingin „hver er IP-talan mín“ keyrir einu sinni, í fyrsta skipti sem FinStats þarf vistfang, í stað þess að keyra á fimmtán mínútna fresti.
- Jellyfin-spjaldið í Stillingum segir hvernig gagnasöfnuninni berast upplýsingar, og, þegar kveikt er á tengingunni en hún flytur ekkert, hvers vegna ekki.

## [1.3.0] - 2026-09-20

Sonarr, Radarr og Seerr, á einni síðu

### Added
- **Tengingar** undir Stillingum, fyrir stjórnendur Jellyfin: **Sonarr**, **Radarr** og **Seerr** (eða Jellyseerr/Overseerr), nokkrar af hverri tegund, hver prófuð áður en hún er vistuð. Niðurhalsforritið þitt þarf enga uppsetningu, því Sonarr og Radarr tala þegar við það og FinStats les það sem þau vita.
- **Beiðnir**: hver bað um hvað, hve langan tíma það tók að koma, og hvort viðkomandi horfði nokkurn tíma á það. Reitir fyrir titla sem bíða og dæmigerða bið, graf yfir hvernig það breyttist mánuð fyrir mánuð, og listar yfir það sem kom fyrir vikum og var aldrei spilað, og hverjir biðja um mest.
- **Væntanlegt**: nýir þættir og útgáfur kvikmynda frá Sonarr og Radarr sem dagskrá eftir dögum, merkt með því hvort *þú* horfir á þáttaröðina og, fyrir þá sem mega sjá alla, hver annar gerir það. Þáttaröð telst áhorfð þegar einhver spilaði þátt úr henni síðustu fjóra mánuði. Röðin „Á næstunni“ á yfirlitinu, ein á hvern prófíl fyrir þáttaraðirnar sem viðkomandi horfir á, og hvað er næst fyrir titil á hans eigin síðu.
- **Niðurhal**: biðröð hvers Sonarr og Radarr í beinni (hvað það er, hve langt komið, hve hratt, hvað fór úrskeiðis við innflutning), með þeim sem bað um það við hliðina. Seríupakki er ein lína hversu marga þætti sem hann geymir, og það sem þarf athygli er efst. Fyrir neðan, það sem kom inn á viku, mánuði eða ári eftir leitarþjóni, gæðum og niðurhalsforriti, og hvaða niðurhöl mistókust. Yfirlitið sýnir sama lista, styttan.
- **Sjá hvað er verið að sækja**, ný heimild. Án hennar sér fólk samt hve langt *eigin* beiðni er komin og hve mikið er eftir, og ekkert annað: engin útgáfuheiti, engan hraða, ekkert annað niðurhal. Beiðnir annarra krefjast *Sjá virkni allra*.

### Changed
- FinStats sendir `GET` til þessara þjónustna og ekkert annað: enginn kóði í því gæti samþykkt beiðni, hafið leit, eða bætt við, gert hlé á eða fjarlægt niðurhal.
- API-lyklar þeirra eru geymdir í eigin gagnagrunni FinStats, aldrei sendir aftur til vafrans, aldrei skrifaðir í annál og aldrei hluti af öryggisafriti. Engri framsendingu er fylgt, svo lykill getur ekki ferðast eitthvert sem þú slóst ekki inn, og skilríki eru staðfest nema þú slökkvir á því fyrir eina tengingu.

## [1.2.2] - 2026-09-20

### Added
- Tungumál hljóðs og skjátexta á hverri kvikmynd og hverjum þætti („Hljóð: japanska · enska“).
- Hve langt talsetning nær: þáttaröð eða sería segir hve margir þáttanna hafa hvert tungumál („Enska: 13 af 26 þáttum“), og þáttalistinn hefur dálkinn **Hljóð**.
- **Tungumál hljóðs** og **Tungumál skjátexta** á síðum safna: hve margar skrár er hægt að spila á hverju.
- FinStats les safnið aftur einu sinni eftir þessa uppfærslu til að ná í tungumálin. Það segir aldrei „talsett“, því Jellyfin gefur ekki upp frummál titils, og rás án tungumálsmerkis er talin upp sem „Óþekkt“.

## [1.2.1] - 2026-09-20

### Changed
- Myndirnar í README eru teknar aftur í þessari útgáfu, með nýrri mynd af síðunni **Öryggi**. Ekkert í forritinu breytist.

## [1.2.0] - 2026-09-20

Hvaðan fólk horfir, á korti

### Added
- **Síðan Öryggi**, fyrir þá sem mega sjá netupplýsingar og virkni allra: heimskort með punkti fyrir hvern stað sem notendur þínir horfa frá, að stærð eftir því hve mikið gerist þar. Heima er grænt, staðir utan heimilis fjólubláir, streymi í beinni púlsa og misheppnaðar innskráningar utan frá eru rauðar. Dragðu til að færa, þysjaðu með hnöppunum, tökkunum eða Ctrl + skrunhjóli, og punktar sem skarast renna saman þar til þú þysjar inn. Fyrir neðan, hver staður með fólki sínu, spilunum, áhorfstíma og innskráningum, og löndin eftir spilunum.
- **Ómögulegt ferðalag**: viðvörun þegar einn aðgangur sést á tveimur stöðum sem ekkert flug tengir, eða á tveimur fjarlægum stöðum samtímis, með báðum skiptunum sem hann sást, fjarlægðinni, tímanum á milli og hraðanum sem það hefði krafist. **Sýna á korti** teiknar ferðina.
- **Nýtt land**: í fyrsta skipti sem einhver spilar eða skráir sig inn frá landi sem hann hefur ekki sést í áður.
- Leystu viðvörun með athugasemd, eða þaggaðu par af stöðum fyrir viðkomandi, vegna VPN eða síma þar sem símafyrirtækið situr í höfuðborginni. Viðvaranir sem finnast í gömlum ferli, eftir innflutning eða í nýjum gagnagrunni, eru skráðar sem leystar frekar en að flæða yfir listann. Viðvaranir og það sem þú ákvaðst um þær eru hluti af öryggisafritum.
- Staðir koma úr borgagagnagrunni sem er lesinn á staðnum: ekkert vistfang er nokkurn tíma sent neitt, og kortið er teiknað úr útlínum sem fylgja FinStats frekar en úr kortaþjónustu. Sæktu ókeypis gagnagrunn DB-IP af síðunni Öryggi eða í **Stillingar → Öryggi**, leyfðu FinStats að uppfæra hann mánaðarlega (slökkt þar til þú kveikir á því), eða settu þína eigin `.mmdb`-skrá (DB-IP, MaxMind GeoLite2-City) í `data/geoip/`. `FINSTATS_GEOIP_DB` vísar á skrá annars staðar.
- **Stillingar → Öryggi**: gagnagrunnurinn sem er í notkun, mánaðarlega uppfærslan, og hve hratt (900 km/klst.) og hve langt hvort frá öðru (500 km) tvö skipti sem aðgangur sést þurfa að vera til að teljast ómögulegt ferðalag. Borgagagnagrunnum skeikar oft um nokkur hundruð kílómetra, og fjarlægðin er til þess.

## [1.1.3] - 2026-09-20

### Added
- Ctrl+Space leitar líka í leikurum og aðstandendum alls í safninu þínu, undir **Leikarar og aðstandendur**, með mynd, hvort þeir leika eða leikstýra, og í hve mörgum af titlunum þínum þeir eru; ef einn er valinn opnast síða hans. Jafn umburðarlynd og titlaleitin (orð í hvaða röð sem er, broddstafir hunsaðir, innsláttarvilla leyfð), og þegar nokkrir passa jafn vel kemur sá fyrst sem er í fleiri af titlunum þínum.

## [1.1.2] - 2026-09-20

### Added
- **Nýlega bætt við** á yfirlitinu, fyrir ofan virknigrafið: 30 nýjustu viðbæturnar sem röð af veggspjöldum sem hægt er að fletta til hliðar, með örvum í tölvu. Hvert segir hvenær það kom og hvað það er (kvikmynd, plata, eða „Sería 4 · 3 þættir“), og stakur nýr þáttur segir hver hann er, með eigin veggspjaldi seríunnar þegar hún hefur það.
- **Shift + skrunhjól** og örvatakkarnir færa um eitt veggspjald, Page Up og Page Down um skjáfylli, Home og End fara á hvorn enda, og strokur virka. Venjulegt skrunhjól flettir enn síðunni.
- Nýir þættir sömu þáttaraðar sem koma sama dag deila einni færslu, og heil þáttaröð sem er bætt við í einu er ein færsla („3 seríur · 60 þættir“), svo einn stór innflutningur ýtir ekki öllu öðru af listanum.
- Röðin hunsar tímabil og manneskju yfirlitsins, því hún fjallar um safnið en ekki spilanir, og felur sig meðan safnið er tómt.

## [1.1.1] - 2026-09-20

### Added
- Síður opnast strax: andartaki eftir að FinStats hefur hlaðist sækir það í hljóði það sem líklegt er að þú opnir næst (yfirlitið, prófílinn þinn og tímalínuna, Virkni, Notendur, Söfn og hvert safn, Spilun, síður þjónsins, uppgjörið og virkasta fólkið), svo þegar ein er opnuð birtist hún samstundis og uppfærist samt á bak við tjöldin.
- Kvikmyndir, þáttaraðir og leikarar eru aðeins sótt þegar ætlunin sést, því þau skipta þúsundum: þegar bendillinn hvílir á tengli, snert er á honum eða lyklaborðið nær honum er sú eina síða sótt.
- Það bíður þar til síðan sem þú opnaðir hefur hlaðist, vinnur eina síðu í einu meðan vafrinn er aðgerðalaus, gerir hlé í flipa í bakgrunni, gerir ekkert á gagnasparandi eða mjög hægri tengingu, og gleymist þegar þú skráir þig út.
- Lítill tengill, **Kóði**, með GitHub-merkinu í stöðustikunni, við hlið útgáfunnar.

### Fixed
- **Tímalínan** gleymdi hvaða söfn höfðu verið afvalin þegar síðan var endurhlaðin eða opnuð úr bókamerki, þótt valið væri í vistfanginu.

## [1.1.0] - 2026-09-20

Tímalína yfir allt sem þú hefur horft á

### Added
- **Tímalína**, við hlið **Yfirlit** á hverjum prófíl: áhorfið þitt sem ein slóð frá deginum í dag aftur að fyrstu spilun sem FinStats veit um, þar sem kvöldi af þáttum er safnað í einn viðkomustað. Hver viðkomustaður er veggspjald með því sem horft var á og hvenær („Sería 2 · Þættir 3–6“, kvikmynd og hvort hún tók meira en eina setu, eða plata og hve mörg lög), og opnar titilinn.
- Slóðin fylgir breidd gluggans: þrír viðkomustaðir í röð á breiðum skjá, tveir á mjórri, önnur hver röð liggur öfugt með beygju sem tengir hana við þá næstu, niður í eina beina línu í síma. Fyrsti viðkomustaður hvers mánaðar ber nafn mánaðarins.
- Eldri viðkomustaðir hlaðast sjálfir þegar þú flettir, alla leið aftur þangað sem ferillinn byrjar.
- **Söfn**-rofar fyrir ofan slóðina sleppa því sem þú vilt ekki sjá, svo sem tónlist, og valið er hluti af vistfanginu.
- Það sem er enn í spilun segir **Í spilun núna**.
- Þú sérð þína eigin tímalínu; tímalínur annarra krefjast heimildarinnar „sjá alla“.

## [1.0.5] - 2026-09-20

### Added
- Vörn gegn niðurfærslu: gagnagrunnurinn man nýjustu útgáfu FinStats sem hefur opnað hann, og eldri útgáfa neitar að ræsast á honum í stað þess að vinna í hljóði með gögn sem hún skilur ekki til fulls. Skilaboðin segja hvað á að gera. Útgáfur upp að 1.0.4 komu út áður en þessi athugun var til.

## [1.0.4] - 2026-09-20

### Fixed
- Í upplýsingum spilunar hafði afritunarhnappurinn við hlið langs **Auðkenni tækis** fallið niður í eigin línu.
- Á síðunni Spilun var síðasti dálkurinn í **Hvaða forrit umkóða** skorinn af („Transc…“); fyrirsagnir með tveimur orðum brotna nú á milli lína.

### Changed
- Skjámyndirnar í README eru aftur nýjar, eftir að hafa enn sýnt útgáfu 0.5.0.

## [1.0.3] - 2026-09-20

### Added
- Síða þáttar sýnir eigin mynd þáttarins, þá sem Jellyfin sýnir í þáttalistanum sínum, í stað veggspjalds þáttaraðarinnar. Þáttur sem Jellyfin hefur enga mynd fyrir heldur veggspjaldinu.

## [1.0.2] - 2026-09-20

### Fixed
- Í **Útgáfulýsingum** höfðu hóparnir v0.9 og v0.10 enga fyrirsögn við hlið útgáfunúmersins. Fyrirsögn hóps er nú alltaf fyrsta setningin um það sem fyrsta útgáfa hans fjallaði um, svo langur inngangur er ekki skorinn af í miðri setningu.

## [1.0.1] - 2026-09-20

### Fixed
- Útgefna Docker-myndin ræstist ekki á nýrri vél: „unable to open database file: /data/finstats.db“, eftir að hafa hangið í hálfa mínútu. Docker býr til `data`-möppu sem vantar sem root, og FinStats keyrir sem venjulegur notandi. Gámurinn gerir nú möppuna að sinni og skiptir svo yfir í þann notanda, svo `docker run` virkar í fyrstu tilraun. Fyrir skrár í eigu annars en notanda 1000, stilltu `PUID` og `PGID`; `--user` virkar eins og áður.
- Gagnamappa sem í raun er ekki hægt að skrifa í er tilkynnt strax, með skipuninni sem lagar það, í stað þess að birtast 30 sekúndum síðar sem villa í gagnagrunni.

## [1.0.0] - 2026-09-20

Tilbúið fyrir alla

### Added
- Tilbúin Docker-mynd, nú á `ghcr.io/finstats/finstats`, fyrir 64 bita Intel/AMD- og ARM-vélar (Raspberry Pi 4 eða 5 virkar): `:latest` er nýjasta útgáfan, útgáfumerki helst þar sem það er, og `:edge` er þróunarútgáfan.
- Hver útgáfa birtist á útgáfusíðu verkefnisins á GitHub með þessum útgáfulýsingum.

### Changed
- Útgáfunúmer þýða nú það sem þau segja: 1.x-uppfærslur munu ekki brjóta gögnin þín, öryggisafritin eða stillingarnar.
- Docker Compose-skráin og allar leiðbeiningar nota útgefnu myndina. Sjálfsmíðuð mynd, `finstats:latest`, þarf aðeins að fá nýtt heiti; `data`-mappan færist yfir óbreytt.

## [0.10.2] - 2026-09-20

### Fixed
- Í síma var hægt að fletta Stillingum og síðu þáttaraðar til hliðar: merki sem aðeins var ætlað skjálesurum slapp út úr töflunni sinni, og þáttalistarnir og leikararöðin gerðu það sama. Nú flettast þau innan í sér.
- Ekki er lengur hægt að fletta uppgjörinu til hliðar þegar bakgrunnsorð kafla er breiðara en síðan.
- Löng „horfir á…“-lína undir **Í spilun núna** endar á úrfellingarpunktum í stað þess að vera skorin í miðjum staf.
- Veggspjaldið fremst við hverja þáttaröð á prófíl var annar, nafnlaus tengill fyrir skjálesara; það er nú falið þeim og titillinn við hliðina er tengillinn.

## [0.10.1] - 2026-09-20

### Fixed
- Niðurhal á öryggisafriti mistókst í vafranum („the source file could not be read“ í Firefox) án þess að neitt á síðunni segði hvers vegna: FinStats þjappaði öryggisafriti, sem er þegar þjappað, í annað sinn, og tvíþjappaði flutningurinn rofnaði áður en honum lauk. Öryggisafrit eru nú send eins og þau eru, með stærðina gefna upp fyrir fram, svo vafrinn getur sýnt raunverulega framvindu.

## [0.10.0] - 2026-09-20

Öryggisafrit, og flutningur í nýja uppsetningu

### Added
- Öryggisafrit í hverri viku, fimm nýjustu geymd: hver spilun með tímalínu hléa og stökka, merkingar um séð, heimildir, heimilisvistföng, atvikaskrá þjónsins og stillingarnar þínar, í einni lítilli skrá, nokkur hundruð KB fyrir þúsundir spilana. **Stillingar → Öryggisafrit** telur þau upp með **Sækja**-hnappi við hvert, býr til eitt eftir þörfum, og stillir hve oft og hve mörg. Þau eru í `backups`-möppunni í gagnamöppunni þinni. Öryggisafrit geymir engan API-lykil Jellyfin og enga innskráningarlotu, en það er heill áhorfsferill með IP-vistföngum, svo geymdu það á öruggum stað.
- Endurheimt sameinar frekar en að skrifa yfir: spilunum sem eru þegar til staðar er sleppt, svo það er öruggt að gera það tvisvar eða inn í uppsetningu sem hefur keyrt um hríð. Taktu hakið af „Endurheimta líka stillingar og heimildir“ til að fá aðeins ferilinn. Öryggisafrit virka milli útgáfna í báðar áttir. Úr skipanalínu: `finstats backup` og `finstats restore <file>`.

### Changed
- Söfnun athugar á hverri sekúndu meðan einhver horfir og á 5 sekúndna fresti meðan enginn horfir, þar sem áður var athugað á 5 sekúndna fresti allan tímann, svo hlé, stökk og skipti á rás eru skráð upp á sekúndu. Hvort tveggja er undir **Stillingar → Söfnun**; gamalt bil sem var breytt þarf að stilla aftur.

### Fixed
- IP-vistfang í upplýsingum spilunar brotnaði á tvær línur í miðju vistfanginu. Merkið Heimanet/Utan heimanets færist undir þegar ekki er pláss.

## [0.9.1] - 2026-09-20

### Fixed
- Fólk heima var sýnt sem utan heimanets. Tæki á þínu eigin neti sem nær í Jellyfin í gegnum opinbert nafn þess, svo sem öfugan milliþjón eða lén, kemur með opinbert IP-vistfang heimilisins, og aðeins einkavistföng töldust heimanet. FinStats lærir nú opinbera vistfangið og telur þær spilanir á heimaneti fyrir allan ferilinn, og man fyrri vistföng.

### Added
- **Stillingar → Heimanet**: vistföngin sem FinStats lítur á sem heima, hægt að bæta við eigin (fyrra vistfang, annað heimili, útgangspunkt VPN), eða slökkva á uppflettingunni. Uppflettingin spyr einfalda „hver er IP-talan mín“-þjónustu á 15 mínútna fresti, og reynir nokkrar (Amazon, Cloudflare og aðrar) því auglýsingasíandi DNS eins og Pi-hole lokar oft á þær, og ein þeirra þarf ekkert DNS. Beiðnin inniheldur ekkert um þig eða þjóninn þinn, og ef slökkt er á rofanum talar FinStats ekki við neitt nema Jellyfin. `FINSTATS_PUBLIC_IP_URL` stillir eigin þjónustu.

## [0.9.0] - 2026-09-20

Hver tafla, í þeirri röð sem þú vilt

### Added
- Hægt er að raða hverri töflu: smelltu á fyrirsögn til að raða eftir henni, aftur til að snúa röðinni við, og í þriðja sinn til að fá upprunalegu röðina. Merking reita er skilin, svo „3 d 2 klst“ raðast sem tími og „1,4 GB“ sem stærð, og tómir reitir fara síðast. Það nær yfir notendalistann, áhorfendur, þætti, tæki, viðbætur, áætluð verkefni, töfluyfirlit hvers grafs, heimildatöfluna og súlulistana (kóðanir, upplausnir, forrit og hitt), sem fá eigin granna fyrirsögn.
- Virkni og atvikaskrá þjónsins raðast yfir allar síður sínar frekar en línurnar á skjánum, og röðin er geymd í vistfanginu, svo hún lifir af endurhleðslu eða deildan tengil.
- Töflur með tíu línum eða fleiri fá lítinn síureit fyrir ofan sig.

## [0.8.2] - 2026-09-20

### Changed
- Ársvalið í uppgjörinu er sami rofi og hvert annað tímabilsval í FinStats (7 d · 30 d · 90 d …).

## [0.8.1] - 2026-09-19

### Added
- Síður fyrir leikara og leikstjóra, úr „Fólkið sem mest var horft á“ í uppgjörinu eða nýju röðinni **Leikarar og aðstandendur** á hverri kvikmynd, þáttaröð eða þætti: allt sem þeir eru í á þjóninum þínum, hve mikið hefur verið horft á og af hverjum, og hvað bíður enn. Það fylgir tímabilinu sem þú velur, þeir sem mega aðeins sjá eigin tölfræði sjá aðeins eigið áhorf þar, og Esc fer með þig til baka.

## [0.8.0] - 2026-09-19

Uppgjörið fær nýtt útlit, og hefur meira að segja

### Added
- **Fólkið sem mest var horft á**: leikararnir sem þú varðir mestum tíma með og leikstjórarnir á bak við það sem þú horfðir á, sem röð af andlitsmyndum. Leikarar og aðstandendur eru lesnir með safninu og birtast eftir fyrsta lestur safnsins eftir þessa uppfærslu.
- **Árið þitt í dögum**: hver dagur ársins sem einn reitur, bjartari eftir því sem þú horfðir meira, með lengstu hrinunni þinni og stærsta deginum fyrir ofan. Haltu bendlinum yfir degi, eða farðu um dagatalið með örvatökkunum, til að sjá hvað hann geymdi.
- **Í tölum**: spilanir, áhorfstími, ólíkir titlar, dagar með áhorfi, lengsta hrina og hve mikið var **endurtekið áhorf**, auk þess hvernig árið skiptist milli þátta, kvikmynda og tónlistar.
- Kaflinn um tegundir segir hve mörgum tegundum þú komst við og hve stóran hlut sú efsta tók.

### Changed
- Ný hönnun: það opnast á árinu þínu „endurspiluðu“, fyrirsögnin við hlið veggspjaldanna sem fylltu það, yfir hljóðbylgju ársins með einni súlu á viku og þá háværustu upplýsta, og þú velur árið beint fyrir ofan. Hver kafli hefur fyrirsögn með aðalorðið í fjólubláu, setningu sem ber tölurnar, og orð kaflans stórt og dauft á bak við. Það endar eins og kvikmyndir gera, með kreditlistanum: í aðalhlutverkum, leikstjórn, sýnt á, sýningartími.
- Klukkustundir, vikudagar og mánuðir deila einum kafla, **Taktur áhorfsins**, með rofa á milli þeirra, og fyrirsögnin fylgir því sem þú ert að skoða („Föstudagurinn tók krúnuna“).

### Fixed
- **Í spilun núna** á yfirlitinu var autt þegar enginn horfði, svo fyrirsögnin virtist tilheyra síunum undir henni. Reiturinn „Ekkert er í spilun núna“ er kominn aftur.

## [0.7.10] - 2026-09-19

### Added
- Myndir safna: hvert safnspjald, og eigin síða safnsins, sýnir myndina sem Jellyfin notar fyrir það, í gegnum myndamilliþjón FinStats eins og veggspjöldin. Safn án myndar heldur tákni sínu.

## [0.7.9] - 2026-09-19

### Changed
- Spjaldið Tegundir opnast aftur sem **Listi**, með ratsjána sem seinni kostinn á rofanum. Val sem þegar var gert er geymt.

## [0.7.8] - 2026-09-19

### Added
- Ratsjárframsetning fyrir spjaldið Tegundir, á yfirlitinu og á prófílum: einn armur á hverja tegund, skipt milli **Ratsjá** og **Listi** í horni spjaldsins, og valið er munað. Þegar bendlinum er haldið yfir punkti, eða örvatakkarnir notaðir, sést áhorfstími og hlutdeild þeirrar tegundar.

## [0.7.7] - 2026-09-19

### Fixed
- Kvikmyndir sem tilheyra samsafni voru sýndar sem „Ekki lengur í safninu“. Þegar kveikt er á *group movies into collections* lætur Jellyfin samsafnið í té í stað kvikmyndanna í því, svo FinStats sá þær aldrei; á einum þjóni faldi það 556 af 1.238 atriðum í kvikmyndasafninu. FinStats biður nú um kvikmyndirnar sjálfar, og lesturinn sem þessi uppfærsla ræsir sækir þær aftur og tengir spilanir þeirra á ný.

## [0.7.6] - 2026-09-19

### Changed
- Á prófílum færðist spjaldið **Þáttaraðir** niður rétt fyrir ofan Tegundir, svo langur listi af þáttaröðum ýtir ekki lengur áhorfstímanum, virknigrafinu og efstu titlunum úr augsýn. Daghrinurnar eru áfram efst, og þáttaröð sem þú hefur opnað helst opin þegar þú breytir tímabilinu.

## [0.7.5] - 2026-09-19

### Changed
- Leitin passar orð fyrir orð, í hvaða röð sem er, með hvað sem er á milli, í stað þess að þurfa nákvæmlega orðasambandið, svo „alya hides“ finnur *Alya Sometimes Hides Her Feelings in Russian*. Hástafir, broddstafir og greinarmerki skipta ekki máli, „The“ fremst er hunsað, og innsláttarvilla er fyrirgefin („mentalsit“ finnur *The Mentalist*); orð með þremur stöfum eða færri þarf samt að slá rétt inn. Betri samsvaranir raðast fyrst, og tónlist finnst líka eftir flytjanda.
- Leitarreitirnir á Virkni og í atvikaskrá þjónsins passa orð fyrir orð yfir alla dálka sína: „alya opera“ finnur spilanir á þeirri þáttaröð í Opera.

## [0.7.4] - 2026-09-19

### Changed
- Útgáfulýsingum er skipt eftir röðum: hver `0.x`-röð er ein felling sem geymir útgáfur sínar, með því um hvað röðin snerist, hve margar þær eru og hvenær. Aðeins sú nýjasta byrjar opin, og lokuð felling sýnir samt hvaða útgáfu þú ert að keyra.

## [0.7.3] - 2026-09-19

### Fixed
- Klukkan á spjöldunum Í spilun núna hikaði á nokkurra sekúndna fresti: 5 sekúndna uppfærslan endurmálaði hana líka, svo hvenær sem þjónninn var broti á undan færðist talan of snemma og stóð svo kyrr í sekúndu. Aðeins tifið einu sinni á sekúndu færir hana nú, um nákvæmlega eina sekúndu (mælt 999 til 1001 ms á milli), og uppfærslan leiðréttir samt eftir hlé, stökk eða raunverulegt rek, og lendir á tifinu.

## [0.7.2] - 2026-09-19

### Added
- Samáhorf í beinni: hvert spjald undir Í spilun núna segir „Með maria“ meðan fólk horfir á það sama saman, þar sem hópur var áður aðeins þekktur eftir að spilununum var lokið. Í beinni telst fólk saman þegar það er á sama titli og byrjaði annaðhvort innan hópbilsins eða er á nánast sömu stöðu, sem virkar líka ef FinStats var endurræst í miðju streymi.

## [0.7.1] - 2026-09-19

### Changed
- Í spilun núna telur hverja sekúndu: klukkan tifar og stikan rennur á milli, í stað þess að hoppa í hvert sinn sem þjónninn er spurður. Þjónninn er enn spurður á 5 sekúndna fresti og er áfram heimildin: birtingin samstillist aftur við hlé, stökk eða þegar þjónninn er á undan, en ekki fyrir stöðu sem er aðeins nokkurra sekúndna gömul, því biðlarar tilkynna aðeins á um það bil tíu sekúndna fresti. Streymi í hléi standa kyrr.

## [0.7.0] - 2026-09-19

Hverjir horfa saman

### Added
- Samáhorf: þegar ólíkt fólk byrjar á sama titli á sama tíma telur FinStats það sem samáhorf. Spjaldið **Horft saman** á yfirlitinu sýnir hópana, titlana sem þeir deila og klukkustundir þeirra saman, hver prófíll sýnir með hverjum viðkomandi horfir mest, sameiginlegar spilanir bera lítið fólksmerki í Virkni, og upplýsingar spilunar segja með hverjum var horft. Það virkar á allan ferilinn þinn, innfluttar spilanir meðtaldar.
- Stilling fyrir hve þétt byrjanirnar þurfa að vera, 60 sekúndur sjálfgefið: um þriðjungur raunverulegra hóplota byrjar með 6 til 60 sekúndna millibili. Fólk þarf líka að halda áfram að horfa hlið við hlið í nokkrar mínútur, svo tvær manneskjur sem opna sama þátt fyrir tilviljun eru ekki hópur.

### Changed
- Leit opnast með **Ctrl + Space** í stað Ctrl/⌘ + K.

## [0.6.1] - 2026-09-19

### Added
- Stjórnendur geta opnað uppgjör annarrar manneskju, efst á síðunni Uppgjör eða með **Opna uppgjör** á prófíl hennar. Það talar þá um hana með nafni í stað „þú“.

### Changed
- Allir aðrir sjá enn aðeins eigið uppgjör, hvaða heimildir sem þeir hafa, og enn er ekkert uppgjör fyrir allan þjóninn.

## [0.6.0] - 2026-09-19

Prófíllinn þinn, og hvar þú ert í hverri þáttaröð

### Added
- Framvinda þáttaraða á prófílum: hver þáttaröð sem þú hefur snert sem stika með einum hluta á hvern þátt (séð, byrjað eða ekki enn), sem má opna seríu fyrir seríu, raðað eftir því hve nálægt þú ert að klára, með aðskildum sýnum fyrir kláraðar þáttaraðir og allt.
- **Merkja sem séð** á þínum eigin prófíl, fyrir þátt, seríu eða þáttaröð, fyrir það sem þú horfðir á meðan ekkert var skráð. Eigin spilunarmerkingar Jellyfin eru líka notaðar. Merkingar haldast í FinStats og breyta aldrei neinu í Jellyfin; ekki er hægt að taka merkingu af skráðri spilun, en þínar eigin merkingar má taka af.
- Daghrinur á prófílum: lengsta samfellda röð daga með spilun, núverandi hrina, og á hve mörgum dögum þú hefur horft á eitthvað.
- **Prófíllinn minn** í hliðarstikunni.
- Virkni sýnir hvar spilun stöðvaðist (til dæmis `44:15 / 45:00`) undir framvindustikunni, og dagsetningu og tíma undir „fyrir 3 klst.“. Spilanir sem voru fluttar inn úr Jellystat hafa enga stöðvunarstöðu, því Jellystat skráði hana aldrei.

### Changed
- Þættir sem vantar eða hafa ekki verið sýndir enn eru ekki lengur taldir hluti af safninu þínu. Jellyfin telur þá upp sem staðgengla án skráar, og þeir töldust áður með í heildarfjölda þátta.

## [0.5.1] - 2026-09-19

### Added
- Merki: spilunarhnappur skorinn í þrjár súlur grafs, í hliðarstikunni og á innskráningarskjáunum, sem skörp SVG-mynd með `favicon.ico` í mörgum stærðum fyrir vafra og bókamerkjastikur sem vilja hana, auk heimaskjástákns fyrir síma og spjaldtölvur.

## [0.5.0] - 2026-09-19

Þú ákveður hver sér hvað

### Added
- Heimildir undir Stillingar → Aðgangsstýring, fyrir alla í einu eða manneskju fyrir manneskju: **sjá virkni allra** (tölfræði og feril annarra, síðuna Notendur, hvert streymi í beinni), **sjá netupplýsingar** (IP-vistföng, auðkenni tækja, heimanet eða ekki), **sjá þjóninn** (síðuna Þjónn, atvikaskrá þjónsins, misheppnaðar innskráningar, skráarslóðir) og **stjórna FinStats** (stillingar, verkefni, innflutning, eyðingu spilana).
- Innskráning fyrir hverja manneskju, til að hleypa nokkrum inn án þess að opna FinStats fyrir hverjum Jellyfin-notanda.
- Breytingar gilda strax, líka þegar aðgangur er tekinn af, og allt er framfylgt af þjóninum frekar en með því að fela hnappa.

### Changed
- Stjórnendur Jellyfin hafa enn allt og eru þeir einu sem geta breytt heimildum; sá sem má stjórna FinStats getur ekki veitt neitt, hvorki sjálfum sér né öðrum.
- Ársuppgjörið er áfram persónulegt hvað sem er veitt.
- Ekkert breytist í núverandi uppsetningum fyrr en þú veitir eitthvað.

## [0.4.5] - 2026-09-19

### Changed
- FinStats er gefið út undir **GNU General Public License v3.0**. Engin fyrri útgáfa var nokkurn tíma birt, svo ekkert var gefið út með öðrum skilmálum. Leyfi letranna sem fylgja með eru afhent við hlið þeirra.

## [0.4.4] - 2026-09-19

### Added
- Esc fer til baka: á opnu safni, notanda eða titli fer það með þig þangað sem þú komst frá, með listann nákvæmlega eins og þú skildir við hann. Allt sem er opið lokast fyrst (upplýsingar spilunar, leit, fellilisti, valmyndin í síma), og Esc er látið í friði meðan þú skrifar eða notar graf með lyklaborðinu.

## [0.4.3] - 2026-09-19

### Changed
- FinStats man hvað hver síða sýndi síðast: þegar farið er aftur á prófíl, safn eða tímabil sem þú hefur þegar skoðað er það teiknað strax, prófílmynd og allt, meðan nýjar tölur hlaðast á bak við, og síðan teiknast aðeins aftur ef eitthvað breyttist. Það sem er munað býr í flipa vafrans og er hent þegar þú skráir þig út eða breytir einhverju.
- Hleðslustaðgenglar birtast aðeins þegar hleðsla er í raun hæg. Áður sýndi hver síða þá í að minnsta kosti þriðjung úr sekúndu.

### Fixed
- Söfn sem var eytt í Jellyfin og höfðu aldrei spilun eru ekki lengur talin upp. Eytt safn með feril er enn sýnt, merkt fjarlægt.

## [0.4.2] - 2026-09-19

### Fixed
- Á síðunni Notendur sat merkið **Stjórnandi** í eigin línu og gerði þá línu hærri en hinar. Merki sitja nú við hlið nafnsins.

## [0.4.1] - 2026-09-19

### Fixed
- Orðið „null“ birtist ekki lengur undir titli yfirlitsins utan desember og janúar, þegar enginn uppgjörsborði er til að sýna.

## [0.4.0] - 2026-09-19

Sjáðu hvað breyttist án þess að yfirgefa forritið

### Added
- Flipinn **Útgáfulýsingar** sýnir hverja útgáfu, með þá útgáfu sem þú keyrir merkta.
- Útgáfan í stöðustikunni vísar á hann, og flipinn sýnir punkt eftir uppfærslu þar til þú hefur litið á hann.

### Changed
- Endurtenging ferils við endurnefnd atriði er miklu hraðari: úr 18 sekúndum í þriðjung úr sekúndu með 666 munaðarlausa titla, svo ræsing helst samstundis þegar ferillinn vex.

## [0.3.3] - 2026-09-19

### Changed
- Safnið er lesið aftur strax eftir að verkefninu **Scan Media Library** í Jellyfin lýkur, aldrei eftir sérstökum tímastilli og aldrei meðan skönnun er í gangi. FinStats hefur alltaf aðeins lesið frá Jellyfin, og ræsir þar aldrei skönnun.
- Vikulegur öryggislestur nær yfir þjóna sem treysta á rauntímavöktun, þar sem skönnunarverkefnið keyrir kannski aldrei.
- Gamla bilið er nú aðeins varaleið, fyrir þegar slökkt er á eftirfylgninni eða þjónninn gefur ekki upp neitt skönnunarverkefni.

### Added
- Rofinn **Fylgja safnskönnun Jellyfin** undir Stillingar → Söfnun, kveikt á honum sjálfgefið.

## [0.3.2] - 2026-09-19

### Fixed
- Ferill lifir af endurnefningar. Jellyfin gefur endurnefndri skrá nýtt auðkenni, sem skildi áður fyrri spilanir hennar eftir undir hráu möppunafninu (`Title (2010) [tmdbid-…] [imdbid-…]`) án veggspjalds. Þær spilanir eru tengdar aftur við núverandi færslu í safninu: eftir TMDB/IMDb/TVDB-auðkenni þegar gamla nafnið ber eitt, annars eftir hreinsuðum titli og ári, og þættir eftir þáttaröð auk seríu- og þáttanúmers. Samsvörun þarf að vera ótvíræð, svo titlar sem var í raun eytt haldast eins og þeir eru.
- Sjónvarpsrásir í beinni sem voru fluttar inn úr Jellystat voru taldar kvikmyndir, því Jellystat skráir enga tegund atriðis. Þær eru nú þekktar sem sjónvarp í beinni, og nýjar spilanir á sjónvarpi í beinni eru skráðar með tegund Jellyfin sjálfs.

### Changed
- Sjónvarp í beinni er alveg haldið utan uppgjörsins: topplistar, heildartölur, áhorfspersónuleiki, met og sæti.

### Added
- `finstats relink` til að tengja feril aftur eftir þörfum. Það keyrir líka við ræsingu og eftir hvern lestur safns.

## [0.3.1] - 2026-09-19

### Changed
- Uppgjörið er stranglega persónulegt: allir, stjórnendur meðtaldir, sjá aðeins sitt eigið, og sýnin „Allir“, notendaskiptirinn og kafli þjónsins eru horfin.
- Uppgjörið opnast á árinu sem er „tilbúið“: yfirstandandi ári í desember, annars árinu á undan. Önnur ár og „Síðustu 12 mánuðir“ eru áfram einum smelli frá.

## [0.3.0] - 2026-09-19

Árið þitt í hnotskurn

### Added
- Flipinn **Uppgjör**, saga ársins sem flett er í gegnum: klukkustundir sem horft var, efstu þáttaraðir, kvikmyndir, lög og tegundir, og áhorfspersónuleiki (nátthrafn, morgunhani, helgarhetja, hámhorfari, bíófíkill, tónlistarunnandi, eins og klukka) með gröfum eftir klukkustund dags og vikudegi.
- Árið mánuð fyrir mánuð, með titlinum sem einkenndi hvern mánuð.
- Met: stærsti dagurinn, mesta hámhorfið, lengsta daglega hrinan, lengsta staka spilunin, titillinn sem oftast var horft á aftur, fyrsta spilun ársins og elsti titillinn sem horft var á.
- Uppgötvanir: nýjar þáttaraðir sem byrjað var á, kvikmyndir og þættir sem voru kláraðir, og þáttaraðirnar sem fengu nákvæmlega einn þátt.
- Eins línu borði á yfirlitinu í desember og janúar þegar uppgjör er tilbúið.

## [0.2.0] - 2026-09-19

Meira en Jellystat skráði nokkurn tíma

### Added
- Tímalína fyrir hverja spilun: byrjun, hlé, framhald, stökk, skipti á hljóði og skjátextum, bein spilun sem breytist í umkóðun, og stöðvun. Auk fjölda hléa og stökka, hvaðan spilun hélt áfram, og hvort áhorfandinn var á staðarnetinu.
- Flipinn **Þjónn**: útgáfa Jellyfin, uppfærslur og endurræsingar sem bíða, diskanotkun á hvert safn (Jellyfin 10.11+), viðbætur, niðurstöður áætlaðra verkefna og hvert skráð tæki.
- Innsýn í **Spilun**: samtímis streymi yfir tíma, hvaða forrit kalla á umkóðun, hve langt fólk kemst áður en það hættir, spilanir á heimaneti á móti utan þess, og áhorfshegðun.
- Úr hverju safnið þitt er gert: upplausnir, kóðanir, birtusvið, ílát, titlar eftir áratug, viðbætur á mánuði, stærstu titlarnir, og titlarnir sem enginn hefur nokkurn tíma horft á, athugað bæði á móti ferli FinStats og eigin spilunarmerkingum Jellyfin.
- Yfirlit: mest samtímis streymi, áætlað gagnamagn streymt, hlutfall spilana utan heimanets, tegundir eftir áhorfstíma og misheppnaðar innskráningar.
- Síður atriða: tenglar á IMDb og TMDB, kvikmyndaver, bitadýpt og rammatíðni, og hver hefur merkt titilinn sem spilaðan í Jellyfin.
- Síður notenda: tegundir, og kvikmyndirnar, þættirnir og eftirlætin sem Jellyfin hefur skráð.

### Fixed
- Tímamerkið á spjaldi undir Í spilun núna flæðir ekki lengur út fyrir spjaldið.
- Löng merki á síðunni Spilun voru skorin of snemma af.
- Þáttaröð sem hefur yfirgefið safnið sýnir enn lista yfir seríur sínar og þætti.

## [0.1.0] - 2026-09-19

Fyrsta útgáfa

### Added
- Söfnun í beinni úr lotum Jellyfin: notandi, titill, forrit, tæki, IP-vistfang, spilunaraðferð, ástæður umkóðunar og vélbúnaðarhröðun, upplýsingar um mynd, hljóð og skjátexta, og áhorfstími á móti tíma í hléi.
- Innskráning með Jellyfin-aðganginum þínum. Uppsetning í tveimur skrefum býr til eigin API-lykil FinStats, og lykilorð eru aldrei geymd. Notendur sem eru ekki stjórnendur geta valið að skrá sig inn til að sjá aðeins eigin tölfræði.
- Síðurnar Yfirlit, Virkni, Notendur, Söfn, Spilun og Atvikaskrá þjóns, skipanagluggi (Ctrl/⌘ K) og lifandi stöðustika.
- Innflutningur úr Jellystat: stór öryggisafrit streymast á disk og flytjast inn á sekúndum, og öruggt er að flytja sömu skrána inn tvisvar.
- Ein lítil keyrsluskrá með innbyggðu vefviðmóti og einni SQLite-skrá, auk Docker-myndar.
