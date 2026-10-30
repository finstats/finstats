# Notes de version

Tout ce qui a changé dans FinStats, du plus récent au plus ancien. Ce fichier est compilé dans le
binaire, et c’est lui qu’affiche l’onglet **Notes de version** de l’application.

Format : `## [version] - date`, ou `## [version] - first day to release day` pour une version préparée
sur plusieurs jours, puis `### Added`, `### Changed`, `### Security`, `### Performance`, `### Stability`,
`### Fixed` ou `### Removed` (Ajouts, Modifications, Sécurité, Performances, Stabilité, Corrections,
Suppressions) avec une puce par changement. Sécurité désigne ce dont quelqu’un aurait pu abuser,
Performances ce qui est devenu plus rapide ou plus léger, Stabilité ce qui ne peut plus planter, s’arrêter,
fuir ou perdre de données, Corrections ce qui donnait une réponse fausse. Une version `x.y.0` porte une
courte ligne de titre sous son en-tête : c’est ce titre que l’application utilise comme intitulé de toute
la série.

## [2.3.1] - 2026-10-26 to 2026-10-28

### Security
- **La configuration demande un code à usage unique** : tant qu’il n’est pas configuré, FinStats écrit un code dans son journal au démarrage (`docker logs finstats`), et l’assistant le demande avant de tester une adresse ou de configurer quoi que ce soit. Sinon, le premier à atteindre le port pouvait diriger FinStats vers son propre Jellyfin et en devenir l’administrateur, ou se servir du test pour explorer le réseau.
- **L’accès aux médiathèques suit Jellyfin** : chacun ne voit que les médiathèques que Jellyfin lui ouvre, et dans celles-ci que les titres que Jellyfin lui liste selon sa limite de classification, ses étiquettes et ses règles sur les titres non classés, dans les pages des médiathèques, la recherche, la distribution, les images, Ajouts récents, la Liste à voir, À venir, le flux du calendrier et les notifications de nouveaux titres. Toute personne connectée pouvait parcourir toutes les médiathèques.
- Un lien pouvait envoyer quelqu’un de l’accueil vers un autre site juste après sa connexion. Seules les pages de FinStats sont désormais suivies.
- Le journal du serveur et les connexions échouées des analyses donnaient les adresses de connexion et l’activité de tous à quiconque avait *Voir le serveur*. Les lignes de quelqu’un d’autre exigent *Voir l’activité de tous*, et une adresse *Voir les détails réseau*, comme partout ailleurs.
- Une destination de notification personnelle pouvait atteindre le réseau local par un nom qui répondait autrement à la deuxième résolution. La connexion se fait désormais à l’adresse vérifiée, davantage de plages qui n’atteignent jamais Internet comptent comme locales, l’adresse publique du foyer est refusée elle aussi, et un nom impossible à résoudre est refusé dans les mêmes termes qu’un nom local.
- Derrière un proxy qui ajoute sa propre ligne `X-Forwarded-For`, ou depuis un réseau IPv6, chaque tentative de connexion pouvait venir d’une nouvelle adresse et la limite ne s’appliquait jamais. Un /48 IPv6 a désormais 50 essais en cinq minutes, une limite pleine refuse une nouvelle adresse au lieu d’en oublier une ancienne, tout le serveur s’arrête à 60 échecs par minute, et une connexion échouée est consignée une fois par adresse et par fenêtre.
- `FINSTATS_TRUST_PROXY` nomme les proxys (adresses ou plages) dont on croit `X-Forwarded-For`, `X-Forwarded-Host` et `X-Forwarded-Proto`, et `1` ou `true` ne croit plus qu’un pair en boucle locale ou sur un réseau privé. Activé, n’importe quel pair pouvait choisir sa propre adresse ; désactivé derrière un proxy, dix mauvais essais bloquaient tout le monde.
- Une fois les utilisateurs lus, une connexion pour un nom inconnu, désactivé, supprimé ou non admis dans FinStats reçoit la réponse d’un mauvais mot de passe sans que Jellyfin soit interrogé, et pas plus tôt que Jellyfin n’aurait répondu. De tels noms pouvaient bloquer des comptes Jellyfin, contourner la règle de Jellyfin sur l’accès distant ou se cacher derrière l’adresse de FinStats.
- Une personne que Jellyfin limite au réseau du domicile (*Autoriser les connexions à distance* désactivé) ne se connecte à FinStats que depuis celui-ci, puisque Jellyfin jugeait ce réglage sur l’adresse de FinStats, et une lecture des utilisateurs qui ne dit pas si quelqu’un est désactivé le laisse dehors.
- Un cookie de session est marqué Secure dès que la page de connexion est en https, y compris derrière un proxy qui ne le signale pas.
- Une connexion réussie n’efface plus les tentatives qui l’ont précédée. En https, le cookie de session est un cookie `__Host-` qu’un site sur un nom voisin ne peut pas remplacer et qui n’est accepté sous aucun autre nom, et une déconnexion met fin à toutes les sessions envoyées par le navigateur.
- Seule une clé de calendrier ouvre le flux du calendrier depuis son adresse ; toute autre clé passe dans l’en-tête.
- Une clé d’API ne peut plus accorder d’autorisations, changer qui entre ou ce qui est public, restaurer une sauvegarde, ni ajouter, modifier ou tester un service ou une destination de notification : chacun survit à la clé, il faut donc un navigateur connecté.
- Chaque réponse porte `Cross-Origin-Opener-Policy: same-origin` et une `Permissions-Policy`, les réponses de l’API portent `Cross-Origin-Resource-Policy: same-origin` et sont `no-store` sauf indication contraire, et la CSP n’autorise plus les images `blob:`. Une page qui avait ouvert FinStats pouvait ensuite transformer son onglet en une page de connexion à elle.
- Là où FinStats connaît le schéma par lequel on l’atteint (le `X-Forwarded-Proto` d’un proxy de confiance, ou l’adresse publique), une écriture venant d’un autre schéma est refusée comme intersite.
- `/auth/me`, la page d’un titre et Santé de la médiathèque renvoient vers Jellyfin à son adresse publique ; l’adresse à laquelle FinStats lui-même lit Jellyfin n’est donnée qu’aux administrateurs.
- Une image n’est demandée à Jellyfin que pour quelque chose que FinStats possède, affiches et cartes d’un profil publié comprises, si bien qu’on ne peut pas demander par identifiant les images d’une autre médiathèque.
- Les clés d’un tracker privé sont aussi masquées dans les erreurs de la file de téléchargement, d’après leur forme comme d’après leur nom, et une personne qui n’a que *Voir les téléchargements* apprend quel service a échoué et comment, jamais son adresse interne ni vers où il redirige.
- La page Serveur ne nomme le dernier utilisateur d’un appareil qu’avec *Voir l’activité de tous*, et Médias non liés l’exige en plus de *Gérer FinStats*. Sans elle, la section Tâches ne compte que ses propres lectures et flux, une médiathèque supprimée est listée d’après ses propres lectures, et un tableau Tautulli en attente ne peut pas être annulé.
- Les paramètres affichent le service d’adresse publique par son seul hôte, et l’adresse de Jellyfin et le dossier de données aux seuls administrateurs, et un titre perdu par la médiathèque n’est décrit qu’à partir des lectures que la personne qui consulte la page a le droit de voir.
- Une sauvegarde Tautulli dont les tables n’étaient pas de simples tables pouvait faire attendre toute autre écriture jusqu’au redémarrage de FinStats. Un tel fichier est refusé, et SQLite lit la sauvegarde de manière défensive.
- Un envoi de plus de 4 Go est refusé, et une sauvegarde Tautulli zippée n’est pas décompressée au-delà de la taille qu’elle annonce ; l’un comme l’autre pouvait remplir le disque où se trouve la base de données de FinStats.
- Une réponse de Jellyfin, Sonarr, Radarr, Seerr, de la recherche d’adresse publique ou du téléchargement GeoIP n’est pas lue au-delà du maximum permis, compté après décompression.
- N’importe quel utilisateur Jellyfin pouvait empêcher FinStats d’enregistrer les lectures de tout le monde en signalant des milliers de lectures courtes d’un même titre. Trouver qui a regardé ensemble se fait désormais en une seule passe, sans garder de verrou pendant le travail.
- Des lectures enchaînées, ou un même titre lu à tour de rôle d’un appareil à l’autre, ne réunissent plus des inconnus en un groupe, et une personne qui ne peut voir qu’elle-même n’apprend que les personnes à côté desquelles sa propre lecture a tourné, et pour combien de temps.
- Un lien pouvait encore envoyer quelqu’un de l’accueil vers un autre site après sa connexion, par un chemin comme `/.//`.
- Quiconque gère FinStats ne peut plus changer le nombre de sauvegardes conservées ni leur fréquence, la fenêtre de fusion (désormais une heure au plus) ni le moment où les utilisateurs, le journal d’activité et les sauvegardes sont lus, et ses sauvegardes n’évincent jamais celles d’un administrateur ; les adresses du domicile et les règles de déplacement exigent ce qu’exigent les alertes, et importer l’historique exige aussi *Voir l’activité de tous*.
- Une clé, un jeton ou un mot de passe enregistré ne part que vers le serveur pour lequel il a été enregistré, et une destination de notification n’est créée ou modifiée qu’en étant connecté, jamais avec une clé d’API. Un administrateur peut désactiver ou supprimer la destination personnelle de quelqu’un d’autre, mais pas la rediriger, ni désactiver la vérification de ses certificats, ni lire son sujet ntfy ou sa clé utilisateur Pushover.
- Les webhooks Discord et Slack doivent être en `https`, puisque leur adresse est leur jeton.
- Une avalanche de connexions échouées ne peut plus cacher une vraie connexion ni noyer les alertes : les noms qui ne sont pas un compte donnent un seul avis par quart d’heure, qui ne nomme personne, les alertes partent en premier, et Slack et Discord affichent le contenu d’un message comme du texte et ne mentionnent personne.
- Le rang du bilan, Capacité, une notification et un profil publié n’en disent pas plus que les pages : aucun rang ni part du temps de tous pour qui ne peut voir que soi-même, les pics seulement comme des jours de trois flux ou plus sans *Voir l’activité de tous*, aucune tâche ni connexion en échec pour qui ne peut pas gérer FinStats, et un profil public change une fois par jour.
- Une sauvegarde Tautulli avec une colonne générée est refusée, et l’image d’une lecture importée n’est demandée qu’une fois son titre dans la médiathèque.
- Une restauration n’ouvre jamais FinStats plus qu’il ne l’est : les paramètres ne reviennent que sur demande, la connexion, les autorisations et les profils publics deviennent ce que permettent à la fois cette installation et la sauvegarde, et les adresses publiques restent celles de cette installation. `finstats restore <file>` ne ramène que l’historique, comme la page, et aussi les paramètres avec `--with-settings`.
- Une restauration de l’historique seul ne ramène plus les adresses du domicile, les alertes mises en sourdine, les entrées d’audit qui se feraient passer pour celles de cette installation ni les titres localisés absents de la médiathèque, et aucune restauration ne ramène de lectures, lignes de journal ou appareils datés d’avant 1990 ou d’après demain. Une lecture restaurée compte comme importée quelle que soit la source qu’elle prétend avoir, si bien qu’un redémarrage ne la poursuit jamais.
- Le dossier de données n’appartient qu’à l’utilisateur de FinStats (0700, ses fichiers 0600, `umask 077` dans l’image). FinStats refuse de tourner en root sur un dossier de données appartenant à un autre utilisateur et indique le `docker exec -u` à utiliser, et le point d’entrée ne change le propriétaire que du dossier et des entrées propres à FinStats.
- Le nom d’une destination ne peut contenir aucun caractère de contrôle, et le nom d’un appareil, d’une personne ou d’un titre ne peut plus forger une ligne dans le journal.
- Le paramètre de route d’un lien ne peut plus atteindre un autre chemin d’API que celui de sa page, et le constructeur d’éléments de FinUI ne pose une adresse que si elle est en http(s) ou relative, jamais `srcdoc`.
- La recherche, les profils publiés et le travail lancé à la main ne peuvent plus accaparer FinStats : une recherche s’arrête à 200 caractères, un profil est gardé par langue, les cartes attendent dans une file courte, et une tâche, la recherche d’adresse publique, le téléchargement de la base de localisation et un message de test ne se lancent qu’à intervalles limités, de quelque façon qu’on les demande, les messages de test étant comptés par expéditeur. Télécharger la base de localisation exige aussi ce qu’exige la lecture de la page Sécurité.
- Le corps d'une requête doit continuer d'arriver une fois ses 15 premières secondes passées, une réponse que personne ne lit est abandonnée après 30 secondes, et une adresse tient au plus 64 connexions, un proxy de confiance mis à part. Quelques centaines de requêtes qui n'envoyaient jamais leur corps pouvaient empêcher FinStats de répondre à qui que ce soit.
- Un serveur de messagerie n'est pas lu au-delà de ce qu'une réponse peut contenir (1 Mo pour une lettre entière), et un message de test s'arrête au bout du temps qu'un message en file peut prendre. Un serveur qui ne terminait jamais une ligne pouvait faire garder à FinStats de plus en plus de mémoire jusqu'à son arrêt.
- La lecture d'une bibliothèque est jugée sur les titres que les lectures ont enregistrés, pas sur ceux d'une importation, et un titre Jellystat dans une bibliothèque déjà lue arrive comme disparu. Une importation de titres inventés pouvait faire arrêter FinStats à chaque lecture de bibliothèque, à chaque démarrage.
- La limite de 60 connexions échouées par minute sur tout le serveur ne compte que les tentatives soumises à Jellyfin, y compris celles qui attendent encore sa réponse. Les noms que FinStats refusait lui-même permettaient à quiconque disposait de quelques adresses d'empêcher tout le monde de se connecter, et de nombreux essais envoyés à la fois atteignaient tous Jellyfin.
- Une personne que Jellyfin limite au réseau domestique y reste limitée à chaque requête, sa session et ses clés comprises, et plus seulement à la connexion ; la maison est un réseau privé ou l'adresse publique du foyer telle que la recherche l'a trouvée, jamais une adresse saisie comme domicile, et derrière un proxy que `FINSTATS_TRUST_PROXY` ne nomme pas elle reste dehors. Un cookie ou une clé emporté hors de la maison ouvrait tout depuis n'importe où, et un gestionnaire pouvait ouvrir la maison à l'adresse de son choix.
- Une clé d'API ne peut plus désactiver ce qui remarquerait qu'on en abuse, ni les sauvegardes qui annuleraient ce qu'elle a fait : combien de sauvegardes sont gardées et à quelle fréquence, la fenêtre de fusion, les adresses du domicile et les règles de déplacement, les planifications des utilisateurs, du journal d'activité et des sauvegardes, mettre en sourdine ou résoudre toutes les alertes, mettre une sauvegarde à la corbeille ou supprimer une destination de notification. Chacun survivait à la clé une fois celle-ci révoquée.
- Les affiches de Sonarr et Radarr, et le prochain épisode d'une liste de souhaits, suivent aussi l'accès aux bibliothèques : une série que Jellyfin cache à quelqu'un n'a pour lui ni affiche ni date. Compter les identifiants des services révélait quelles séries cachées arrivaient, et une liste nommait le prochain épisode d'une série cachée.
- Le chapitre Ensemble du bilan ne nomme que les personnes dont la lecture a tourné à côté de la vôtre, comme la page Ensemble. Un groupe rejoint par quelqu'un d'autre nommait des gens avec qui vous n'aviez jamais regardé.
- Une importation Jellystat ou Streamystats lit tout son fichier avant de prendre le verrou d'écriture de la base, puis n'écrit que les lignes qu'elle garde. Lu sous le verrou, un fichier de quelques vraies lignes et de gigaoctets d'autre chose faisait attendre toute autre écriture, lectures et connexions comprises.
- Une restauration garde une autorisation par personne, pour pas plus de personnes qu'un foyer bien des fois, et cherche un doublon d'une entrée d'audit sans relire chaque entrée qu'elle vient d'écrire. Une sauvegarde fabriquée pouvait retenir la mémoire et le verrou d'écriture de la base de plus en plus longtemps pendant la restauration.
- Une restauration avec ses réglages n'active la recherche de l'adresse publique et le téléchargement de la base de localisation que là où cette installation les a aussi activés, et ne garde jamais moins de sauvegardes ni ne les écrit moins souvent que cette installation. Une sauvegarde qui en gardait une pouvait faire supprimer toutes les autres au passage suivant.
- Une taille de fichier qu'aucun disque ne pourrait contenir est lue comme aucune taille, et les tailles sont additionnées de sorte qu'aucun total ne puisse déborder. Une seule taille importée faisait répondre les pages des bibliothèques par une erreur pour de bon.
- Dès que FinStats sait qu'il est joint en https (son adresse publique est en https, ou un proxy de confiance le dit), une session n'est prise que dans son cookie `__Host-`, et avec une adresse publique en https chaque nouvelle session est une session https. Une connexion envoyée sans `Origin` par un proxy qui ne disait rien créait une session ordinaire qu'un site voisin pouvait déposer dans le navigateur de quelqu'un.

### Performance
- Un profil publié est calculé au plus une fois par minute, et non à nouveau pour chaque image et chaque carte qu’il contient.
- Une seule analyse à la fois cherche des alertes de sécurité dans l’historique d’une personne, quel que soit le nombre de lectures qui commencent.
- Se connecter ou se déconnecter, révoquer une clé ou changer les autorisations d’une personne ne demande de revérifier qu’aux connexions en direct concernées.
- Le tableau de câblage Tautulli lit l’historique une seule fois, et non une fois par utilisateur Plex.
- Une image absente du cache est demandée par une seule requête pendant que les autres attendent, et une image que la source n’a pas pu fournir n’est pas redemandée avant 30 secondes.
- Les fichiers de l’interface web sont compressés en gzip une fois pour toutes et conservés, et les polices ne sont plus compressées à nouveau.

### Stability
- Une sauvegarde au format ancien de Jellystat est lue ligne par ligne au lieu d’être chargée en entier, et une ligne plus longue que celle de toute sauvegarde est refusée.
- Deux envois Tautulli simultanés n’écrivent plus dans le même fichier.
- Un numéro de page bien au-delà de la fin des titres d’une médiathèque ou de Santé de la médiathèque donne une page vide, et non une erreur.
- Une importation ou une restauration dont l’envoi a été interrompu ne refuse plus toutes les autres jusqu’à un redémarrage, et ni elle ni une importation abandonnée avant de commencer ne laisse de fichier derrière elle.
- Une ligne d’une sauvegarde n’est pas lue au-delà de 8 Mo, et une seule sauvegarde Tautulli arrive à la fois.
- Une lecture garde au plus 2 000 événements, et une importation ignore une lecture qui ne commence pas puis ne se termine pas entre 1990 et demain.
- Un seul numéro d’épisode très négatif ne bloque plus toutes les lectures de la médiathèque : Santé de la médiathèque (son résumé aussi), Capacité, la reliaison et la localisation additionnent et comparent les nombres sans débordement, et une version de production vérifie les débordements, si bien que tout oubli arrête une tâche au lieu de tourner en boucle.
- Un numéro de saison, d’épisode ou de fin n’est conservé que là où Jellyfin pourrait le contenir (0 à 2 147 483 647), qu’il vienne de Jellyfin, de Jellystat ou de Tautulli.
- Une sauvegarde Tautulli dont les identifiants se répètent est refusée, car quelques Mo de lignes sous un même identifiant épuisaient la mémoire du processus, et une sauvegarde est ouverte en mode immuable, sans laisser de `-wal` ni de `-shm` derrière elle.
- Une connexion doit envoyer sa requête dans les 15 secondes et au plus 512 sont servies à la fois : quelques centaines de sockets à moitié ouverts mettaient FinStats hors ligne. Une trame du canal en direct est limitée à 4 Ko.
- Quelques kilo-octets venant de Sonarr, Radarr ou Seerr ne peuvent plus devenir des gigaoctets de mémoire : une liste ne garde pas plus que ce que FinStats en lit.
- Une destination de notification qui ne répond jamais ne retarde qu’elle-même : les destinations sont servies en parallèle, et une qui dépasse sans cesse le délai est laissée de côté pendant dix minutes.
- Le temps restant et la vitesse totale d’une file, les nombres d’une lecture restaurée et le jour où une lecture quitte la corbeille ne peuvent plus déborder.
- Une restauration lit tout le fichier avant de prendre le verrou d’écriture, refuse un fichier qui se décompresse en bien plus que sa taille, et écrit par lots, si bien que le collecteur continue d’enregistrer pendant ce temps.
- Les sauvegardes sont nommées en UTC et n’écrasent jamais un fichier existant, et seules les copies d’avant mise à jour de FinStats lui-même sont élaguées.
- Une ligne du journal du serveur datée dans le futur n’empêche plus la lecture du journal d’activité : il est lu à partir de sa ligne la plus récente qui n’est pas dans le futur.
- Des requêtes arrivant ensemble sur un lien publié calculent sa réponse une seule fois, et ses cartes sont dessinées dans une file à part, si bien qu’un détenteur du lien ne peut plus faire répondre 503 aux cartes du bilan du propriétaire.

### Fixed
- Les connexions échouées sur un compte donnent un seul avertissement par fenêtre, qui devient une alerte à vingt, et un nom ne compte contre un compte que s’il est tapé en entier (« x alice » ne compte plus contre alice).
- Les messages Discord et Slack sont échappés d’abord et coupés ensuite : un nom d’appareil plein de Markdown faisait refuser par Discord toute alerte qui le nommait.
- Une adresse IPv6 publique est acceptée pour une destination personnelle.
- Derrière un proxy qui connecte lui-même les gens (`auth_basic` de nginx), le cookie de FinStats n’est plus refusé.
- Un nom d’affichage contenant `{{url}}` s’affiche tel qu’écrit dans l’aperçu d’une page publiée.
- Un bilan demandé pour l'année 9999 affiche l'année par défaut au lieu d'une erreur.
- Une page en direct dont l'autorisation a été retirée puis rendue pendant qu'elle était ouverte est de nouveau tenue à jour, sans rechargement.

### Removed
- Le **Synnmørsk** ne fait plus partie des langues de FinStats : un navigateur qui l'avait choisi lit le nynorsk, la langue écrite dont il était tiré.

## [2.3.0] - 2026-10-07 to 2026-10-26

Capacité, un canal en direct, le mode simple et les langues

### Added
- **Langues** : FinStats parle 23 langues. L’anglais britannique et américain ainsi que le norvégien bokmål et nynorsk sont écrits et relus par le mainteneur ; le danois, l’allemand, l’espagnol, le finnois, le français, l’hindi, l’islandais, l’italien, le japonais, le coréen, le néerlandais, le polonais, le portugais (Brésil), le russe, le suédois, le turc, l’ukrainien et le chinois (simplifié et traditionnel) sont des traductions que personne ici n’a relues, et disent en anglais ce qui leur manque encore. Chaque page des deux dispositions, le profil public, les notes de version et ce que le serveur répond en toutes lettres (une erreur, les cartes du bilan, ce que fait une tâche Jellyfin) s’affichent dans la langue choisie sous Paramètres → Apparence → Langue, pour chaque navigateur ; tant qu’aucune n’est choisie, c’est la langue du navigateur qui décide. Les nombres, les dates, les unités et les énumérations s’écrivent comme les écrit cette langue. Un script peut en demander une avec `Accept-Language`.
- **Mode simple** : une deuxième disposition pour ceux qui viennent de Plex, organisée comme un serveur de statistiques Plex classique. Une barre en haut au lieu du menu, et seulement les pages de statistiques : Accueil, Médiathèques, Utilisateurs, Historique, Graphiques et Paramètres, avec un engrenage pour votre profil, les paramètres, les notes de version, le thème et la déconnexion. Il se choisit pour chaque navigateur sous Paramètres → Apparence → Disposition. Il ne charge que sa propre moitié de FinStats et démarre donc plus vite ; Bilan, Ensemble, Pipeline, Sécurité et les pages du serveur restent dans la disposition FinStats. Il porte les couleurs de FinStats, en clair comme en sombre, et rien n’y bouge.
- **L’accueil du mode simple** : ce qui est en lecture, une carte par flux (comment il est lu, depuis le domicile ou de l’extérieur, quand il se termine), les statistiques de visionnage en cartes classées par nombre de lectures ou par durée sur les N derniers jours, ce que contient chaque médiathèque et ce qui y est arrivé récemment.
- **Les graphiques du mode simple** : les lectures par type de média (jour par jour, par jour de la semaine et par heure, les dix premières applications et personnes), par mode de lecture (jour par jour, le maximum de flux simultanés de chaque jour, la résolution du fichier et celle envoyée, applications et personnes) et mois par mois, en nombre de lectures ou en durée, pour tout le monde ou pour les personnes choisies.
- **Médiathèques, utilisateurs et titres en mode simple** : des tableaux de chaque médiathèque et de chaque personne avec ce qui a été lu en dernier, sur quoi et d’où (l’adresse seulement avec *Voir les détails réseau*), et une page pour chaque médiathèque, personne et titre avec ses chiffres et son propre historique.
- Pour les scripts : `/api/stats/split` (lectures par application, personne, résolution, jour ou mois, réparties par type de média ou par mode de lecture) ; classements par nombre de spectateurs distincts (`sort=users`) ; activité restreinte aux lectures regardées (`watched=1`) ; analyses des seules parties demandées (`parts=`) ; le pic réparti entre transcodages, streaming direct et lectures directes, jour par jour aussi ; ce que contient chaque médiathèque et ce qu’elle a lu en dernier ; ajouts récents par type (`kind=`) ; la carte de chaleur par type de média ; la dernière lecture d’une personne ; si un flux est lu depuis le domicile.
- **Un canal en direct** : le tableau de bord, les téléchargements de Pipeline et la barre d’état se tiennent à jour par un seul WebSocket par onglet (`/api/live`), si bien que ce qui est en lecture et ce qui se télécharge change à l’écran au moment même où cela change, et non au prochain coup d’œil. Derrière un proxy inverse qui ne transmet pas les mises à niveau WebSocket, ou sur une connexion coupée sans prévenir, les pages interrogent le serveur toutes les quelques secondes, comme avant. `/api/now-playing`, `/api/downloads` et `/api/summary` restent tels quels, pour les scripts.
- **Serveur → Capacité** : ce serveur tiendra-t-il le coup, d’après ce que FinStats a vu ? Les moments les plus chargés jamais enregistrés et ceux des 90 derniers jours : les flux simultanés, et combien d’entre eux avaient leur image transcodée avec accélération matérielle, sans, ou sans que rien ne soit indiqué (un historique importé de Tautulli est celui de Plex et n’est pas pris en compte). Chaque disque qui porte les médiathèques, et la vitesse à laquelle ce que Jellyfin a ajouté en six mois le remplit (un disque rempli plus récemment est mesuré à partir d’un mois après son premier fichier, puisque son premier mois est celui de son remplissage), et, là où Jellyfin 10.11 ou plus récent indique l’espace libre, combien de temps celui-ci durera à ce rythme. FinStats ne dit jamais combien de flux le serveur pourrait supporter.
- **Exporter en .ics** dans Pipeline → À venir : la liste telle que vous l’avez filtrée, téléchargée sous forme de fichier de calendrier à importer une fois. Elle ne se met pas à jour toute seule ; une clé de calendrier créée dans Paramètres → Clés API donne toujours un calendrier qui le fait.
- **À examiner** : Capacité signale quand chaque transcodage dont Jellyfin a rendu compte en 90 jours s’est fait sans accélération matérielle alors que le transcodage est fréquent, quand l’analyse de la médiathèque par Jellyfin occupe un quart du temps ou plus, et quand il reste moins de six mois d’espace sur un disque. Chaque remarque est accompagnée de ses chiffres et peut être écartée comme les constats de Santé de la médiathèque.
- **De quoi est faite votre médiathèque** peut se limiter à une médiathèque ou à plusieurs, dans Médiathèques et sur la page de chaque médiathèque (`library_id=a,b`).
- **Formats de date et d’heure** dans Paramètres → Apparence : une date comme 31.01.2026, 31/01/2026, 31-01-2026, 01/31/2026, 2026-01-31, 2026.01.31 ou 2026/01/31, et une heure sur 24 ou 12 heures, avec ou sans zéro devant, ou chacune comme la langue l’écrit. Gardés par ce navigateur.
- **Un accueil** : la première fois que quelqu’un se connecte, quel que soit le navigateur, et à la fin de la configuration, FinStats demande la langue, les formats de date et d’heure et la mise en page, avec une image de chacune. Passer garde l’anglais britannique, 31/01/2026, 13:05 et FinStats.

### Changed
- Le logo à l’intérieur d’une page est le symbole seul à côté du nom, composé en Inter quelle que soit la police choisie par un préréglage ; là où il apparaît seul (le favicon), il garde sa tuile arrondie.
- Une lecture transcodée de façon logicielle est enregistrée comme telle quand Jellyfin le précise ; auparavant, elle ressemblait à celle d’un serveur qui ne le précise pas. Les pages appellent toujours les deux Logiciel.
- Une mise à jour exécute désormais, une seule fois et en commençant par la version la plus ancienne, ce dont une version a besoin sur une base de données écrite par un FinStats plus ancien, en plus de ses migrations. Celle de 2.3.0 lit les détails du serveur au premier démarrage, pour que Capacité soit disponible tout de suite.

### Performance
- **Une page laissée ouverte ne demande plus rien** : le tableau de bord faisait 30 requêtes par minute pour rester à jour, et les téléchargements de Pipeline 18 ; avec le canal en direct, ils n’en font plus aucune. Quel que soit le nombre d’onglets ouverts, FinStats ne demande rien de plus qu’avant à Jellyfin, Sonarr et Radarr, et un onglet masqué laisse toujours la file de téléchargement ralentir.
- La page d’une personne ne lit plus que les lectures de cette personne : ses totaux et sa dernière lecture étaient calculés sur toutes les lectures du serveur puis restreints à elle, environ quatre secondes sur un million de lectures.
- La page d’une médiathèque ne lit plus que les titres et les lectures de cette médiathèque, là où elle comptait ceux de toutes les médiathèques pour n’en garder qu’une : un tiers du temps sur un million de lectures.
- Avec l’horloge de Jellyfin en avance sur celle de FinStats, *Après l’analyse de la médiathèque par Jellyfin* relisait toute la médiathèque chaque minute jusqu’à ce que l’écart soit rattrapé ; chaque analyse n’est désormais suivie qu’une fois.
- Un fichier GeoIP impossible à ouvrir était recopié (cent mégaoctets et plus) et toutes les adresses relocalisées, tous les quarts d’heure ; il n’est réessayé que lorsqu’il change.
- Santé de la médiathèque s’ouvre immédiatement après que le pointeur s’est posé sur son lien : ce qui avait été chargé à l’avance était rangé là où la page ne regardait jamais.

### Stability
- Une mise à jour qui échouait après l’exécution d’une migration, et que Docker redémarrait aussitôt, prenait à chaque démarrage une nouvelle copie de la base à moitié mise à jour, et trois démarrages suffisaient à supprimer la seule copie que la version précédente savait ouvrir. La copie d’avant la mise à jour est désormais conservée.
- Un titre supprimé dans Jellyfin pendant que FinStats lisait la médiathèque (un fichier remplacé par Radarr) pouvait faire manquer un autre titre, pourtant toujours là, qui était alors marqué comme supprimé ; ses lectures pouvaient ensuite être déplacées vers une autre copie de ce titre. Ce que la lecture de la médiathèque n’a pas vu est désormais d’abord redemandé par son identifiant.
- Restaurer une sauvegarde faite avant qu’un fichier soit renommé ou remplacé par une meilleure version restaurait une seconde fois, et pour de bon, chaque lecture déplacée depuis vers le nouveau fichier.
- Une demande que FinStats ne savait pas lire dans la réponse de Seerr (un champ renommé par une mise à jour de Seerr) était marquée comme supprimée par la lecture complète quotidienne.
- Une tâche qui rencontrait un bogue restait « en cours » pour toujours, et aucun déclencheur ne la relançait avant un redémarrage de FinStats ; elle se termine désormais en échec.
- Un zip Tautulli endommagé interrompait l’envoi avant la suppression du fichier, qui contient les jetons de chaque utilisateur Plex, jusqu’au nettoyage des six heures.
- Une importation ou une restauration interrompue pendant l’arrivée de son fichier laissait ce fichier dans le dossier de données pour toujours, adresses comprises ; il est supprimé au démarrage suivant.
- Une image pouvait être servie à moitié écrite pendant sa mise en cache, et le navigateur la gardait alors ainsi.
- Les clés API révoquées étaient conservées pour toujours, quel que soit le nombre de clés créées et révoquées.
- Une lecture des marques vu et favori de quelqu’un revenue vide ou illisible les effaçait toutes jusqu’à la lecture suivante ; elle est désormais refusée, comme une lecture vide d’une médiathèque.
- Un épisode dont le numéro de fin est impossible pour un épisode (une date écrite là) pouvait occuper sans fin la Santé de la médiathèque après une lecture de la médiathèque.

### Fixed
- Une note de version avec du code en gras (celles de 2.0.2 et 2.0.3) affichait ses accents graves au lieu de mettre le code en forme.
- Un graphique de comptes pouvait numéroter son axe 0, 3, 5, 8, 10, voire 0, 0, 1, 1, 1 : des pas de 2,5 écrits arrondis. Les comptes montent désormais par pas entiers.
- Une clé de calendrier créée par quelqu’un qui n’a pas le droit de gérer FinStats venait avec un lien vers l’adresse vue par le navigateur au lieu de l’adresse publique de FinStats, et ne fonctionnait donc que de l’intérieur ; la page demandait aussi des paramètres qui lui étaient refusés.
- La dernière lecture d’une personne pouvait être trop courte de quelques secondes pour compter comme une lecture ailleurs, et nommer un titre que son Historique n’affiche pas.
- Une restauration depuis Supprimées récemment qui échouait ne disait rien et laissait son bouton tourner ; de même pour un *Rétablir* raté dans Santé de la médiathèque et un *Rouvrir* ou un *Tout résoudre* raté dans Sécurité.
- Le flux de calendrier s’appelle FinStats dans une application de calendrier, et non finstats.
- La file de téléchargement indiquait qui avait demandé chaque téléchargement à quiconque avait le droit de la voir ; la demande de quelqu’un d’autre exige désormais *Voir l’activité de tous*, comme dans Pipeline.
- Retirer *Recevoir des notifications* n’arrêtait pas ce qui était envoyé aux destinations de cette personne, qui ne pouvait plus les désactiver.
- Une lecture mise en pause puis reprise dans le délai de fusion était annoncée une fois comme arrêtée, et sa reprise et sa vraie fin jamais.
- Les lectures d’une piste pouvaient être déplacées vers la piste homonyme d’un autre artiste (« Intro ») quand son album quittait la médiathèque.
- Une destination de notification écrite comme une adresse IPv4 dans de l’IPv6 (NAT64, 6to4) passait pour publique alors qu’elle atteignait le réseau local.
- Une connexion Sonarr ou Radarr redirigée vers un autre serveur continuait d’afficher les affiches de l’ancien serveur pendant une semaine.
- Les alertes de Sécurité disaient que rien ne demandait d’attention quand deux personnes ou plus étaient choisies, et une page vidée par la résolution de sa dernière alerte n’offrait aucun moyen de revenir.
- Chaque clé API non expirée disait qu’elle avait expiré « à l’instant ».
- Une boîte de dialogue restait au-dessus de la page après un retour arrière, et la page ne défilait plus tant qu’elle n’était pas fermée.
- Le journal d’audit affichait huit types d’entrées (une lecture ou une sauvegarde rétablie, la corbeille vidée, les profils publics, une apparence modifiée) par leurs identifiants, sans rien dans Détail.
- Ajouter un titre à la liste à voir depuis son menu contextuel échouait sans un mot quand le serveur refusait.
- *Regardé ensemble* sur le tableau de bord traitait plusieurs personnes choisies à la fois comme une seule.
- Le bilan écrivait les heures et les jours en dessous de dix avec la virgule décimale du navigateur, à côté de ses propres virgules de milliers.
- Capacité disait qu’un disque serait plein "au rythme des 0 derniers mois".
- Sur la page Sécurité, les adresses sans lieu comptaient les lectures de tous depuis toujours, quelle que soit la personne et la période choisies.
- Quelqu’un ayant *Gérer FinStats* mais pas *Voir l’activité de tous* pouvait mettre la lecture de n’importe qui à la corbeille.
- Une carte du Bilan ou d’un profil public pouvait sortir du cache du navigateur dans la langue choisie auparavant.
- Les téléchargements envoyés par le canal en direct étaient en anglais britannique, quelle que soit la langue de la page.
- Une connexion qui changeait ce que quelqu’un peut voir n’atteignait ses autres onglets ouverts qu’à la vérification suivante, toutes les demi-minutes.
- Un onglet qui cédait sa connexion en direct à un plus récent en reprenait une à sa page suivante, si bien que les onglets d’une même personne se fermaient sans cesse les uns les autres.
- Les tableaux triaient comme du texte les tailles de 1 000 à 1 023 unités dans les langues qui groupent les chiffres par un espace, et de même les pourcentages en turc.
- Un titre contenant un espace réservé dans son nom, comme "Big {device} Bunny", était rempli par les messages du serveur.
- Les tableaux triaient comme du texte les durées et les nombres sur un total (5 sur 26) dans toutes les langues sauf l’anglais et les langues nordiques.
- Un onglet encore ouvert dans une langue recevait le serveur dans une autre dès qu’un autre onglet l’avait choisie.
- Un mot à côté d’un nombre prenait la mauvaise forme en polonais, en russe et en ukrainien ("5 wieczory", "2 потоков").
- Capacité comptait le premier remplissage d’une médiathèque comme de la croissance et divisait ce que chaque médiathèque ajoutait par six mois, quelle que soit la période mesurée; un fichier qui prenait la place d’un autre (une mise à niveau de Radarr ou Sonarr, ou un fichier auquel Jellyfin donnait un nouvel id) comptait comme entièrement nouveau, alors que seule la différence l’était.
- Les personnes regardant ensemble avec SyncPlay ne formaient pas un groupe quand l’une rejoignait l’autre quelques minutes après ; les lectures qui finissent en même temps au même endroit vont ensemble désormais, et l’historique est regroupé une fois de plus.
- Quelqu’un sans *Voir l’activité de tous* pouvait déduire de *Jamais regardés* d’une médiathèque qu’une autre personne avait lu un titre ; seul son propre visionnage compte désormais.

## [2.2.0] - 2026-10-01 to 2026-10-07

Santé de la médiathèque et FinUI

### Added
- **Un menu à vous sur ordinateur** : Paramètres → Apparence en propose huit : la barre latérale telle qu’elle était, groupée selon l’usage des pages, un rail d’icônes qui se déplie en barre latérale sous le pointeur, une barre latérale qui commence par la recherche, un dock, une barre de commande, des pages que vous épinglez en haut, et des tuiles colorées. Les barres latérales peuvent se placer à droite. Le dock s’escamote pendant que vous faites défiler vers le bas et revient quand vous remontez ou pointez vers le bas de la fenêtre ; avec lui, la barre d’état passe en haut.
- **Un menu à vous sur téléphone** : une barre d’onglets avec Plus (le nouveau choix par défaut), un panneau qui s’ouvre à mi-hauteur, l’écran entier, les pages déployées en éventail autour du pouce, ou une barre d’adresse qui se transforme en menu. Chacun cherche parmi ses pages et transmet tout autre mot à la recherche.
- Les deux sont conservés par le navigateur où ils ont été choisis, si bien qu’un téléphone et un ordinateur gardent chacun le leur ; ce sont deux composants FinUI, `desktop-nav` et `mobile-nav`.
- **Un menu contextuel** : faites un clic droit sur un titre, une personne ou une médiathèque n’importe où dans FinStats (une affiche du tableau de bord, une ligne de liste, un nom dans un tableau, un film dans À venir qui n’est pas encore dans la médiathèque), ou gardez le doigt dessus, ou appuyez sur la touche menu, et un menu s’ouvre depuis le point où vous avez appuyé : l’ouvrir, l’ouvrir dans un nouvel onglet, ajouter un film ou une série à votre liste à voir, l’ouvrir dans Jellyfin ou sur TMDB et TVDB, copier son lien. Maj + clic droit ouvre toujours le menu du navigateur. C’est le `context-menu` de FinUI.
- **La recherche sans boîte de dialogue** : la recherche se déploie depuis votre menu (une barre latérale s’élargit pour l’accueillir, le dock s’élève, le nom de la barre de commande devient le champ, le menu d’un téléphone la contient) ou, si vous le choisissez dans Paramètres → Apparence, prend la place de la page tandis que le menu reste. Ctrl+Espace l’ouvre comme avant ; la fenêtre par-dessus tout a disparu. C’est le `search` de FinUI.
- **Tout le contenu d’une médiathèque** : la carte Ajouts récents d’une médiathèque propose *Tout son contenu*, chaque film, série ou album qu’elle contient en affiches, par nom, année, date d’ajout ou taille, filtré au fil de la frappe. La suite arrive à mesure que vous faites défiler.
- **Serveur → Santé de la médiathèque** : FinStats compare chaque fichier à ses voisins et liste ce qui sort du lot, chaque fois avec les chiffres qui le justifient. Il se contente de signaler : rien n’est modifié, supprimé ni réanalysé dans Jellyfin.
- **Trous** : des épisodes manquants entre deux fichiers d’une saison (« Saison 2 : les épisodes 4 et 5 manquent entre le 3 et le 6 »). Un fichier qui contient deux épisodes compte pour les deux ; les épisodes spéciaux et la fin d’une saison sont laissés de côté.
- **Saisons qui diffèrent** du reste de leur série par la résolution, le HDR ou le codec, et **épisodes atypiques** : un fichier qui ne ressemble pas au reste de sa saison, comme un épisode SD dans une saison en 1080p. La résolution se juge à la largeur ou à la hauteur, si bien qu’un film recadré en 1920×800 est en 1080p.
- **Copies et versions** : le même film ou épisode en double, selon les identifiants de fournisseur déjà utilisés par la liste à voir et Pipeline. Deux copies dans une même résolution montrent l’espace occupé deux fois ; une copie 4K à côté d’une copie 1080p est listée comme deux versions et n’est jamais comptée comme du gaspillage.
- **Fichiers maigres** : un débit très inférieur à ce que demandent habituellement leur résolution et leur codec (« 4K HEVC à 1,1 Mbit/s ») ; les épisodes maigres d’une saison forment un seul constat.
- **Doublages qui s’arrêtent** : une langue audio que certaines saisons ont sur chaque épisode et d’autres sur aucun.
- **Jamais identifiés** : les films et séries que Jellyfin a associés sans identifiant TMDB, TVDB ni IMDb.
- **Écarter** un constat qui est voulu, avec une note. Il revient de lui-même quand ses fichiers changent. Il faut pour cela pouvoir à la fois gérer FinStats et voir le serveur ; les constats écartés figurent dans le journal d’audit et font partie des sauvegardes.
- La page d’une médiathèque indique combien de choses y sont à examiner, et y mène.
- **FinUI**, les composants dont FinStats est fait, devient un projet à part entière (`github.com/finstats/finui`) : chaque bouton, carte et contrôle, dans tous ses états et les deux thèmes, sur `finui.finstats.no`, et **FinUI create** pour en changer les couleurs, l’arrondi, la densité et plus encore en direct, puis l’emporter chez vous d’une seule commande. **Paramètres → Système** y mène.
- **Paramètres → Apparence** : chacun choisit l’allure que FinStats a pour lui, par le code d’un préréglage FinUI : couleur de base, accent, couleurs des graphiques, contraste, arrondi, densité, bordures, cartes, boutons, champs, tableaux, mise en évidence, animation, icônes, le menu, la page derrière tout cela, l’anneau de focus, la façon dont les titres sont composés, et les polices du texte, des titres et des chiffres. Votre choix n’appartient qu’à vous ; la page montre ce que signifie le code et mène à FinUI create pour le changer.
- **Styles** : onze allures complètes pour partir d’un seul choix dans FinUI create, dont Paramètres → Apparence reprend le code. Ce sont Washi (FinStats tel qu’il est livré), Studio, Noir, Matinee, Console, Pastille, Ledger, Gazette, Arcade, Canopy et Harbour. FinStats lui-même est livré avec une seule allure et ne propose aucune liste de styles.
- **Des icônes qui bougent** : chaque icône d’un lien, d’un bouton ou d’un onglet fait son propre mouvement, une fois, quand on pointe dessus (l’actualisation tourne, la flèche du téléchargement tombe dans son bac, la corbeille soulève son couvercle), et une actualisation sur laquelle on a appuyé continue de tourner jusqu’à l’arrivée de sa réponse. Immobiles avec les animations réduites, et avec Animation : désactivée.
- **Le mouvement, avec FinMotion** : les composants de FinUI bougent désormais : le bouton d’un interrupteur est lancé d’un côté à l’autre, le soulignement d’un onglet glisse jusqu’à l’onglet suivant, un champ refusé secoue la tête, une copie est cochée d’un seul trait, les notifications s’empilent comme une main de cartes et s’ouvrent en éventail quand on pointe dessus, le panneau d’un téléphone s’étire comme un élastique quand on le tire, les lignes d’un tableau trié rejoignent leur place, un chiffre qui change défile comme un compteur kilométrique, et les graphiques se dessinent la première fois qu’on les voit. Tout cela suit le choix Animation de Paramètres → Apparence et reste immobile avec les animations réduites. FinMotion est un projet à part (`github.com/finstats/finmotion`), posé par-dessus FinUI.
- **Plus d’icônes dans FinUI** : 239 désormais, contre 71, chacune avec son propre mouvement (volume plus et moins, un casque et un casque coupé, un crayon qui écrit, une cloche qui sonne, un téléphone qui vibre), et chacune peut être téléchargée depuis la galerie de FinUI en SVG, fixe ou animée.
- **Polices** : trente-quatre familles sous licence libre au choix (Geist, Roboto, Manrope, IBM Plex Sans, Montserrat, Lora, Playfair Display, Fira Code et d’autres), fournies avec FinStats comme Inter, jamais chargées d’ailleurs, et chargées seulement quand elles sont choisies.
- **Plus de bases et d’accents** : Nord, Solarized, Gruvbox, Catppuccin, Rosé Pine, Tokyo Night, Dracula, Everforest et Ocean ; seize accents de plus, dont dix ont une couleur le jour et une autre la nuit. Chaque base a désormais son propre papier le jour : les gris se ressemblaient dans le thème clair.
- **Une suppression s’annule pendant 30 jours** : une lecture ou une sauvegarde supprimée attend dans la corbeille, et **Annuler** dans la notification qui suit la fait revenir. Activité et Paramètres → Sauvegardes listent chacun ce qui a été supprimé récemment, avec la date de disparition définitive, dès qu’il y en a. Une lecture dans la corbeille est exclue de tous les totaux, graphiques, bilans et profils, et une importation ou une restauration ne la fait pas revenir comme nouvelle ; quiconque l’a regardée avec quelqu’un est compté comme seul jusqu’à son retour.

### Changed
- Dark Reader et les extensions du même genre laissent les pages de FinStats tranquilles (`darkreader-lock`) : FinStats dessine lui-même les deux thèmes, et une page repeinte se battait avec eux.
- Sur un téléphone, le menu ne glisse plus depuis la gauche : la barre d’onglets se tient en bas, sauf si un autre menu est choisi.
- FinStats relit votre médiathèque une fois après cette mise à jour, pour savoir où se terminent les fichiers qui contiennent plusieurs épisodes.
- Sur la page d’un titre, **Ajouter à la liste à voir** est aussi haut que **Ouvrir dans Jellyfin** à côté ; il était une taille en dessous.
- Les filtres de période et de personnes restent en haut de l’écran pendant que la page défile, si bien que 7 j devient Tout depuis le bas d’une longue page ; de même pour les filtres des demandes et des sorties à venir de Pipeline.
- Le bouton de retour en haut s’écarte du menu qui occupe le bas de la fenêtre, quel qu’il soit : en bas dans le coin, à côté du dock, juste au-dessus de la barre d’onglets, du bouton du pouce ou de la barre d’adresse d’un téléphone, et au-dessus de la barre d’état sinon. C’est le `to-top` de FinUI.
- Supprimer une lecture ou une sauvegarde la met dans la corbeille au lieu de l’effacer aussitôt (`PUT` avec `{"deleted": true}` ; `DELETE` disparaît de l’API). Les sauvegardes au-delà du nombre conservé sont toujours supprimées définitivement.
- Les **Notes de version** tiennent sur une seule feuille au lieu d’une pile de cartes : chaque note est une ligne à part, précédée dans une colonne de son type (Ajouts, Modifications, Corrections, etc.) et de l’icône de ce type, sous un intitulé par série. La liste des versions reste à droite.

### Performance
- Ouvrir FinStats ne télécharge plus que ce qui a changé : une feuille de style, un script ou une police que le navigateur possède déjà est confirmé en quelques octets au lieu d’être renvoyé. Chaque visite rechargeait en entier toutes les feuilles de style et tous les modules de FinStats, alors que chacun portait une étiquette pour les distinguer.

### Stability
- Restaurer une sauvegarde lit les paramètres uniquement dans la ligne des paramètres : un fichier venant d’un FinStats plus récent, qui y range davantage, voyait ses paramètres abandonnés sans un mot.
- Un onglet ouvert ne grossit plus à chaque page visitée : chaque page quittée restait en mémoire derrière celle affichée (environ 2 600 éléments par visite), retenue par la liste de ce que FinMotion anime et par l’écouteur de défilement de la barre de filtres, jusqu’à la fermeture de l’onglet.
- Une commande mal tapée (`finstats bakcup`), ou `restore` sans son fichier, ne crée ni ne met plus à niveau une base de données avant de dire qu’elle ne sait pas quoi faire.

### Fixed
- Le petit graphique d’une tuile ne déborde plus de la tuile quand le nombre à côté est large (« 56 j 23 h »), ni quand quatre tuiles partagent une ligne plus étroite : le graphique cède la place et le nombre garde sa largeur.
- Dans le thème clair, un bouton principal devenait violet sous le pointeur ; il garde désormais la couleur du sceau.
- Dans Paramètres, un interrupteur est aligné avec le nom et l’aide de son réglage ; il pendait en haut de la ligne.
- Dans une liste classée sans rien sous un nom, comme Utilisateurs les plus actifs sur le tableau de bord, le nom se plaçait en haut de sa ligne à côté d’une image centrée ; il est désormais centré.
- Une rangée de tuiles statistiques à une largeur entre téléphone et ordinateur (quatre tuiles sur trois colonnes) laissait la dernière seule sur sa ligne ; quatre tuiles se placent désormais par quatre, deux ou une, et trois par trois ou une.
- La lecture des détails du serveur comptait un appareil sur lequel personne ne s’était connecté, alors qu’elle ne le garde pas : la tâche annonçait trois appareils là où la page Serveur en listait deux.
- Une seconde, une minute, une heure : les alertes disaient « 1 secondes d’écart » et « dans 1 minutes », et une tâche Jellyfin à intervalle d’une minute s’exécutait « toutes les 1 minutes ».
- `finstats --help` nomme chaque variable que lit FinStats ; `FINSTATS_PUBLIC_IP_URL`, `FINSTATS_ALLOW_LIBRARY_SHRINK` et `FINSTATS_SKIP_PREUPDATE_BACKUP` manquaient.
- L’assistant de configuration disait d’une adresse qui n’en était pas une « connexion impossible », dans une phrase qui se prolongeait dans la suivante ; il dit désormais que ce n’est pas une adresse.
- Dans Paramètres → Sauvegardes, une question posée pendant l’écriture d’une sauvegarde disparaissait à la fin de la sauvegarde.
- Avec le menu en rail dépliable, fermer la recherche montrait en même temps la recherche ouverte et le rail fermé, puis laissait le rail ouvert jusqu’à ce que le pointeur vienne et reparte. Échap rend désormais le focus du clavier là où il était à l’ouverture de la recherche, et abandonne celui laissé par un clic.
- Sur un téléphone, fermer la recherche gardait la page assombrie jusqu’à la disparition de la recherche, puis repassait au clair d’un coup ; l’assombrissement s’estompe désormais avec elle.
- Les étapes de l’importation Tautulli disaient que la sauvegarde se télécharge après **Backup Database**. Tautulli l’enregistre sur sa propre machine : les étapes disent désormais où (son dossier `backups`, `/config` avec Docker), quel fichier prendre, et qu’il faut le copier.
- Une recherche fermée avant d’avoir fini de s’ouvrir privait le rail dépliable de son animation : il s’ouvrait et se fermait ensuite d’un coup.
- Le rail dépliable restait ouvert après le choix d’une page jusqu’à un clic ailleurs et, sur certains navigateurs ou avec un zoom, dessinait une barre de défilement sous ses icônes. Il ne s’ouvre plus que pour le pointeur et le clavier, et ne dessine aucune barre de défilement une fois fermé.
- Sur la page Sécurité, une adresse que la base situe dans un pays mais pas sur la carte compte pour ce pays, et non parmi les adresses sans lieu connu.

## [2.1.2] - 2026-10-01

### Added
- **Importer depuis Tautulli** : rapatriez votre historique Plex. Envoyez la sauvegarde de base de données de Tautulli (le `.db`, ou le `.zip` qui le contient) sous **Paramètres → Importation** ; ses lectures ont leur propre entrée dans le filtre *Enregistré par* d’Activité.
- **Un tableau de câblage pour dire qui est qui** : les utilisateurs Plex d’un côté, les utilisateurs Jellyfin de l’autre. Tirez un câble de chaque utilisateur Plex vers la personne qu’il est aujourd’hui, ou cliquez sur l’un puis sur l’autre, ou utilisez le clavier. Toute personne laissée sans câble n’est pas importée, et deux comptes Plex peuvent aboutir à une seule personne.
- Les films et les épisodes sont associés à votre médiathèque par leur nom (un film par son titre et son année, un épisode par sa série et son numéro), et celui qui n’est pas encore sur votre serveur est gardé et rattaché à son arrivée. Un visionnage repris forme une seule lecture ; la musique n’est pas importée.
- La sauvegarde contient les jetons d’accès de Plex : FinStats ne les lit jamais, et supprime le fichier dès que vous importez ou recommencez.
- **Médias non liés** sous **Paramètres → Données** : chaque titre venu de Tautulli, Jellystat ou Streamystats qui ne correspond à rien dans votre médiathèque, avec sa provenance et l’endroit où il se trouve le plus probablement déjà. **Localiser** le relie à partir des meilleures suppositions ou d’une recherche (un film peut être localisé comme épisode spécial d’une série), et FinStats retient le choix, si bien que le même historique importé à nouveau se relie tout seul. Les choix font partie des sauvegardes.

### Performance
- **Le bilan s’ouvre trois à cinq fois plus vite** : les acteurs et réalisateurs les plus vus de l’année étaient calculés en reparcourant l’année une fois pour chaque personne qui y figurait. Mesuré sur un historique réel, une année est passée de 0,29 s à 0,09 s et les 12 derniers mois de 0,53 s à 0,12 s.

### Fixed
- **L’historique importé retrouve les titres connus sous un autre nom** : une lecture est aussi comparée au titre en langue originale, si bien qu’une série que Plex ou Tautulli connaissait sous le nom 오징어 게임 se rattache à Squid Game, et *Im Westen nichts Neues* à *All Quiet on the Western Front*. Les titres originaux de votre médiathèque sont lus à sa prochaine analyse.
- Les noms sont comparés comme la recherche les compare (casse, accents, ponctuation, année écrite dans le nom, « The » initial), et une année de sortie peut avoir un an d’écart, si bien que « Star Wars: Episode I - The Phantom Menace » trouve « … – The Phantom Menace » et qu’un Kingsman de 2014 trouve celui de 2015. Un titre connu sous un tout autre nom n’est toujours jamais deviné.
- Une importation Streamystats ou une restauration ratée était annoncée dans les notifications sous son nom interne.
- Le bouton *Partager l’année en cartes* du bilan et le rang de spectateur à côté n’avaient pas d’espace entre eux.

## [2.1.1] - 2026-10-01

### Changed
- **La liste à voir est proposée partout où se trouve un titre** : sur les cartes À venir du tableau de bord, comme dans Pipeline → À venir, et sur la page d’un épisode ou d’une saison, ce qui ajoute sa série à la liste.
- Sur une affiche du tableau de bord, le bouton de la liste à voir apparaît quand vous pointez la carte ou l’atteignez au clavier, au lieu de couvrir le coin de chaque affiche ; sur un écran tactile, il est toujours là.

### Stability
- Enregistrer une connexion Sonarr, Radarr ou Seerr pendant que quelqu’un avait **Paramètres → Connexions sortantes** ouvert pouvait laisser les deux s’attendre l’un l’autre pour toujours.
- **Des paramètres illisibles arrêtent FinStats au démarrage**, en nommant le paramètre fautif, au lieu de remettre discrètement chaque paramètre à sa valeur par défaut.
- **Deux modifications simultanées des paramètres** (la page des paramètres et la planification d’une tâche, par exemple) ne s’annulent plus l’une l’autre.
- Une pause, un saut ou un changement de piste vu pendant que la base de données était occupée est écrit au passage suivant au lieu d’être perdu.
- La lecture de la médiathèque n’échoue plus sur une liste à voir contenant plusieurs entrées qui se révèlent être un seul titre.

### Fixed
- **Un fichier remplacé n’est pas « désormais sur le serveur »** : remplacer le fichier d’un film par une meilleure version ne fait plus croire à ceux qui l’ont dans leur liste à voir qu’il vient d’arriver.
- Un seul échec de lecture des extensions, des tâches planifiées ou du stockage de Jellyfin ne vide plus cette liste sur la page Serveur jusqu’à la lecture suivante.
- **Une lecture Streamystats s’arrête là où elle s’est arrêtée, sur chaque page** : la courbe d’abandon d’un titre et le « terminé » d’un épisode se fondaient plutôt sur sa durée, et la comptaient comme estimée.

## [2.1.0] - 2026-09-29 to 2026-09-30

Liste à voir

### Added
- **Une liste à voir rien qu’à vous** : ajoutez-y un film ou une série depuis sa page, depuis À venir, depuis Ajouts récents ou depuis la recherche (Ctrl+Entrée), et retrouvez-la sous **Liste à voir** sur votre propre profil.
- **Chaque entrée dit où elle en est** : sur le serveur, « 5 épisodes sur 26 », demandée, « À venir vendredi », pas sur le serveur, a quitté la médiathèque, ou vue. C’est calculé à chaque consultation, avec la même définition de « vu » que la progression de vos séries. Les titres vus restent, dans un groupe à part, jusqu’à ce que vous les retiriez.
- **Un titre qui n’est pas encore sur votre serveur** peut entrer dans la liste directement depuis ce qu’attendent Sonarr ou Radarr. Il est rattaché au titre de lui-même à son arrivée, et un fichier renommé emporte son entrée avec lui.
- **« Dans votre liste à voir : désormais disponible »** : une notification envoyée à vos propres destinations quand quelque chose que vous attendiez arrive. Désactivée tant que vous ne la cochez pas.
- **Personne d’autre ne voit votre liste à voir**, pas même un administrateur, et sa notification ne va qu’à vos destinations. Rien n’est écrit dans Jellyfin, Seerr, Sonarr ou Radarr.
- Les listes à voir font partie des sauvegardes, et une restauration les fusionne sans rien doubler.

### Changed
- Les entrées de `/api/upcoming` portent les identifiants TVDB et TMDB de leur titre, et `/api/auth/me` indique si le serveur gère les listes à voir.

## [2.0.4] - 2026-09-29

### Added
- **La planification des tâches, à la manière de Jellyfin** : chaque tâche que FinStats exécute de lui-même a des déclencheurs (chaque jour à une heure donnée, chaque semaine un jour donné, à intervalle, au démarrage, ou après l’analyse de la médiathèque par Jellyfin), chacun avec une durée maximale facultative. Cliquez sur une tâche pour voir et modifier sa planification ; *Ajouter un déclencheur* ouvre la même boîte de dialogue que dans Jellyfin.
- **Tâches devient une section à part** dans Paramètres, avec la dernière exécution de chaque tâche en toutes lettres (« Dernière exécution il y a 38 minutes, en 6 minutes ») et un bouton pour la lancer ; elle se trouvait auparavant en haut de Système.
- **Modifications des métadonnées** : toutes les heures, FinStats lit ce que Jellyfin a enregistré depuis la dernière fois (noms, résumés, genres, notes, distribution et équipe, détails des fichiers et images), si bien qu’une modification dans Jellyfin apparaît sans attendre une analyse de la médiathèque.
- La sauvegarde et la base de géolocalisation peuvent aussi être lancées depuis Tâches ; Échap ramène de la planification d’une tâche à la liste.

### Changed
- **Le thème est un curseur à trois positions**, Clair · Appareil · Sombre, à la place du bouton qui passait de l’un à l’autre : cliquez sur un tiers du curseur, faites glisser le bouton ou utilisez les flèches. Le bouton porte l’icône du choix, qui apparaît en mouvement quand il change.
- **La déconnexion se trouve sous Paramètres → Compte**, qui indique qui est connecté ici ; la barre latérale garde le curseur de thème.
- **Les anciens réglages de planification sont désormais des déclencheurs** : *Suivre l’analyse de la médiathèque de Jellyfin* avec son intervalle de secours, *Sauvegarder tous les N jours* et l’interrupteur GeoIP *Tenir la base à jour* sont devenus les déclencheurs correspondants de leurs tâches, avec les valeurs que vous aviez. Collecte, Sauvegardes et Sécurité renvoient désormais à la planification.
- Un démarrage ne relance plus toutes les tâches : la fin de la dernière exécution de chacune est conservée.
- Une tâche Jellyfin sans estimation pour l’instant affiche « encore environ … », les points défilant là où viendra la durée, au lieu de « ETA… ».

### Performance
- **La carte de Sécurité ne rame plus** : faire glisser et zoomer déplace la carte déjà dessinée, qui n’est redessinée qu’une fois le geste terminé, et la pulsation des lectures en cours ne fait plus repeindre toute la carte 120 fois par seconde quand personne n’y touche. Un glissement est passé de 34 à 60 images par seconde sur un ordinateur lent.

### Fixed
- **Une affiche ou un portrait remplacé dans Jellyfin apparaît dans FinStats** : les images restaient en cache une semaine sur le serveur et une autre dans le navigateur, quoi qu’ait fait Jellyfin depuis. Elles sont désormais mises en cache sous l’étiquette d’image de Jellyfin, si bien qu’une nouvelle image est un nouveau fichier, et le navigateur vérifie que sa copie est à jour chaque fois qu’il en affiche une. Remplacer un portrait n’enregistre à nouveau que la personne, c’est pourquoi Modifications des métadonnées lit aussi les personnes.
- **Serveur → Tâches affiche une estimation dès l’ouverture de la page** : le temps restant n’était mesuré qu’à partir des relevés faits par la page elle-même pendant qu’elle était ouverte, si bien que chaque visite commençait sur « ETA… » jusqu’à avoir vu le pourcentage bouger. FinStats chronomètre désormais lui-même une tâche Jellyfin en cours, toutes les 10 secondes tant qu’elle tourne, et la page reçoit l’estimation. Une tâche que Jellyfin indique à 0 % tout du long (la première phase d’une analyse de la médiathèque) garde ses points tant qu’elle n’avance pas : rien n’est deviné.
- **Les infobulles de la carte apparaissent sur le point** : dans Firefox, l’infobulle du point d’une lecture en cours pouvait atterrir loin de lui, et là où des points se chevauchent (quelqu’un qui regarde depuis le domicile), l’infobulle affichée dépendait d’un pixel. Une seule infobulle nomme désormais tout ce qui se trouve sous le pointeur.
- **La barre d’état a une seule forme de survol** : chaque lien (les flux, Dépôt, la version) s’éclaire comme la même petite boîte, et les points entre les éléments sont centrés dans leurs intervalles. Survoler la version éclairait aussi le point et l’intervalle qui la précédaient.
- **Un chemin de fichier prend toute la largeur de sa carte** : sur la carte Fichier d’un titre, et pour les dossiers de Jellyfin sur la page Serveur, un chemin était serré dans une seule cellule étroite et s’étirait sur neuf lignes à côté d’un espace vide.
- Une boîte de dialogue en train de disparaître n’avale plus un clic sur la page en dessous.

## [2.0.3] - 2026-09-28

### Security
- La page Serveur donnait l’identifiant de chaque appareil à quiconque avait *Voir le serveur* ; les identifiants exigent aussi *Voir les détails réseau*, comme partout ailleurs.
- Deux interrupteurs d’autorisation basculés avant que le premier soit enregistré envoyaient deux fois la même liste de départ, et le second annulait le premier : une autorisation retirée restait accordée, une section de profil public désactivée restait publiée, alors que les deux interrupteurs affichaient ce qui avait été choisi.

### Stability
- **Une seule importation ou restauration à la fois** : deux envois lancés ensemble passaient tous deux la vérification, et une restauration ne tenait jamais compte des importations ; la seconde attendait la base de données et échouait après plusieurs minutes d’envoi.
- **Une adresse contenant un échappement `%` invalide ouvre la page introuvable**, au lieu de laisser un indicateur de chargement tourner pour toujours.
- **Un numéro de page trop grand dans une liste donne une page vide** : il débordait et ramenait à la première page.

### Fixed
- **Les tableaux trient correctement les nombres dans un navigateur allemand ou norvégien** : « 1,204 » était lu comme 1 virgule 204 et non comme mille deux cent quatre, et tout ce qui dépassait le million comme du texte.
- **La vue Tableau d’un graphique trie les dates, jours de la semaine et positions dans l’ordre du temps**, et non par leurs lettres.
- **L’historique des téléchargements indique d’où viennent les importations** : Sonarr et Radarr ne nomment l’indexeur et le protocole qu’au moment de la récupération, si bien que ces ventilations étaient toujours vides. Un pack de saison en échec est un seul échec, et non un par épisode.
- **Supprimer une lecture regardée ensemble** recalcule les groupes de ce titre, au lieu de laisser un « groupe » d’une seule personne.
- **Le temps passé ensemble, pour une personne, ne dépasse jamais le temps qu’elle est restée** : la page Ensemble lui attribuait celui de toute la soirée.
- **Une soirée en compagnie à cheval sur minuit le soir du Nouvel An** (ou sur la limite des « N derniers jours ») appartient tout entière à la période où elle a commencé, au lieu de faire deux soirées en solitaire.
- **Les lectures d’épisodes importées portent le nom de l’épisode** là où Jellystat n’avait gardé que celui de la série.
- **La géolocalisation trouve la base des villes à côté des bases Country et ASN** qu’écrit `geoipupdate`, au lieu de se désactiver.
- **Le passage le plus rembobiné suit les filtres de la page**, comme ses décomptes.
- **« Votre année est prête »** ne va qu’aux personnes dont le bilan contient quelque chose.
- **Résoudre à nouveau une alerte mise en sourdine garde sa paire en sourdine** ; seule la réouverture annule une mise en sourdine.
- **Une clé API expirée ne compte plus dans les vingt.**
- **La limite de sessions libère la session la plus ancienne** même quand de nombreuses connexions arrivent dans la même seconde.
- **Une adresse du domicile trouvée par la recherche automatique reste** quand la même est retirée de la liste saisie à la main.
- **Des images dans le journal d’audit et la liste des clés**, là où il n’y avait que des initiales.
- **La liste des notifications d’un administrateur indique à qui appartient une destination personnelle**, au lieu de « La vôtre » pour tout le monde ; **Envoyés récemment** affiche chaque destination de son dernier événement.
- **Échap sur un titre ou une personne ne quitte jamais FinStats**, et ne ferme que la boîte de dialogue du dessus.
- **Une page s’affiche même quand un enregistrement annule le préchargement qu’elle attendait.**
- **En cours de lecture montre un spectateur en pause qui se déplace dans la lecture** là où il se trouve maintenant, et non quand il appuie sur lecture.
- **Supprimer une connexion met à jour les pages disponibles**, comme le fait son enregistrement.
- **La date de compilation de la base de géolocalisation garde son année** en anglais américain.

## [2.0.2] - 2026-09-28

### Security
- Derrière un proxy, la limite de connexions comptait la première adresse de `X-Forwarded-For`, celle qu’envoie le client : une nouvelle adresse inventée à chaque tentative, et la limite ne s’appliquait jamais. C’est désormais la dernière, celle qu’a écrite le proxy.
- *Inclure les adresses IP et les lieux* envoyait des adresses à une destination personnelle dont le propriétaire n’a pas *Voir les détails réseau*, alors qu’aucune page ne les lui aurait montrées. Les connexions échouées de quelqu’un d’autre exigent désormais aussi *Voir le serveur*, comme sur la page Sécurité.

### Stability
- **Un arrêt n’est jamais perdu** : quand une lecture de la médiathèque était refusée dans les premiers instants après le démarrage, avant que le serveur ne soit prêt à l’entendre, le refus était noté et FinStats continuait au lieu de s’arrêter.
- **Un Jellyfin qui répond sous une forme que FinStats ne sait pas lire est refusé**, pas cru : tous les utilisateurs, ou tous les titres d’une médiathèque, arrivant sous des clés inconnues (`id` pour `Id`) passaient la garde avec un compte complet puis étaient marqués comme supprimés.
- **La fin d’une lecture n’est jamais abandonnée** : quand la base de données restait occupée au-delà de son délai d’attente (une importation la monopolisant), une lecture terminée à ce moment-là restait marquée en cours jusqu’au redémarrage suivant. Elle est désormais écrite au passage suivant.
- **Les paramètres qu’une restauration ne sait pas lire sont laissés tels quels** : une sauvegarde dont les paramètres ne correspondaient plus à cette version remettait à zéro chaque paramètre à la restauration : délai de fusion, autorisations, adresses du domicile, planification.
- **Un caractère de contrôle dans un nom ou un titre ne casse plus les cartes** : la carte de profil, chaque carte d’histoire qui montrait ce titre et leur ZIP répondaient par une erreur.

### Fixed
- **Les notifications s’arrêtent pour quelqu’un de désactivé ou supprimé dans Jellyfin** : ses destinations personnelles continuaient de recevoir tout ce que ses anciennes autorisations permettaient.
- **Une notification en attente est revérifiée avant d’être envoyée** : une destination désactivée, ou un propriétaire qui avait perdu une autorisation, recevait encore ce qui attendait d’être renvoyé.
- **Une destination qui répond toujours `Retry-After` est abandonnée** après cinq tentatives comme toutes les autres, au lieu d’être relancée pendant trente jours.
- **Les identifiants d’appareil sur la page d’une personne** ne s’affichent qu’avec *Voir les détails réseau*, comme partout ailleurs.
- **Une année publiée attend un jour pour tout** : ses séries terminées et abandonnées suivaient chaque lecture, si bien qu’appuyer sur lecture sur une série abandonnée changeait la page publique en quelques secondes.
- **Une série demandée compte comme regardée** quand l’un de ses épisodes l’est : chaque série demandée passait pour jamais regardée, dans le chapitre Demandes et sur la carte Demandes.
- **Le chapitre Ensemble du bilan exclut la TV en direct**, comme le reste de l’année : une chaîne laissée allumée ensemble pouvait devenir le titre principal, et la part pouvait dépasser 100 %.
- **La carte Jours des « 12 derniers mois » se termine sur le jour le plus récent** : la grille perdait les trois ou quatre dernières semaines.
- **Les nouveaux épisodes d’une série sont annoncés une fois par soirée** : les épisodes de part et d’autre de minuit UTC étaient annoncés deux fois sur un serveur à l’est ou à l’ouest de ce fuseau.
- **Connecter Seerr n’annonce pas les anciennes demandes comme prêtes à regarder** : une demande déjà disponible lors de sa première lecture, sans autre trace de sa date, passait pour arrivée à cet instant.
- **Les titres d’une médiathèque supprimée dans Jellyfin partent avec elle** : ils restaient dans la recherche, dans la rangée Ajouts récents et dans les totaux.
- **Un remux reste en streaming direct** quand un relevé perd un instant ses détails de transcodage (un saut), au lieu de devenir un transcodage pour le reste de la lecture.
- **« Jellyfin est hors service » signifie des lectures échouées à la suite** : des délais d’attente isolés, à plusieurs jours d’écart, sur un serveur au repos sur le WebSocket finissaient par s’additionner.
- **Une importation Streamystats affiche sa progression sur sa propre carte**, et non par-dessus le dernier résultat de l’importation Jellystat.
- **Une adresse dont l’hôte commence par « web »** (`http://web:8096`) est lue comme un hôte, et non tronquée comme s’il s’agissait du chemin du client web de Jellyfin.
- **Une erreur de file masque tout l’identifiant de connexion** d’une adresse, même quand le mot de passe contient une barre oblique.

## [2.0.1] - 2026-09-28

### Changed
- **Les langues d’un titre ne prennent plus de place** : l’en-tête nomme les trois premières langues audio et de sous-titres, une ligne pour chacune, et une nouvelle carte **Langues** les liste toutes : les complètes ensemble, les partielles avec le nombre d’épisodes qui les ont. Les deux douzaines de langues de sous-titres d’une longue série repoussaient l’illustration d’un écran vers le bas.

### Fixed
- **L’affiche d’un titre reste à côté de son nom** : quand le texte à côté était plus long que l’affiche (un long synopsis, de nombreux studios), l’affiche glissait en bas de l’en-tête, sous une illustration vide.
- **Une version que FinStats ne sait pas lire est nommée comme telle** : un `app_version` modifié à la main en quelque chose d’illisible faisait refuser à FinStats de démarrer au motif d’être « l’ancien FinStats 2.0.1 », ce qu’il n’était pas. Il refuse toujours, et dit désormais quelle valeur il n’a pas su lire et comment l’écrire.

## [2.0.0] - 2026-09-25 to 2026-09-28

Bilan 2026, profils publics et visionnage à plusieurs

### Added
- **Avec qui vous avez regardé**, dans le bilan : les soirées en compagnie, le temps passé ainsi, le titre qui a le plus réuni les gens, et jusqu’à trois compagnons, nommés dans l’application et reliés à leur profil.
- **Les séries vues jusqu’au bout**, dans le bilan : le dernier épisode de chacune vu dans l’année, selon la même définition de « vu » que les barres de progression du profil, ainsi que celles commencées puis laissées de côté, vues à moins de la moitié et sans aucune lecture depuis deux mois.
- **Ce que vous avez demandé**, dans le bilan : les demandes Seerr faites dans l’année, combien sont arrivées et combien vous avez ensuite regardées. Seulement là où des demandes existent.
- **L’année face à la précédente** : heures, lectures et jours, et de combien c’est plus ou moins.
- **L’année comme une story** : chaque chapitre sous forme de carte 1080×1920, le format des stories sur téléphone (l’année, les chiffres, les séries, films et musiques phares, les genres, le profil de spectateur, les heures et les jours, chaque jour de l’année, les records, la compagnie, les séries terminées, les demandes et l’année précédente). Enregistrez-en une, ou toutes dans un ZIP, depuis « Partager l’année en cartes ».
- **Une carte ne nomme jamais personne d’autre** : les cartes sont dessinées à partir d’une copie de l’année qui n’a aucun champ pour une autre personne, un rang ou une application. Dans l’application, une carte ne porte aucun nom.
- **L’année du serveur**, pour les administrateurs Jellyfin : chaque titre et chaque total, et le profil de spectateur de la maison, sans classer ni nommer personne. Un interrupteur sur la page du bilan, `scope=server` dans l’API.
- **« Votre bilan de l’année est prêt »** : une notification en décembre, une fois, à chaque personne qui a regardé quelque chose dans l’année.
- **Profils publics** : chacun peut publier une partie de son profil à un lien que n’importe qui peut ouvrir sans compte (totaux et titres phares, séries de jours et moments de visionnage, toute l’année sous forme de story en cartes, les dix dernières lectures), chaque partie restant désactivée tant que son propriétaire ne l’active pas sous Paramètres → Profil public. Appareils, adresses, applications, modes de lecture, chemins de fichiers et autres personnes ne sont jamais publiés, quoi qu’on active, et le nom affiché est celui que le propriétaire saisit, jamais son identifiant Jellyfin.
- **Rien sur une page publiée n’a moins d’un jour**, lectures récentes comme totaux, si bien qu’une page que quelqu’un surveille ne dit jamais qui regarde en ce moment.
- **Des cartes à partager** : chaque profil publié est accompagné d’une image en 1200×630, du profil et de l’année, que Discord, Signal, Mastodon et les autres affichent quand le lien est collé, et qui peut être téléchargée pour être publiée. Dessinée sur le serveur à partir de ce que montre la page, et de rien d’autre.
- **Un lien que l’on peut reprendre** : 22 caractères aléatoires, réinitialisés d’un clic ; l’ancien lien répond alors exactement comme un lien qui n’a jamais existé.
- **Les profils publics sont désactivés tant qu’un administrateur ne les autorise pas** : un interrupteur pour tout le serveur, une liste de qui publie quoi, et un moyen de retirer un profil. Publications, modifications, réinitialisations et retraits figurent dans le journal d’audit, et une clé ne peut rien rendre public.
- **Ensemble**, une page à part entière : qui regarde avec qui, combien, et comment cela a évolué. Le temps passé ensemble, les soirées et la part du temps de visionnage passée en compagnie, chacun comparé à la période précédente ; les heures en compagnie face aux heures en solitaire par jour ou par semaine ; chaque paire de personnes avec ses soirées, son temps passé ensemble, sa dernière séance commune et ce qu’elle regarde le plus ; les titres les plus regardés ensemble ; les soirées récentes ; et, pour qui peut voir tout le monde, le temps de chacun en solitaire face à son temps en compagnie.
- Chaque paire compte : une soirée à trois compte pour chacune de ses trois paires, là où la carte ne comptait que le groupe exact de personnes. Le temps passé ensemble d’une paire est le plus court des deux temps de présence.
- La carte **Regardé ensemble** du tableau de bord et d’un profil renvoie à la page, et celle d’un profil indique quelle part du temps de visionnage de cette personne s’est passée en compagnie.
- `GET /api/stats/groups` gagne `pairs`, `series`, `people`, `previous`, `bucket`, ainsi que `watch_s` et `share` dans ses totaux ; les clés existantes sont inchangées.
- **Où l’on s’arrête**, sur chaque page de film et d’épisode : la part des lectures encore en cours à chaque minute, de tout le monde au début jusqu’à ceux qui ont atteint la fin, avec les retours en arrière et les activations de sous-titres de ces lectures marqués sur le même axe. La carte indique combien d’arrêts ont été mesurés (lectures enregistrées par FinStats) et combien estimés (lectures importées, d’après leur durée), et attend trois lectures avant de dessiner quoi que ce soit.
- **Qui continue de regarder**, sur la page d’une série : combien de personnes ont commencé chaque épisode, dans l’ordre, et une colonne **Terminé** par épisode. « Tout le monde abandonne à l’épisode trois », ce sont des gens qui n’appuient jamais sur lecture pour l’épisode quatre.
- **Fichiers qui ne se lisent jamais**, sur Lecture : un film ou un épisode lancé trois fois ou plus qui n’a jamais dépassé trente secondes, avec qui a essayé et sur quelles applications. Le journal de Jellyfin ne contient aucune erreur de lecture, ce sont donc les lectures qui font office de témoin ; la durée minimale d’une lecture ne s’applique pas ici, puisque c’est précisément ce que ces lectures n’atteignent jamais.
- **Les plus rembobinés** et **Sous-titres activés**, sur Lecture : les titres classés par retours en arrière par lecture, avec la minute où ils se concentrent, et par part des lectures où les sous-titres ont été activés dans les dix premières minutes. Les trois listes exigent *Voir l’activité de tous*. Les sauts de moins de 20 secondes n’ont jamais été enregistrés, si bien que réécouter brièvement une réplique est invisible par conception.
- Un saut garde son point de départ sous forme de nombre à côté de son libellé (`from_s` dans la chronologie d’une lecture). Chaque saut déjà enregistré tire son nombre du libellé, tout comme une sauvegarde restaurée antérieure à ce changement.
- `GET /api/stats/files` pour les trois listes ; `GET /api/items/{id}` gagne `insights`, et les épisodes d’une série gagnent `users` et `finished`.
- **Clés API** : toute personne connectée peut créer des clés pour elle-même sous Paramètres → Clés API, pour un script, un tableau de bord ou un téléphone. Une clé s’envoie sous la forme `Authorization: Bearer fs_…`, n’est affichée qu’une fois, est stockée hachée, peut avoir une date d’expiration et se révoque d’un clic ; un administrateur voit et peut révoquer celles de tout le monde. Elle correspond à la même personne avec les mêmes autorisations qu’une connexion, relues en direct à chaque requête, si bien que perdre le droit de se connecter arrête toutes les clés à la fois. Une clé ne peut ni créer ni révoquer de clés.
- **Un flux de calendrier** : `GET /api/calendar.ics?key=…` présente ce que Sonarr et Radarr ont à venir, sous forme d’iCalendar auquel un téléphone peut s’abonner. Une clé de portée *calendrier* ouvre le flux et rien d’autre ; le flux lit la clé dans l’adresse, jamais dans le cookie, parce qu’un calendrier abonné ne peut envoyer aucun en-tête, et il ne nomme personne.
- **Un journal d’audit** sous Serveur → Audit, pour les administrateurs Jellyfin : chaque connexion et chaque tentative échouée avec son adresse, chaque paramètre ou autorisation modifié et sa nouvelle valeur, chaque clé créée, utilisée ou révoquée, chaque connexion à un service, destination, sauvegarde et importation, chaque lecture supprimée et alerte résolue, avec qui, d’où, par quelle clé, et si cela a réussi. Conservé un an, inclus dans les sauvegardes. `GET /api/audit` permet de le parcourir page par page.
- **Importer depuis Streamystats**, dans sa propre section de Paramètres à côté de Jellystat. Prenez son fichier **Settings → Backup & Import → Download Backup** et déposez-le : des années d’historique arrivent en quelques secondes, et rien de ce que vous avez déjà n’est compté deux fois. `finstats import-streamystats <file>` le fait sans interface, `POST /api/import/streamystats` par l’API ; une seule importation à la fois, quel que soit son type.
- **Vous utilisiez les deux outils de suivi ? Importez les deux fichiers.** Une lecture est reconnue comme déjà présente dans FinStats grâce à l’identifiant que lui donne l’outil de suivi et, à défaut, si la même personne a regardé le même élément avec l’une des deux extrémités de la lecture proche d’une lecture de l’historique, si bien qu’un export Jellystat et un export Streamystats des mêmes soirées peuvent tous deux être importés, dans n’importe quel ordre, et qu’une soirée enregistrée par FinStats lui-même n’est pas non plus réimportée par-dessus. « Proche » correspond au délai de fusion sous Paramètres → Collecte.
- Les lignes importées disent ce qu’elles ne savent pas dire. Une lecture que Streamystats a enregistrée lui-même ne garde rien du fichier (ni codec, ni résolution, ni débit, ni conteneur) : ces champs restent donc vides au lieu d’être remplis au jugé, et les cartes qui les afficheraient se masquent. Une lecture qu’il avait reprise de Jellystat contient toute la session, et c’est elle qui est lue : datée du moment où elle s’est terminée, le seul moment que les deux outils aient gardé, et décrite comme un transcodage seulement quand la session en était vraiment un.
- Les lignes que Streamystats a écrites parce que Jellyfin signalait un élément comme vu, pour un visionnage qu’il n’a jamais observé, sont comptées puis ignorées : elles durent autant que le film entier et deviendraient du temps de visionnage un soir où personne n’a rien regardé. Le résultat indique combien il y en avait.
- Comment les données de Streamystats sont interprétées, champ par champ, d’après un vrai export plutôt que d’après une documentation : `docs/streamystats-import.md`.
- **De quel outil de suivi vient chaque lecture**, comme filtre dans Activité : ce que FinStats a enregistré lui-même, ce qui vient de Jellystat, ce qui vient de Streamystats. Seuls ceux que votre historique contient réellement sont proposés, et le filtre est restreint comme toute autre liste, si bien que quelqu’un qui ne peut voir que ses propres lectures n’apprend que d’où vient son propre historique.
- **Les filtres d’Activité sont des listes déroulantes à cocher**, si bien que vous pouvez demander les films *et* les épisodes sans la musique, deux modes de lecture ou deux personnes à la fois, au lieu d’un seul de chaque ou de tous. Le type de média gagne **Autre**, maintenant qu’il peut être combiné.
- Plusieurs personnes à la fois, partout où apparaît le filtre d’utilisateur (le tableau de bord, la lecture, les médiathèques, la sécurité et le reste), la carte et chaque statistique suivant le choix. Un filtre d’utilisateur ne fait toujours que restreindre ce que quelqu’un a le droit de voir : sans *Voir l’activité de tous*, une requête est limitée à son auteur, quelle que soit la personne nommée dans l’adresse et quel que soit leur nombre.
- **Un thème clair** : du papier washi, avec un texte à l’encre et un accent vermillon. FinStats suit le réglage clair ou sombre de l’appareil jusqu’à ce que vous en choisissiez un.
- **Un bouton de thème à côté de Se déconnecter** passe par Appareil, Clair et Sombre. Le choix est conservé dans ce navigateur et appliqué avant que la page ne soit dessinée, si bien qu’elle n’affiche jamais un éclair de l’autre thème.
- **Ouvrir dans Jellyfin** : chaque page de film, de série et d’épisode a un bouton qui ouvre la page de ce titre dans Jellyfin, dans un nouvel onglet.
- **L’adresse de Jellyfin pour les utilisateurs** sous Paramètres → Jellyfin : là où pointent ces boutons, pour le cas où l’adresse à laquelle FinStats se connecte n’est pas joignable par un navigateur. Vide, c’est l’adresse à laquelle FinStats se connecte qui est utilisée ; seul un administrateur Jellyfin peut la définir.

### Changed
- **FinStats a déménagé** : `github.com/finstats/finstats`, et l’image est `ghcr.io/finstats/finstats`. Changez le nom de l’image dans votre `docker run` ou votre fichier compose pour continuer à recevoir les mises à jour ; le dossier de données reste tel quel.
- **Paramètres n’affiche qu’une section à la fois.** La page comptait quatorze cartes sur une seule colonne et six cents mots d’aide. Désormais, chaque section a sa propre adresse (`/settings/collection`), une liste à gauche permet de passer de l’une à l’autre, et sur un téléphone la liste se replie en pastilles. Les liens vers les anciennes ancres fonctionnent toujours.
- **Un réglage tient sur une ligne** : ce qu’il est, une ligne qui énonce la règle, le contrôle à droite. Les nombres d’une section s’enregistrent ensemble avec un seul bouton en bas ; les interrupteurs s’enregistrent toujours d’eux-mêmes. Les cartes d’état (Tâches, Connexions sortantes, Base de données) et le lien vers les licences se trouvent sous **Système**, et les deux outils d’importation ont leur propre section.
- **Trouver un réglage** depuis la case de l’en-tête (`/` y place le curseur) avec n’importe quel mot de son nom ; elle indique dans quelle section se trouve le réglage, et Entrée mène à sa ligne.
- **Paramètres est ouvert à toute personne connectée** : les sections qui gèrent le serveur restent réservées à ceux qui peuvent le gérer, et une personne autorisée seulement à recevoir des notifications peut enfin atteindre ses propres destinations.
- **La page Serveur n’affiche qu’une section à la fois**, comme Paramètres : Vue d’ensemble (système et stockage), Tâches, Appareils, Extensions, Journal et Audit, chacune à sa propre adresse. Les liens vers `/server#jobs` fonctionnent toujours.
- **Le journal du serveur est une section de la page Serveur** (`/server/log`) au lieu d’une page à part dans le menu ; `/events` y renvoie avec ses filtres.
- Une notification concernant une tâche échouée, une sauvegarde échouée ou une connexion qui ne répond plus renvoie à la section des paramètres concernée, au lieu du haut de la page.
- Ce que votre médiathèque peut dire d’une lecture importée (le type d’élément, l’épisode, sa durée, la médiathèque à laquelle il appartient) est complété après **chaque** lecture de la médiathèque, et pas seulement au moment d’une importation. L’historique est généralement importé avant que FinStats ait jamais lu la médiathèque, si bien que ces lectures restaient devinées ou inconnues pour toujours.
- Restaurer une sauvegarde FinStats n’ajoute plus une lecture arrivée d’un outil de suivi sous un autre identifiant, et garde toujours deux vrais visionnages de la même chose à quelques minutes d’intervalle.
- Les détails d’une lecture nomment l’outil de suivi qui l’a apportée, au lieu d’appeler lecture Jellystat tout ce qui a été importé.

### Performance
- **Un redémarrage est quasi instantané, quel que soit l’historique.** Chaque démarrage recalculait qui avait regardé ensemble sur tout l’historique et redécidait quelles lectures étaient locales, pour ne rien changer : 24 s et 5,5 s sur dix millions de lectures. Un démarrage ne fait plus que ce que la dernière exécution a laissé en suspens, et ne refait tout qu’après une mise à jour ou quand le réglage concerné a changé.
- **Une foule qui arrive d’un coup est enregistrée en quelques secondes.** Chaque nouvelle lecture, chaque enregistrement de progression et chaque appareil étaient écrits séparément, et chaque lecture terminée était close en relisant tout l’historique de son titre ; un passage écrit désormais chaque catégorie en une fois. 100 000 lectures qui commencent ensemble prennent 3 s, là où 25 000 qui se terminaient ensemble prenaient plus de quatre minutes.
- **En cours de lecture reste rapide avec des milliers de spectateurs.** Trouver qui regarde ensemble comparait chaque flux à tous les autres et listait chaque compagnon par son nom : quarante secondes à 10 000 flux, et des gigaoctets de mémoire au-delà. Les flux sont désormais comparés au sein d’un même titre, et chacun affiche quelques noms et le nombre des autres.
- Calculer qui avait regardé ensemble réécrivait chaque lecture groupée de l’historique à chaque démarrage, importation ou fin de lecture de FinStats ; seul ce qui a changé est désormais écrit. Trouver les lectures terminées ne ralentit plus avec le carré du nombre de lectures en cours.
- **Reconnaître une lecture déjà présente dans l’historique passe par un index**, si bien qu’une importation de 150 000 lectures prend quelques secondes, et le rattachement après une lecture de la médiathèque avance de titre en titre au lieu de lire chaque lecture en bloquant la base de données.
- **Les analyses de lecture et la vue d’ensemble sont plus rapides sur toute la période** : leurs parties indépendantes sont calculées côte à côte, le graphique de simultanéité cherche le jour local une fois par quart d’heure au lieu d’une fois par lecture, et les genres sont comptés par titre. Sur un million de lectures : de 4,1 s à 2,4 s, et de 1,6 s à 1,1 s.

### Stability
- **Une médiathèque de séries n’est plus prise pour une médiathèque vidée.** Lire une médiathèque se fait en deux passes (chaque élément, puis une passe plus petite pour la distribution et l’équipe des films et des séries seulement), et la vérification qui décide si une lecture est assez fiable pour supprimer ce qu’elle n’a pas vu regardait la seconde. Sur une médiathèque de séries ou de musique, les deux comptes sont très éloignés, si bien que la vérification refusait chaque lecture et arrêtait FinStats, puis l’installation redémarrait et recommençait. Rien n’a jamais été supprimé à tort, mais aucune lecture de la médiathèque ne pouvait aboutir. Si votre médiathèque Séries ou Musique ne récupérait plus les nouveaux titres, voici pourquoi.
- Une importation pouvait échouer avec « database is locked » quand une autre écriture (une lecture en cours de mise à jour) se glissait entre sa première lecture et sa première écriture. Les deux outils d’importation prennent le verrou d’écriture dès le début, et le journal de la base de données ne reste plus aussi gros que la plus grosse importation jamais faite.
- **Un fichier de base de données endommagé, tronqué, appartenant à un autre programme ou en lecture seule est refusé immédiatement**, avec le nom du fichier et l’assurance que rien n’a été modifié, au lieu de 30 secondes de silence ; la base de données d’un autre programme n’est jamais modifiée.
- Une sauvegarde ou un instantané d’avant mise à jour interrompu à mi-chemin ne laisse plus de fichier derrière lui, un fichier qui pouvait prendre la place d’un bon instantané parmi les trois conservés.
- **Remplacer la base de géolocalisation pendant que FinStats tourne ne le fait plus planter** : FinStats lit une copie privée du fichier.
- Une application Jellyfin qui annonçait un nouvel identifiant d’appareil à chaque fois pouvait faire grossir la mémoire de FinStats aussi longtemps qu’il tournait ; les appareils non vus depuis cinq minutes sont désormais oubliés.
- Une personne garde au plus 30 sessions connectées ; la plus ancienne est déconnectée à la connexion suivante.

### Fixed
- **La même soirée n’est plus comptée deux fois quand un élément a été renommé.** Rattacher l’historique à un titre réajouté dans Jellyfin pouvait transformer une lecture importée d’un outil de suivi en doublon d’une lecture déjà présente, parce que la vérification qui l’aurait repéré s’exécutait avant le changement d’identifiant. Elle s’exécute désormais à nouveau partout où des identifiants sont rattachés. Seules des lectures importées sont supprimées, jamais une lecture que FinStats a enregistrée lui-même, et leur nombre figure dans le journal. Le premier démarrage de 2.0 passe une fois tout l’historique en revue pour celles qui sont déjà là.
- **Une session suit désormais la personne telle que Jellyfin la connaît aujourd’hui.** Un administrateur rétrogradé par Jellyfin gardait un accès complet à FinStats pour le reste de sa session de 30 jours, et une personne désactivée ou supprimée dans Jellyfin restait connectée. Elle perd désormais cet accès dès que FinStats lit les utilisateurs de Jellyfin (toutes les 15 minutes, ou immédiatement depuis Paramètres), comme l’ont toujours fait les clés API.
- La connexion inscrit ce que Jellyfin dit de la personne dans la liste des utilisateurs de FinStats, si bien qu’une sauvegarde importée d’un outil de suivi ne peut plus faire de quelqu’un un administrateur avant que la première lecture des utilisateurs ne corrige cela.
- Une personne supprimée dans Jellyfin est marquée comme supprimée même quand deux lectures des utilisateurs tombent dans la même seconde.
- **Les erreurs de téléchargement n’affichent jamais le mot de passe d’un client de téléchargement ni la clé API d’un indexeur**, que Sonarr et Radarr peuvent y citer.
- Les lectures rattachées à un titre renommé pendant une lecture de la médiathèque n’étaient vérifiées pour le visionnage à plusieurs qu’au redémarrage suivant ; elles le sont désormais immédiatement.

## [1.6.5] - 2026-09-24

### Added
- **Une sauvegarde complète est faite automatiquement avant chaque mise à niveau.** La première fois qu’un FinStats plus récent ouvre votre base de données, il en écrit une copie complète (tout votre historique, paramètres compris) dans `data/pre-update-backups/` *avant* de modifier quoi que ce soit. Si une mise à niveau casse un jour quelque chose, vos données sont à l’abri : arrêtez FinStats, remettez la copie à la place de `finstats.db`, démarrez la version que vous utilisiez et signalez le bogue. Rien n’est perdu. Les quelques copies les plus récentes sont conservées ; définissez `FINSTATS_SKIP_PREUPDATE_BACKUP=1` pour désactiver ce mécanisme si l’espace disque est compté.

## [1.6.4] - 2026-09-24

### Added
- **Cinq nouvelles destinations pour les notifications** : Telegram, Slack, Pushover, Pushbullet et l’e-mail, en plus des destinations webhook, Discord, ntfy et Gotify déjà présentes. Chacune ne demande que ce dont elle a besoin (un identifiant de discussion, une clé utilisateur, une boîte aux lettres), et les trois qui vivent sur un seul service (Telegram, Pushover, Pushbullet) ne peuvent jamais être jointes ailleurs, si bien qu’un jeton ne peut pas être envoyé à un hôte qui les imite. Le courrier part en TLS dès le premier octet ou avec `STARTTLS`, et il n’y a pas de troisième option.
- **Licences**, sous **Paramètres** : la GPL de FinStats et chaque licence tierce sur laquelle il repose (253 crates Rust, les deux polices fournies, les contours de la carte et la base de géolocalisation), chaque texte en entier, tel que son projet l’a écrit. Choisissez un composant et sa licence s’ouvre dans le volet à côté de la liste ; une crate qui fournit plusieurs fichiers passe de l’un à l’autre sur place. La liste est générée à partir des crates dont FinStats est réellement construit, si bien qu’aucune dépendance ne peut être ajoutée sans que sa licence soit enregistrée.

## [1.6.3] - 2026-09-24

### Fixed
- Le garde-fou intervient désormais aussi quand Jellyfin ne liste *aucune* médiathèque ou *aucun* utilisateur (une lecture de Jellyfin défaillante, jamais une journée normale), ce que le réglage de 1.6.2 avait laissé passer. Perdre quelques médiathèques reste considéré comme ordinaire ; les perdre toutes d’un coup ne l’est pas.

## [1.6.2] - 2026-09-24

### Fixed
- Le garde-fou d’intégrité des données ajouté en 1.6.1 était trop zélé : une médiathèque qui perdait réellement la plupart de ses éléments (une poignée de clips dont les fichiers avaient disparu, par exemple) était prise pour une lecture défaillante de Jellyfin, si bien que FinStats refusait de la mettre à jour et s’arrêtait. Il n’intervient désormais que pour une lecture *manifestement* défaillante : une médiathèque assez grande pour compter qui revient complètement vide, ou une grande médiathèque réduite à presque rien. Les changements ordinaires s’appliquent comme avant, et le cas « Jellyfin n’a rien renvoyé » est toujours détecté.

## [1.6.1] - 2026-09-24

### Added
- **FinStats ne fait plus confiance à une lecture de Jellyfin qui effacerait votre historique.** Si une médiathèque pour laquelle FinStats conserve des milliers d’éléments revient soudain vide ou presque (une mise à niveau de Jellyfin qui a changé son API, une erreur déguisée en résultat vide), FinStats garde les données qu’il a, marque la synchronisation comme échouée (et le notifie), et s’arrête proprement avec un message clair au lieu de tout marquer comme supprimé. Redémarrez une fois que Jellyfin est redevenu lui-même, ou définissez `FINSTATS_ALLOW_LIBRARY_SHRINK=1` si vous avez vraiment vidé une médiathèque. Le même garde-fou couvre la liste des médiathèques et la liste des utilisateurs.

### Security
- Un en-tête `X-Forwarded-Host` envoyé par un client pouvait se porter garant d’un `Origin` étranger et faire passer une écriture intersite à travers la protection de même origine. Cet en-tête n’est désormais pris en compte que derrière un proxy de confiance (`FINSTATS_TRUST_PROXY`), la même règle que celle qu’applique déjà la détection de l’adresse du client. Le cookie de session `SameSite=Lax` était, et reste, la première ligne de défense.
- Une destination de notification personnelle pouvait viser `0.0.0.0` ou `::`, qui atteignent cette machine. La vérification qui oblige les destinations personnelles à pointer vers des adresses publiques les compte désormais comme locales, au même titre que le bouclage et les plages privées.

### Fixed
- La page **Lecture** ne manque plus de se charger pour une médiathèque contenant une lecture sans mode enregistré (une ligne restaurée depuis une sauvegarde antérieure à la colonne) ; une telle lecture compte simplement comme une lecture directe.
- Un mot très long dans une recherche ou un filtre ne provoque plus d’erreur serveur : SQLite refuse un motif `LIKE` de plus de 50 000 caractères, les mots de filtre sont donc coupés à une longueur raisonnable avant la requête.
- La page **Paramètres** ne défile plus latéralement sur un téléphone quand une carte affiche une phrase entière en retour, comme le résultat d’une restauration.

## [1.6.0] - 2026-09-24

Notifications

### Added
- Carte **Notifications** sous Paramètres : ajoutez une destination, cochez ce dont elle doit être informée, appuyez sur Tester. Sans destination, aucune requête n’est envoyée nulle part.
- Quatre types de destination : un simple **webhook** (JSON, documenté dans `docs/api.md`, en-tête `Authorization` facultatif), **Discord** (un embed via l’URL de webhook d’un salon), **ntfy** (ntfy.sh ou le vôtre, vers un sujet) et **Gotify** (votre propre serveur, avec un jeton d’application). Les titres voyagent dans le message plutôt que dans un en-tête.
- Onze types d’événements, chacun avec sa propre case : un voyage impossible, un compte vu dans un nouveau pays, une rafale de connexions échouées, une tâche échouée, une sauvegarde qui n’a pas pu être écrite, quelque chose qui a cessé de répondre (Jellyfin, Sonarr, Radarr ou Seerr) et son retour, les nouveaux titres de la médiathèque (regroupés par série et par jour, comme la rangée du tableau de bord), une demande devenue disponible, et le début ou la fin d’une lecture. Les deux derniers sont désactivés par défaut.
- Une demande devenue disponible nomme le titre, qui l’a demandé et combien de temps cette personne a attendu.
- Destinations personnelles : accordez **Recevoir des notifications** et quelqu’un peut ajouter les siennes, qui ne reçoivent que ce qu’il peut déjà voir : ses propres demandes et alertes toujours, celles des autres avec *Voir l’activité de tous*, les adresses avec *Voir les détails réseau*. Une destination personnelle doit pointer vers une adresse publique ; seul un administrateur peut en diriger une vers l’intérieur du réseau.
- **Inclure les adresses IP et les lieux**, par destination et désactivé par défaut : sinon, un message sur un voyage impossible ne nomme que des lieux (« Oslo, Norvège et Londres, Royaume-Uni »).
- **Envoyés récemment**, dans la même carte : ce qui a été dit, à quelle destination, et ce que l’autre côté a répondu quand le message n’est pas arrivé. Un message échoué est retenté quatre fois sur environ une heure, et une destination qui limite le débit est attendue.
- **L’adresse de FinStats**, un réglage pour les administrateurs, sert à construire le lien d’une notification. Vide, les messages ne portent aucun lien.
- Les tâches planifiées de Jellyfin sur la page Serveur : ce que fait chacune, en mots plutôt que par le nom de son code, et si c’est l’une des plus lourdes. Une tâche en cours affiche une barre de progression, un pourcentage et le temps restant ; les autres indiquent leur prochaine exécution, leur dernière exécution et si elle s’est bien passée. Les tâches que Jellyfin masque dans son propre tableau de bord sont incluses, et rien dans FinStats ne peut lancer ni arrêter une tâche.
- Le temps restant d’une tâche est mesuré d’après la vitesse à laquelle son pourcentage a réellement avancé, par rapport au relevé le plus récent assez ancien pour signifier quelque chose plutôt qu’à la moyenne de toute l’exécution, si bien qu’une tâche qui accélère ou ralentit est décrite selon son rythme actuel.
- Les listes de tâches que FinStats lit déjà pour d’autres raisons (la vérification d’analyse de la médiathèque toutes les 5 minutes, les détails du serveur toutes les 15) alimentent la même mesure, si bien qu’une tâche en cours depuis une heure a généralement une estimation dès l’ouverture de la page. Rien de plus n’est demandé à Jellyfin pour cela.
- Une tâche en cours sans aucune mesure pour l’instant affiche **ETA** et des points de suspension qui défilent au lieu d’un nombre, et celle dont le pourcentage n’a pas bougé depuis une demi-minute affiche « 68 % depuis 4 min » à côté. Rien ne remplace une estimation qui n’a pas été mesurée.
- Une liste de toutes les versions à côté des notes de version : un clic mène à n’importe quelle version, celle qu’on lit est signalée, et sur un écran étroit elle devient une rangée de séries au-dessus des notes.

### Changed
- Les notes de version ne se replient plus. Chaque série reste ouverte et c’est la liste des versions qui permet de circuler, au lieu d’un seul groupe ouvert à la fois et d’un clic pour atteindre les autres.
- **Paramètres → Connexions sortantes** liste chaque destination de notification à côté de Jellyfin et des connexions aux services : ce qu’elle est, son hôte, si elle est activée, et quand elle a accepté un message pour la dernière fois.
- Rien ne change dans la collecte, l’historique ou les chiffres affichés. FinStats ne fait toujours que *lire* Jellyfin, Sonarr, Radarr et Seerr.

## [1.5.2] - 2026-09-22

### Changed
- Une lecture de la liste des sessions de Jellyfin et la remise à zéro de l’horloge qui l’a demandée ne forment plus qu’un seul appel, si bien que les deux ne peuvent plus se désynchroniser. C’était la forme de la panne de requêtes en rafale découverte pendant la préparation de 1.5.0.
- Le prouver prend une fraction de seconde au lieu de sept minutes contre un vrai serveur, si bien que c’est vérifié à chaque changement. Les mêmes requêtes, le même historique, les mêmes chiffres à l’écran.

## [1.5.1] - 2026-09-22

### Fixed
- Un transcodage est enregistré une fois, et non une fois par seconde. Quand une lecture commence en transcodage puis revient à la lecture directe, Jellyfin laisse les détails de transcodage sur la session ; chaque relevé était comparé à une valeur que FinStats venait d’écraser, si bien que chaque relevé ressemblait à un changement et que la page Activité se remplissait de lignes « Transcodage » portant en dessous « Lecture directe : … ».
- Les lignes déjà écrites sont nettoyées au démarrage : là où une série de lignes dit la même chose, la première est gardée. Une lecture qui a eu besoin d’un transcodage compte toujours comme un transcodage.

## [1.5.0] - 2026-09-22

Suivi des sessions par WebSocket

### Changed
- FinStats garde une connexion ouverte vers Jellyfin dès son démarrage, et c’est par elle qu’il apprend qu’une lecture a lieu. 1.4.0 l’avait livrée sous forme de réglage, désactivé par défaut, pour qu’elle fasse ses preuves sur de vrais serveurs ; elle est désormais toujours active. Les mêmes lectures, pauses et sauts, calculés par le même code.
- Rien en lecture : FinStats ne demande rien du tout à Jellyfin. Une soirée où personne ne regarde ne coûte aucune requête.
- Quelque chose en lecture : la liste des sessions est lue une fois par seconde. C’est ce qui donne sa précision à une pause, à un saut ou à un changement de piste, et c’est ce qui met fin à une lecture dont le client a disparu sans dire au revoir.
- Tout ce qui est chargé est en pause depuis trois relevés d’affilée : les interrogations s’arrêtent et la connexion prend le relais, là où un film en pause coûtait auparavant la même question chaque seconde, environ 16 Mo par heure. Une reprise, ou quelqu’un d’autre qui lance quelque chose, obtient une réponse en une seconde environ, et le temps de pause ne compte toujours jamais comme temps de visionnage.
- Chaque requête sortante demande des réponses compressées et les décompresse (Jellyfin, Sonarr, Radarr et Seerr compressent tous quand on le leur demande), ce qui allège de plus de moitié les deux plus grosses réponses : la liste des sessions pendant une lecture, et la médiathèque. La base de géolocalisation est toujours téléchargée telle qu’elle est stockée.
- La carte Jellyfin de Paramètres indique laquelle des deux moitiés est active, et le point de direct suit le fait que la connexion transmet ou non, plutôt que la moitié dans laquelle on se trouve.
- Une seule des deux tourne à la fois : pendant que FinStats interroge, il demande à Jellyfin d’arrêter d’envoyer, et lui demande de reprendre dès que la dernière lecture se termine. La connexion reste ouverte tout du long.
- Un Jellyfin qui répond à l’abonnement par le silence est cru tant qu’il répond aux keep-alive : FinStats le signale une fois, lit lui-même la liste des sessions en attendant, et reste abonné jusqu’à ce que le premier envoi tranche la question. « Ne sait pas envoyer » et « était en train de redémarrer quand on a demandé » se ressemblent, donc le silence n’est jamais un verdict.
- Le mode de secours est inchangé : une connexion qui se ferme, devient muette ou se révèle ne pas parler ce protocole revient aux interrogations à intervalle dès le passage suivant et continue de se reconnecter, aux intervalles **Pendant un visionnage, vérifier toutes les** et **Quand rien n’est en lecture, vérifier toutes les**.

### Added
- Ce que fait le collecteur, sur `GET /api/status` : lequel des quatre états il occupe (`idle_socket`, `playing_poll`, `paused_socket`, `fallback`), si la connexion est ouverte et abonnée, à quelle fréquence il interroge, combien de lectures de la liste des sessions sont parties dans la dernière minute, et quand cela a changé pour la dernière fois. Répondu depuis la mémoire (aucune requête, aucune interrogation de la base), et sans aucun nom, titre ni décompte.
- Ces champs sont comparés entre eux à chaque passage, et un désaccord qui dure est écrit dans le journal.
- Un limiteur unique sous chaque lecture de la liste des sessions dans le processus : au plus 2 par seconde et 70 par minute, tout dépassement étant retardé, compté et consigné au plus une fois par minute. Un visionnage normal reste largement en dessous.

### Fixed
- Les mises à jour en direct remplacent vraiment les interrogations. Une fois la fonction activée, la 1.4.0 demandait encore la liste des sessions toutes les cinq secondes et rouvrait la connexion toutes les vingt à quarante, parce qu’elle jugeait la connexion selon l’ancienneté de la dernière liste reçue, alors que Jellyfin n’en envoie une que lorsque quelque chose *change*, et rien entre-temps. La vitalité de la connexion se mesure désormais au fait que Jellyfin réponde ou non, vérifié par un keep-alive environ deux fois par minute.
- Une liste de sessions arrivée pendant que FinStats était entre deux passages était jetée, en supposant qu’une autre suivait à une seconde et demie. Elle est gardée et utilisée.
- **Dernière vérification** sous Paramètres, et la carte Connexions sortantes, ne vieillissent plus pendant que la connexion est silencieuse : c’est la réponse de Jellyfin à un keep-alive qu’elles affichent.

### Removed
- Le réglage **Laisser Jellyfin envoyer ce qui est en lecture** sous Paramètres → Collecte, et `live_socket` de l’API des paramètres.
- `socket_enabled` de l’état du collecteur : il n’existe plus d’état « désactivé ».
- `last_reconcile_at` de l’état du collecteur, remplacé par `socket_live`.

## [1.4.1] - 2026-09-22

### Changed
- Seerr n’est lu en entier que lorsque quelque chose a changé : toutes les cinq minutes, FinStats demande une seule ligne, la demande modifiée le plus récemment, et le passage s’arrête là s’il la connaît déjà. Cela fait environ un kilooctet, contre une soixantaine pour une page de cinquante. Les demandes encore en route sont examinées tous les quarts d’heure, tout est listé une fois par jour, et une nouvelle demande apparaît toujours en moins de cinq minutes.
- Les files de Sonarr et Radarr sont lues toutes les cinq secondes pendant qu’une page les affiche, toutes les minutes tant que quelque chose est dans la file et toutes les cinq minutes quand elle est vide, au lieu de toutes les minutes jour et nuit. Ouvrir la page, connecter un service ou une nouvelle demande dans Seerr les actualise immédiatement.

## [1.4.0] - 2026-09-21

Sessions par WebSocket en option, et tout ce que FinStats peut joindre

### Added
- **Laisser Jellyfin envoyer ce qui est en lecture** sous Paramètres → Collecte, désactivé tant que vous ne l’activez pas : une seule connexion transporte la liste des sessions, et environ 17 000 requêtes par jour deviennent presque aucune. Les lectures, pauses, sauts et groupes sont calculés exactement par le même code qu’avant. Tant qu’elle est active, FinStats fait encore une requête ordinaire par minute *pendant qu’une lecture est en cours*, et aucune quand rien n’est lu. Une connexion qui se ferme, reste muette quinze secondes ou se révèle ne pas parler ce protocole revient au minuteur dès le passage suivant et continue de se reconnecter.
- **Connexions sortantes**, une carte dans Paramètres pour les administrateurs Jellyfin : chaque destination que FinStats peut joindre (votre Jellyfin, le service « quelle est mon IP », la base de DB-IP, et chaque Sonarr, Radarr ou Seerr), avec son usage, si elle est activée, et quand elle a répondu pour la dernière fois. Construite à partir de ce que FinStats sait déjà, elle n’affiche que des adresses : jamais une clé, jamais un chemin de base.
- **Rechercher maintenant** sous Paramètres → Réseau domestique, pour le jour où votre adresse publique change.

### Changed
- La recherche « quelle est mon IP » ne s’exécute qu’une fois, la première fois que FinStats a besoin d’une adresse, au lieu de tous les quarts d’heure.
- La carte Jellyfin de Paramètres indique comment le collecteur est informé et, quand la connexion est activée mais ne transmet rien, pourquoi.

## [1.3.0] - 2026-09-20

Sonarr, Radarr et Seerr, sur une seule page

### Added
- **Connexions** sous Paramètres, pour les administrateurs Jellyfin : **Sonarr**, **Radarr** et **Seerr** (ou Jellyseerr/Overseerr), plusieurs de chaque type, chacun testé avant d’être enregistré. Votre client de téléchargement n’a besoin d’aucune configuration, car Sonarr et Radarr lui parlent déjà et FinStats lit ce qu’ils savent.
- **Demandes** : qui a demandé quoi, combien de temps il a fallu pour que cela arrive, et si la personne l’a jamais regardé. Des tuiles pour les titres en attente et l’attente typique, un graphique de son évolution mois par mois, et des listes de ce qui est arrivé il y a des semaines sans jamais être lu, et de qui demande le plus.
- **À venir** : les nouveaux épisodes et les sorties de films de Sonarr et Radarr sous forme d’agenda jour par jour, avec une marque indiquant si *vous* regardez cette série et, pour ceux qui peuvent voir tout le monde, qui d’autre la regarde. Une série compte comme regardée quand quelqu’un en a lu un épisode au cours des quatre derniers mois. Une rangée « À venir » sur le tableau de bord, une par profil pour les séries que cette personne regarde, et la suite d’un titre sur sa propre page.
- **Téléchargements** : la file en direct de chaque Sonarr et Radarr (ce que c’est, son avancement, sa vitesse, ce qui a échoué à l’importation), avec à côté la personne qui l’a demandé. Un pack de saison tient sur une ligne quel que soit son nombre d’épisodes, et ce qui demande de l’attention est en haut. En dessous, ce qui est arrivé en une semaine, un mois ou un an par indexeur, qualité et client de téléchargement, et les téléchargements qui ont échoué. Le tableau de bord affiche la même liste, raccourcie.
- **Voir les téléchargements**, une nouvelle autorisation. Sans elle, chacun voit toujours où en est sa *propre* demande et combien de temps il reste, et rien d’autre : aucun nom de release, aucune vitesse, aucun autre téléchargement. Les demandes des autres exigent *Voir l’activité de tous*.

### Changed
- FinStats n’envoie que des `GET` à ces services : il ne contient aucun code capable d’approuver une demande, de lancer une recherche, ni d’ajouter, mettre en pause ou supprimer un téléchargement.
- Leurs clés API sont stockées dans la base de données de FinStats, jamais renvoyées au navigateur, jamais écrites dans un journal et jamais incluses dans une sauvegarde. Aucune redirection n’est suivie, si bien qu’une clé ne peut pas partir ailleurs que là où vous l’avez saisie, et les certificats sont vérifiés à moins que vous ne désactiviez cette vérification pour une connexion.

## [1.2.2] - 2026-09-20

### Added
- Les langues audio et de sous-titres sur chaque film et épisode (« Audio : japonais · anglais »).
- Jusqu’où va un doublage : une série ou une saison indique combien de ses épisodes ont chaque langue (« Anglais : 13 épisodes sur 26 »), et la liste des épisodes a une colonne **Audio**.
- **Langues audio** et **Langues des sous-titres** sur les pages des médiathèques : combien de fichiers peuvent être lus dans chacune.
- FinStats relit la médiathèque une fois après cette mise à jour pour récupérer les langues. Il ne dit jamais « doublé », parce que Jellyfin ne donne pas la langue originale d’un titre, et une piste sans étiquette de langue est listée comme « Inconnu ».

## [1.2.1] - 2026-09-20

### Changed
- Les images du README sont reprises pour cette version, avec une nouvelle de la page **Sécurité**. Rien ne change dans l’application.

## [1.2.0] - 2026-09-20

D’où l’on regarde, sur une carte

### Added
- **Page Sécurité**, pour ceux qui peuvent voir les détails réseau et l’activité de tous : une carte du monde avec un point pour chaque lieu d’où vos utilisateurs regardent, dimensionné selon ce qui s’y passe. Le domicile est en vert, les lieux extérieurs en violet, les lectures en cours pulsent et les connexions échouées depuis l’extérieur sont en rouge. Faites glisser pour vous déplacer, zoomez avec les boutons, les touches ou Ctrl + molette, et les points qui se chevauchent fusionnent tant que vous n’avez pas zoomé. En dessous, chaque lieu avec ses personnes, ses lectures, son temps de visionnage et ses connexions, et les pays par nombre de lectures.
- **Voyage impossible** : une alerte quand un même compte est vu en deux lieux qu’aucun vol ne relie, ou en deux lieux éloignés en même temps, avec les deux observations, la distance, le temps écoulé entre elles et la vitesse qu’il aurait fallu. **Afficher sur la carte** trace le trajet.
- **Nouveau pays** : la première fois que quelqu’un lit ou se connecte depuis un pays où on ne l’avait jamais vu.
- Résolvez une alerte avec une note, ou mettez en sourdine une paire de lieux pour cette personne, pour un VPN ou un téléphone dont l’opérateur est installé dans la capitale. Les alertes trouvées dans un ancien historique, après une importation ou sur une base neuve, sont classées comme résolues au lieu d’inonder la liste. Les alertes et ce que vous en avez décidé font partie des sauvegardes.
- Les lieux viennent d’une base de villes lue localement : aucune adresse n’est jamais envoyée nulle part, et la carte est dessinée à partir de contours fournis avec FinStats plutôt que par un service de cartographie. Téléchargez la base gratuite de DB-IP depuis la page Sécurité ou **Paramètres → Sécurité**, laissez FinStats l’actualiser chaque mois (désactivé tant que vous ne l’activez pas), ou déposez votre propre `.mmdb` (DB-IP, MaxMind GeoLite2-City) dans `data/geoip/`. `FINSTATS_GEOIP_DB` désigne un fichier situé ailleurs.
- **Paramètres → Sécurité** : la base utilisée, la mise à jour mensuelle, et à quelle vitesse (900 km/h) et à quelle distance l’une de l’autre (500 km) deux observations doivent être pour compter comme un voyage impossible. Les bases de villes se trompent souvent de quelques centaines de kilomètres, et c’est à cela que sert la distance.

## [1.1.3] - 2026-09-20

### Added
- Ctrl+Espace cherche aussi dans la distribution et l’équipe de tout ce que contient votre médiathèque, sous **Distribution et équipe**, avec leur photo, s’ils jouent ou réalisent, et dans combien de vos titres ils figurent ; en choisir un ouvre sa page. Aussi tolérante que la recherche de titres (mots dans n’importe quel ordre, accents ignorés, faute de frappe permise), et quand plusieurs correspondent aussi bien, celui qui figure dans le plus de vos titres passe en premier.

## [1.1.2] - 2026-09-20

### Added
- **Ajouts récents** sur le tableau de bord, au-dessus du graphique d’activité : les 30 arrivées les plus récentes sous forme de rangée d’affiches à faire défiler latéralement, avec des flèches sur ordinateur. Chacune indique quand elle est arrivée et ce que c’est (un film, un album, ou « Saison 4 · 3 épisodes »), et un seul nouvel épisode indique lequel, avec l’affiche propre de la saison quand elle en a une.
- **Maj + molette** et les flèches avancent d’une affiche, Page précédente et Page suivante d’un écran, Début et Fin vont à l’une ou l’autre extrémité, et le balayage fonctionne. La molette seule fait toujours défiler la page.
- Les nouveaux épisodes d’une même série arrivés le même jour partagent une entrée, et une série entière ajoutée d’un coup forme une seule entrée (« 3 saisons · 60 épisodes »), si bien qu’une grosse importation ne chasse pas tout le reste de la liste.
- La rangée ignore la période et la personne du tableau de bord, puisqu’elle concerne la médiathèque et non les lectures, et se masque tant que la médiathèque est vide.

## [1.1.1] - 2026-09-20

### Added
- Les pages s’ouvrent immédiatement : un instant après son chargement, FinStats récupère discrètement ce que vous allez probablement ouvrir ensuite (le tableau de bord, votre profil et votre chronologie, Activité, Utilisateurs, Médiathèques et chaque médiathèque, Lecture, les pages du serveur, le bilan et les personnes les plus actives), si bien qu’en ouvrir une l’affiche aussitôt, et elle s’actualise quand même en arrière-plan.
- Les films, séries et membres de la distribution ne sont récupérés que sur intention, car il y en a des milliers : poser le pointeur sur un lien, le toucher ou l’atteindre au clavier récupère cette page-là.
- Il attend que la page ouverte ait fini de se charger, travaille une page à la fois pendant que le navigateur est inactif, se met en pause dans un onglet en arrière-plan, ne fait rien sur une connexion en économie de données ou très lente, et tout est oublié quand vous vous déconnectez.
- Un petit lien **Dépôt** avec le logo GitHub dans la barre d’état, à côté de la version.

### Fixed
- La **Chronologie** oubliait quelles médiathèques étaient désactivées quand la page était rechargée ou ouverte depuis un marque-page, alors que le choix figurait dans l’adresse.

## [1.1.0] - 2026-09-20

Une chronologie de tout ce que vous avez regardé

### Added
- **Chronologie**, à côté de **Vue d’ensemble** sur chaque profil : votre visionnage sous forme d’un seul parcours, d’aujourd’hui jusqu’à la première lecture que FinStats connaît, une soirée d’épisodes repliée en une seule étape. Chaque étape est une affiche avec ce qui a été regardé et quand (« Saison 2 · Épisodes 3-6 », un film et s’il a fallu plus d’une séance, ou un album et combien de pistes), et ouvre le titre.
- Le parcours suit la largeur de la fenêtre : trois étapes par ligne sur un écran large, deux sur un plus étroit, une ligne sur deux allant à rebours avec un virage qui la relie à la suivante, jusqu’à une seule ligne droite sur un téléphone. La première étape de chaque mois porte le mois.
- Les étapes plus anciennes se chargent d’elles-mêmes à mesure que vous faites défiler, jusqu’au début de l’historique.
- Des interrupteurs **Médiathèques** au-dessus du parcours écartent ce que vous ne voulez pas voir, comme la musique, et le choix fait partie de l’adresse.
- Ce qui est encore en lecture indique **En cours de lecture**.
- Vous voyez votre propre chronologie ; celles des autres exigent l’autorisation « Voir l’activité de tous ».

## [1.0.5] - 2026-09-20

### Added
- Protection contre les retours en arrière : la base de données retient la version de FinStats la plus récente qui l’a ouverte, et une version plus ancienne refuse de démarrer dessus au lieu de travailler discrètement sur des données qu’elle ne comprend pas entièrement. Le message dit quoi faire. Les versions jusqu’à 1.0.4 sont sorties avant que cette vérification existe.

## [1.0.4] - 2026-09-20

### Fixed
- Dans les détails d’une lecture, le bouton de copie à côté d’un long **Identifiant de l’appareil** était tombé sur une ligne à part.
- Sur la page Lecture, la dernière colonne de **Quels clients transcodent** était coupée (« Transc… ») ; les en-têtes de deux mots passent désormais à la ligne.

### Changed
- Les captures d’écran du README sont de nouveau à jour ; elles montraient encore la version 0.5.0.

## [1.0.3] - 2026-09-20

### Added
- La page d’un épisode affiche l’image propre à cet épisode, celle que Jellyfin montre dans sa liste d’épisodes, au lieu de l’affiche de la série. Un épisode pour lequel Jellyfin n’a pas d’image garde l’affiche.

## [1.0.2] - 2026-09-20

### Fixed
- Dans les **Notes de version**, les groupes v0.9 et v0.10 n’avaient pas d’intitulé à côté de leur version. L’intitulé d’un groupe est désormais toujours la première phrase de ce dont parlait sa première version, si bien qu’une longue introduction n’est pas coupée au milieu d’une phrase.

## [1.0.1] - 2026-09-20

### Fixed
- L’image publiée ne démarrait pas sur une machine neuve : « unable to open database file: /data/finstats.db », après une demi-minute de blocage. Docker crée un dossier `data` manquant en tant que root, et FinStats tourne sous un utilisateur ordinaire. Le conteneur s’approprie désormais le dossier puis passe à cet utilisateur, si bien que `docker run` fonctionne du premier coup. Pour des fichiers appartenant à un autre utilisateur que l’utilisateur 1000, définissez `PUID` et `PGID` ; `--user` fonctionne comme avant.
- Un dossier de données réellement impossible à écrire est signalé immédiatement, avec la commande qui règle le problème, au lieu de 30 secondes plus tard sous forme d’erreur de base de données.

## [1.0.0] - 2026-09-20

Prêt pour tout le monde

### Added
- Une image toute prête, désormais sur `ghcr.io/finstats/finstats`, pour les machines Intel/AMD et ARM 64 bits (un Raspberry Pi 4 ou 5 convient) : `:latest` est la version la plus récente, une étiquette de version ne bouge pas, et `:edge` est la version de développement.
- Chaque version apparaît sur la page des versions GitHub du projet, avec ces notes de version.

### Changed
- Les numéros de version veulent désormais dire ce qu’ils disent : les mises à jour 1.x ne casseront ni vos données, ni vos sauvegardes, ni vos paramètres.
- Le fichier Docker Compose et toutes les instructions utilisent l’image publiée. Un `finstats:latest` construit soi-même n’a besoin que d’un changement de nom d’image ; le dossier `data` est repris sans y toucher.

## [0.10.2] - 2026-09-20

### Fixed
- Sur un téléphone, Paramètres et la page d’une série pouvaient défiler latéralement : un libellé destiné uniquement aux lecteurs d’écran sortait de son tableau, et les listes d’épisodes et la rangée de la distribution faisaient de même. Ils défilent désormais à l’intérieur d’eux-mêmes.
- Le bilan ne défile plus latéralement quand le mot de fond d’un chapitre est plus large que la page.
- Une longue ligne « regarde sur… » sous **En cours de lecture** se termine par des points de suspension au lieu d’être coupée au milieu d’une lettre.
- L’affiche devant chaque série d’un profil était, pour les lecteurs d’écran, un second lien sans nom ; elle leur est désormais masquée et c’est le titre à côté qui sert de lien.

## [0.10.1] - 2026-09-20

### Fixed
- Le téléchargement d’une sauvegarde échouait dans le navigateur (« the source file could not be read » dans Firefox) sans que rien sur la page n’explique pourquoi : FinStats compressait une seconde fois une sauvegarde déjà compressée, et le transfert doublement empaqueté s’interrompait avant la fin. Les sauvegardes sont envoyées telles quelles, avec leur taille annoncée d’emblée, si bien que le navigateur peut afficher une vraie progression.

## [0.10.0] - 2026-09-20

Les sauvegardes, et le passage à une nouvelle installation

### Added
- Une sauvegarde chaque semaine, les cinq plus récentes conservées : chaque lecture avec sa chronologie de pauses et de sauts, les marques de visionnage, les autorisations, les adresses du domicile, le journal du serveur et vos paramètres, dans un seul petit fichier, quelques centaines de Ko pour des milliers de lectures. **Paramètres → Sauvegardes** les liste avec un bouton **Télécharger** pour chacune, en crée une à la demande, et règle leur fréquence et leur nombre. Elles se trouvent dans le dossier `backups` de votre répertoire de données. Une sauvegarde ne contient ni clé API Jellyfin ni session de connexion, mais c’est un historique de visionnage complet avec des adresses IP : gardez-la en lieu sûr.
- La restauration fusionne au lieu d’écraser : les lectures déjà présentes sont ignorées, on peut donc la faire deux fois sans risque, ou dans une instance qui tourne depuis un moment. Décochez « Restaurer aussi les paramètres et les autorisations » pour ne restaurer que l’historique. Les sauvegardes fonctionnent d’une version à l’autre dans les deux sens. Depuis un terminal : `finstats backup` et `finstats restore <file>`.

### Changed
- La collecte vérifie chaque seconde pendant que quelqu’un regarde et toutes les 5 secondes quand personne ne regarde, là où elle vérifiait toutes les 5 secondes en permanence, si bien que les pauses, les sauts et les changements de piste sont enregistrés à la seconde près. Les deux réglages se trouvent sous **Paramètres → Collecte** ; un ancien intervalle modifié doit être saisi à nouveau.

### Fixed
- Une adresse IP dans les détails d’une lecture se coupait sur deux lignes au milieu de l’adresse. La marque Local/Distant passe en dessous quand la place manque.

## [0.9.1] - 2026-09-20

### Fixed
- Les personnes au domicile apparaissaient comme distantes. Un appareil de votre propre réseau qui atteint Jellyfin par son nom public, comme un proxy inverse ou un domaine, arrive avec l’IP publique du foyer, et seules les adresses privées comptaient comme locales. FinStats apprend désormais l’adresse publique et compte ces lectures comme locales sur tout l’historique, et il se souvient des adresses précédentes.

### Added
- **Paramètres → Réseau domestique** : les adresses que FinStats traite comme le domicile, l’ajout des vôtres (une ancienne adresse, une résidence secondaire, une sortie VPN), ou la désactivation de la recherche automatique. Celle-ci interroge un simple service « quelle est mon IP » toutes les 15 minutes, en essayant plusieurs (Amazon, Cloudflare et d’autres) parce que les DNS bloqueurs de publicité comme Pi-hole les bloquent souvent, et l’un d’eux n’a besoin d’aucun DNS. La requête ne contient rien sur vous ni sur votre serveur, et avec l’interrupteur désactivé, FinStats ne parle qu’à Jellyfin. `FINSTATS_PUBLIC_IP_URL` désigne un service à vous.

## [0.9.0] - 2026-09-20

Chaque tableau, dans l’ordre que vous voulez

### Added
- Chaque tableau se trie : cliquez sur un en-tête pour trier selon cette colonne, une deuxième fois pour inverser l’ordre, une troisième pour revenir à l’ordre d’origine. Le sens des cellules est compris, si bien que « 3 j 2 h » se trie comme une durée et « 1,4 GB » comme une taille, et les cellules vides passent en dernier. Cela couvre la liste des utilisateurs, les spectateurs, les épisodes, les appareils, les extensions, les tâches planifiées, la vue tableau de chaque graphique, la matrice des autorisations, et les listes en barres (codecs, résolutions, clients et le reste), qui reçoivent leur propre en-tête discret.
- Activité et le journal du serveur se trient sur toutes leurs pages plutôt que sur les lignes affichées, et l’ordre est conservé dans l’adresse, si bien qu’il survit à un rechargement ou à un lien partagé.
- Les tableaux de dix lignes ou plus reçoivent un petit champ de filtre au-dessus d’eux.

## [0.8.2] - 2026-09-20

### Changed
- Le sélecteur d’année du bilan est le même curseur que tous les autres sélecteurs de période de FinStats (7 j · 30 j · 90 j …).

## [0.8.1] - 2026-09-19

### Added
- Des pages pour les acteurs et les réalisateurs, depuis « Les personnes les plus vues » du bilan ou la nouvelle rangée **Distribution et équipe** sur chaque film, série ou épisode : tout ce dans quoi ils figurent sur votre serveur, ce qui a été regardé et par qui, et ce qui attend encore. La page suit la période choisie, ceux qui ne peuvent voir que leurs propres statistiques n’y voient que leur propre visionnage, et Échap vous ramène en arrière.

## [0.8.0] - 2026-09-19

Le bilan fait peau neuve, et en dit plus

### Added
- **Les personnes les plus vues** : les acteurs avec qui vous avez passé le plus de temps et les réalisateurs de ce que vous avez regardé, sous forme de rangée de portraits. La distribution et l’équipe sont lues avec la médiathèque et apparaissent après la première lecture de la médiathèque qui suit cette mise à jour.
- **Votre année en jours** : chaque jour de l’année sous forme d’un carré, d’autant plus lumineux que vous avez regardé, avec au-dessus votre plus longue série de jours et votre plus grosse journée. Survolez un jour, ou parcourez le calendrier avec les flèches, pour voir ce qu’il contenait.
- **En chiffres** : lectures, temps de visionnage, titres différents, jours de visionnage, plus longue série de jours et part de **revisionnage**, plus la répartition de l’année entre épisodes, films et musique.
- Le chapitre des genres indique combien de genres vous avez abordés et quelle part a pris le premier.

### Changed
- Un nouveau design : il s’ouvre sur votre année « rejouée », le titre à côté des affiches qui l’ont remplie, sur une forme d’onde de l’année avec une barre par semaine et la plus forte éclairée, et vous choisissez l’année juste au-dessus. Chaque chapitre a un titre dont le mot clé est en violet, une phrase qui porte les chiffres, et le mot du chapitre écrit en grand et en transparence derrière. Il se termine comme les films, par le générique : en vedette, réalisé par, projeté sur, durée.
- Les heures, les jours de la semaine et les mois partagent un seul chapitre **Rythme de visionnage**, avec un sélecteur pour passer de l’un à l’autre, et le titre suit ce que vous regardez (« Le vendredi l’emporte »).

### Fixed
- **En cours de lecture** sur le tableau de bord restait vide quand personne ne regardait, si bien que l’intitulé semblait appartenir aux filtres en dessous. Son encadré « Rien n’est en lecture pour le moment » est de retour.

## [0.7.10] - 2026-09-19

### Added
- Illustrations des médiathèques : chaque carte de médiathèque, et la page de la médiathèque elle-même, affiche l’image que Jellyfin utilise pour elle, via le proxy d’images de FinStats comme les affiches. Une médiathèque sans image garde son icône.

## [0.7.9] - 2026-09-19

### Changed
- La carte Genres s’ouvre de nouveau sur la **Liste**, le radar devenant la seconde option du sélecteur. Un choix déjà fait est conservé.

## [0.7.8] - 2026-09-19

### Added
- Une vue radar pour la carte Genres, sur le tableau de bord et sur les profils : un rayon par genre, avec un sélecteur **Radar** / **Liste** dans le coin de la carte, et le choix est mémorisé. Survoler un point, ou utiliser les flèches, affiche le temps de visionnage et la part de ce genre.

## [0.7.7] - 2026-09-19

### Fixed
- Les films appartenant à une collection apparaissaient comme « N’est plus dans la médiathèque ». Avec *regrouper les films en collections* activé, Jellyfin renvoie la collection au lieu des films qu’elle contient, si bien que FinStats ne les voyait jamais ; sur un serveur, cela masquait 556 des 1 238 éléments de la médiathèque Films. FinStats demande désormais les films eux-mêmes, et la relecture de la médiathèque que déclenche cette mise à jour les ramène et y rattache de nouveau leurs lectures.

## [0.7.6] - 2026-09-19

### Changed
- Sur les profils, la carte **Séries** descend juste au-dessus de Genres, si bien qu’une longue liste de séries ne repousse plus hors de vue le temps de visionnage, le graphique d’activité et les titres phares. Les séries de jours restent en haut, et une série que vous avez dépliée reste ouverte quand vous changez de période.

## [0.7.5] - 2026-09-19

### Changed
- La recherche compare mot par mot, dans n’importe quel ordre, avec n’importe quoi entre les mots, au lieu d’exiger l’expression exacte, si bien que « alya hides » trouve *Alya Sometimes Hides Her Feelings in Russian*. La casse, les accents et la ponctuation n’ont pas d’importance, un « The » initial est ignoré, et une faute de frappe est pardonnée (« mentalsit » trouve *The Mentalist*) ; les mots de trois lettres ou moins doivent toujours être tapés correctement. Les meilleures correspondances passent en premier, et la musique se trouve aussi par son artiste.
- Les champs de recherche d’Activité et du journal du serveur comparent mot par mot sur toutes leurs colonnes : « alya opera » trouve les lectures de cette série dans Opera.

## [0.7.4] - 2026-09-19

### Changed
- Les notes de version sont regroupées par série : chaque série `0.x` forme un bloc repliable qui contient ses versions, avec le sujet de la série, le nombre de versions et leurs dates. Seule la plus récente est ouverte au départ, et un bloc fermé montre quand même laquelle vous utilisez.

## [0.7.3] - 2026-09-19

### Fixed
- L’horloge d’En cours de lecture hésitait toutes les quelques secondes : l’actualisation toutes les 5 secondes la redessinait aussi, si bien que chaque fois que le serveur avait une fraction d’avance, le nombre avançait trop tôt puis restait figé une seconde. Seul le battement d’une seconde la fait désormais avancer, d’exactement une seconde (mesuré entre 999 et 1 001 ms d’écart), et l’actualisation corrige toujours après une pause, un saut ou une vraie dérive, en tombant sur le battement.

## [0.7.2] - 2026-09-19

### Added
- Le visionnage à plusieurs, en direct : chaque carte En cours de lecture indique « Avec maria » quand des personnes regardent la même chose ensemble, là où un groupe n’était auparavant reconnu qu’une fois les lectures terminées. En direct, des personnes comptent comme ensemble quand elles sont sur le même titre et ont soit commencé dans la fenêtre de groupe, soit atteint presque la même position, ce qui fonctionne même si FinStats a redémarré en cours de lecture.

## [0.7.1] - 2026-09-19

### Changed
- En cours de lecture compte chaque seconde : l’horloge avance et la barre glisse entre deux relevés, au lieu de sauter chaque fois que le serveur est interrogé. Le serveur est toujours interrogé toutes les 5 secondes et reste la source de vérité : l’affichage se resynchronise sur une pause, un saut ou quand le serveur est en avance, mais pas pour une position simplement en retard de quelques secondes, puisque les clients ne rendent compte qu’environ toutes les dix secondes. Les flux en pause restent immobiles.

## [0.7.0] - 2026-09-19

Qui regarde ensemble

### Added
- Le visionnage à plusieurs : quand différentes personnes lancent le même titre au même moment, FinStats le compte comme un visionnage ensemble. Une carte **Regardé ensemble** sur le tableau de bord montre les groupes, les titres qu’ils partagent et leurs heures ensemble, chaque profil montre avec qui cette personne regarde le plus, les lectures partagées portent une petite marque de personnes dans Activité, et les détails d’une lecture disent avec qui elle a été regardée. Cela fonctionne sur tout votre historique, lectures importées comprises.
- Un réglage pour l’écart maximal entre les débuts, 60 secondes par défaut : environ un tiers des vraies séances à plusieurs commencent avec 6 à 60 secondes d’écart. Les personnes doivent aussi continuer à regarder côte à côte pendant quelques minutes, si bien que deux personnes qui ouvrent le même épisode par hasard ne forment pas un groupe.

### Changed
- La recherche s’ouvre avec **Ctrl + Espace** au lieu de Ctrl/⌘ + K.

## [0.6.1] - 2026-09-19

### Added
- Les administrateurs peuvent ouvrir le bilan d’une autre personne, en haut de la page Bilan ou avec **Ouvrir le bilan** sur son profil. Il parle alors d’elle par son nom au lieu de « vous ».

### Changed
- Tous les autres ne voient toujours que leur propre bilan, quelles que soient leurs autorisations, et il n’existe toujours pas de bilan du serveur entier.

## [0.6.0] - 2026-09-19

Votre profil, et où vous en êtes dans chaque série

### Added
- La progression des séries sur les profils : chaque série que vous avez entamée sous forme de barre avec un segment par épisode (vu, commencé ou pas encore), dépliable saison par saison, triée selon votre proximité de la fin, avec des vues séparées pour les séries terminées et pour tout.
- **Marquer comme vu** sur votre propre profil, par épisode, saison ou série, pour ce que vous avez regardé quand rien n’enregistrait. Les marques de lecture propres à Jellyfin sont aussi utilisées. Les marques restent dans FinStats et ne changent jamais rien dans Jellyfin ; une lecture enregistrée ne peut pas être démarquée, vos propres marques le peuvent.
- Séries de jours sur les profils : votre plus longue suite de jours consécutifs avec une lecture, votre série en cours, et le nombre de jours où vous avez regardé quelque chose.
- **Mon profil** dans la barre latérale.
- Activité indique où la lecture s’est arrêtée (par exemple `44:15 / 45:00`) sous la barre de progression, et la date et l’heure sous « il y a 3 h ». Les lectures importées de Jellystat n’ont pas de position d’arrêt, parce que Jellystat n’en a jamais enregistré.

### Changed
- Les épisodes manquants ou pas encore diffusés ne sont plus traités comme faisant partie de votre médiathèque. Jellyfin les liste comme des emplacements réservés sans fichier, et ils comptaient dans le total des épisodes.

## [0.5.1] - 2026-09-19

### Added
- Un logo : un bouton de lecture découpé en trois barres de graphique, dans la barre latérale et sur les écrans de connexion, sous forme de SVG net avec un `favicon.ico` multi-tailles pour les navigateurs et barres de favoris qui en veulent un, plus une icône d’écran d’accueil pour les téléphones et tablettes.

## [0.5.0] - 2026-09-19

Vous décidez qui voit quoi

### Added
- Des autorisations sous Paramètres → Accès, pour tout le monde à la fois ou personne par personne : **Voir l’activité de tous** (les statistiques et l’historique des autres, la page Utilisateurs, toutes les lectures en cours), **Voir les détails réseau** (adresses IP, identifiants d’appareil, local ou distant), **Voir le serveur** (la page Serveur, le journal du serveur, les connexions échouées, les chemins de fichiers) et **Gérer FinStats** (paramètres, tâches, importation, suppression de lectures).
- La connexion personne par personne, pour laisser entrer quelques personnes sans ouvrir FinStats à tous les utilisateurs Jellyfin.
- Les changements s’appliquent immédiatement, y compris le retrait d’un accès, et tout est imposé par le serveur plutôt que par des boutons masqués.

### Changed
- Les administrateurs Jellyfin ont toujours tout et sont les seuls à pouvoir modifier les autorisations ; quelqu’un qui peut gérer FinStats ne peut rien accorder, ni à lui-même ni à personne d’autre.
- Le bilan de l’année reste personnel, quoi qu’on accorde.
- Rien ne change pour les installations existantes tant que vous n’accordez rien.

## [0.4.5] - 2026-09-19

### Changed
- FinStats est publié sous la **licence publique générale GNU v3.0**. Aucune version antérieure n’a jamais été publiée, donc rien n’est sorti sous d’autres conditions. Les licences des polices fournies sont livrées avec elles.

## [0.4.4] - 2026-09-19

### Added
- Échap revient en arrière : sur une médiathèque, un utilisateur ou un titre ouvert, il vous ramène d’où vous veniez, avec la liste exactement comme vous l’aviez laissée. Ce qui est ouvert se ferme d’abord (détails d’une lecture, recherche, liste déroulante, menu mobile), et Échap est laissé tranquille pendant que vous tapez ou utilisez un graphique au clavier.

## [0.4.3] - 2026-09-19

### Changed
- FinStats se souvient de ce que chaque page a affiché en dernier : revenir sur un profil, une médiathèque ou une période déjà consultés l’affiche immédiatement, photo de profil comprise, pendant que des chiffres frais se chargent derrière, et la page ne se redessine que si quelque chose a changé. Ce qui est mémorisé reste dans l’onglet du navigateur et est effacé quand vous vous déconnectez ou changez quoi que ce soit.
- Les espaces réservés de chargement n’apparaissent que lorsque le chargement est vraiment lent. Avant, chaque page les affichait au moins un tiers de seconde.

### Fixed
- Les médiathèques supprimées dans Jellyfin qui n’ont jamais eu de lecture ne sont plus listées. Une médiathèque supprimée qui a un historique est toujours affichée, marquée comme supprimée.

## [0.4.2] - 2026-09-19

### Fixed
- Sur la page Utilisateurs, l’étiquette **Admin** se trouvait sur sa propre ligne et rendait cette ligne plus haute que les autres. Les étiquettes se placent désormais à côté du nom.

## [0.4.1] - 2026-09-19

### Fixed
- Le mot « null » n’apparaît plus sous le titre du tableau de bord en dehors de décembre et janvier, quand il n’y a pas de bandeau de bilan à afficher.

## [0.4.0] - 2026-09-19

Voir ce qui a changé sans quitter l’application

### Added
- Un onglet **Notes de version** qui montre chaque version, avec celle que vous utilisez signalée.
- La version dans la barre d’état y renvoie, et l’onglet affiche un point après une mise à jour tant que vous ne l’avez pas consulté.

### Changed
- Rattacher l’historique aux éléments renommés est bien plus rapide : de 18 secondes à un tiers de seconde avec 666 titres orphelins, si bien que le démarrage reste instantané à mesure que l’historique grandit.

## [0.3.3] - 2026-09-19

### Changed
- La médiathèque est relue juste après la fin de la tâche **Scan Media Library** de Jellyfin, jamais selon un minuteur séparé et jamais pendant qu’une analyse est en cours. FinStats a toujours été en lecture seule vis-à-vis de Jellyfin, et il n’y lance jamais d’analyse.
- Une relecture hebdomadaire de sécurité couvre les serveurs qui reposent sur la surveillance en temps réel, où la tâche d’analyse peut ne jamais s’exécuter.
- L’ancien intervalle n’est plus qu’un secours, pour le cas où le suivi est désactivé ou le serveur ne signale aucune tâche d’analyse.

### Added
- Interrupteur **Suivre l’analyse de la médiathèque de Jellyfin** sous Paramètres → Collecte, activé par défaut.

## [0.3.2] - 2026-09-19

### Fixed
- L’historique survit aux renommages. Jellyfin donne un nouvel identifiant à un fichier renommé, ce qui laissait ses lectures précédentes en plan sous le nom brut du dossier (`Title (2010) [tmdbid-…] [imdbid-…]`), sans affiche. Ces lectures sont rattachées à l’entrée actuelle de la médiathèque : par identifiant TMDB/IMDb/TVDB quand l’ancien nom en contient un, sinon par titre nettoyé et année, et les épisodes par série plus numéros de saison et d’épisode. Une correspondance doit être sans ambiguïté, si bien que les titres réellement supprimés restent tels quels.
- Les chaînes de TV en direct importées de Jellystat étaient comptées comme des films, parce que Jellystat n’enregistre aucun type d’élément. Elles sont reconnues comme TV en direct, et les nouvelles lectures de TV en direct sont enregistrées avec le type propre à Jellyfin.

### Changed
- La TV en direct est entièrement exclue du bilan : classements, totaux, profil de spectateur, records et rang.

### Added
- `finstats relink` pour rattacher l’historique à la demande. Il s’exécute aussi au démarrage et après chaque lecture de la médiathèque.

## [0.3.1] - 2026-09-19

### Changed
- Le bilan est strictement personnel : chacun, administrateurs compris, ne voit que le sien, et la vue « Tout le monde », le sélecteur d’utilisateur et le chapitre du serveur disparaissent.
- Le bilan s’ouvre sur l’année « prête » : l’année en cours pendant décembre, sinon la précédente. Les autres années et « 12 derniers mois » restent à un clic.

## [0.3.0] - 2026-09-19

Votre année en revue

### Added
- Onglet **Bilan**, le récit défilant d’une année : heures regardées, séries, films, pistes et genres phares, et un profil de spectateur (oiseau de nuit, lève-tôt, pro du week-end, accro aux séries, cinéphile, mélomane, fidèle au rendez-vous) avec des graphiques par heure de la journée et par jour de la semaine.
- L’année mois par mois, avec le titre qui a marqué chaque mois.
- Records : plus grosse journée, plus long marathon, plus longue série de jours, plus longue lecture d’une traite, titre le plus revu, première lecture de l’année et titre le plus ancien regardé.
- Découvertes : nouvelles séries commencées, films et épisodes terminés, et les séries qui n’ont eu droit qu’à un seul épisode.
- Un bandeau d’une ligne sur le tableau de bord en décembre et janvier quand un bilan est prêt.

## [0.2.0] - 2026-09-19

Plus que Jellystat n’en a jamais enregistré

### Added
- Une chronologie pour chaque lecture : début, pause, reprise, saut, changements d’audio et de sous-titres, lecture directe qui devient un transcodage, et arrêt. Plus le nombre de pauses et de sauts, l’endroit où la lecture a repris, et si le spectateur était sur le réseau local.
- Onglet **Serveur** : version de Jellyfin, mises à jour et redémarrages en attente, utilisation du disque par médiathèque (Jellyfin 10.11+), extensions, résultats des tâches planifiées et chaque appareil enregistré.
- Analyses de **Lecture** : flux simultanés dans le temps, clients qui imposent des transcodages, jusqu’où les gens vont avant d’arrêter, lectures locales face aux distantes, et habitudes de visionnage.
- La composition de votre médiathèque : résolutions, codecs, plage dynamique, conteneurs, titres par décennie, ajouts par mois, les titres les plus volumineux, et les titres que personne n’a jamais regardés, vérifiés à la fois dans l’historique de FinStats et dans les marques de lecture propres à Jellyfin.
- Tableau de bord : pic de flux simultanés, estimation des données diffusées, part des lectures distantes, genres par temps de visionnage et connexions échouées.
- Pages des éléments : liens IMDb et TMDB, studios, profondeur de couleur et fréquence d’images, et qui a marqué le titre comme vu dans Jellyfin.
- Pages des utilisateurs : genres, et les films, épisodes et favoris que Jellyfin a enregistrés.

### Fixed
- Le libellé de durée d’une carte En cours de lecture ne déborde plus de la carte.
- Les libellés longs de la page Lecture étaient coupés trop tôt.
- Une série qui a quitté la médiathèque affiche toujours sa liste de saisons et d’épisodes.

## [0.1.0] - 2026-09-19

Première version

### Added
- Collecte en direct depuis les sessions de Jellyfin : utilisateur, titre, client, appareil, adresse IP, mode de lecture, raisons du transcodage et accélération matérielle, détails vidéo, audio et sous-titres, et temps regardé face au temps en pause.
- Connexion avec votre compte Jellyfin. Une configuration en deux étapes crée la propre clé API de FinStats, et les mots de passe ne sont jamais stockés. Les utilisateurs non administrateurs peuvent, en option, se connecter pour ne voir que leurs propres statistiques.
- Pages Tableau de bord, Activité, Utilisateurs, Médiathèques, Lecture et Journal du serveur, une palette de commandes (Ctrl/⌘ K) et une barre d’état en direct.
- Importation Jellystat : les grosses sauvegardes sont transférées en flux vers le disque et importées en quelques secondes, et importer deux fois le même fichier ne pose aucun problème.
- Un seul petit binaire avec une interface web intégrée et un seul fichier SQLite, plus une image Docker.
