# Novedades

Todo lo que ha cambiado en FinStats, de lo más reciente a lo más antiguo. Este archivo se compila dentro del
binario y es lo que muestra la pestaña **Novedades** de la aplicación.

Formato: `## [version] - date`, o `## [version] - first day to release day` para una versión preparada a lo
largo de varios días; después, `### Added`, `### Changed`, `### Security`, `### Performance`, `### Stability`,
`### Fixed` o `### Removed` (Añadido, Cambiado, Seguridad, Rendimiento, Estabilidad, Corregido, Eliminado), con
una viñeta por cambio. Seguridad es algo de lo que alguien podría haber abusado; Rendimiento, algo que ahora va
más rápido o pesa menos; Estabilidad, algo que ya no puede bloquearse, detenerse, acaparar memoria ni perder
datos; Corregido, algo que daba una respuesta equivocada. Una versión `x.y.0` lleva una línea de título corta
bajo su encabezado: ese título es el que la aplicación usa como titular de toda la serie.

## [2.3.1] - 2026-10-26 to 2026-10-28

### Security
- **La configuración pide un código de un solo uso**: hasta que está configurado, FinStats escribe un código en su registro al arrancar (`docker logs finstats`), y el asistente lo pide antes de probar una dirección o configurar nada. De lo contrario, quien llegara primero al puerto podía apuntar FinStats a un Jellyfin propio y ser su administrador, o usar la prueba para explorar la red.
- **El acceso a las bibliotecas sigue a Jellyfin**: cada persona ve solo las bibliotecas que Jellyfin le abre, y en ellas solo los títulos que Jellyfin le muestra según su límite de clasificación, sus etiquetas y sus reglas para lo no clasificado, en las páginas de biblioteca, la búsqueda, el reparto, las imágenes, Añadido recientemente, Mi lista, Próximamente, el feed del calendario y las notificaciones de títulos nuevos. Cualquier persona con sesión iniciada podía recorrer todas las bibliotecas.
- Un enlace podía llevar a alguien desde la bienvenida a otro sitio justo después de iniciar sesión. Ahora solo se siguen las páginas propias de FinStats.
- El registro del servidor y los inicios de sesión fallidos de las estadísticas daban las direcciones de inicio de sesión y la actividad de todos a cualquiera con *Ver el servidor*. Las líneas de otra persona requieren *Ver la actividad de todos*, y una dirección, *Ver detalles de red*, como en todas partes.
- Un destino de notificaciones personal podía llegar a la red local a través de un nombre que respondía de otra forma la segunda vez que se consultaba. Ahora se conecta a la dirección que se comprobó, más rangos que nunca llegan a internet cuentan como locales, también se rechaza la propia dirección pública de la casa, y un nombre que no se puede resolver se rechaza con las mismas palabras que uno local.
- Detrás de un proxy que añade una línea `X-Forwarded-For` propia, o desde una red IPv6, cada intento de inicio de sesión podía venir de una dirección nueva y el límite nunca se aplicaba. Ahora un /48 de IPv6 tiene 50 intentos en cinco minutos, un límite lleno rechaza una dirección nueva en lugar de olvidar una antigua, todo el servidor se detiene a los 60 fallos por minuto, y un inicio de sesión fallido se audita una vez por dirección y ventana.
- `FINSTATS_TRUST_PROXY` nombra los proxies (direcciones o rangos) cuyos `X-Forwarded-For`, `X-Forwarded-Host` y `X-Forwarded-Proto` se creen, y `1` o `true` ahora solo cree a un par en loopback o en una red privada. Activado, cualquier par podía elegir su propia dirección; desactivado detrás de un proxy, diez intentos fallidos dejaban fuera a todos.
- Una vez leídos los usuarios, un inicio de sesión con un nombre desconocido, desactivado, eliminado o sin acceso a FinStats se responde como contraseña incorrecta sin preguntar a Jellyfin, y no antes de lo que Jellyfin habría tardado en responder. Esos nombres podían bloquear cuentas de Jellyfin, saltarse la regla de Jellyfin sobre el acceso remoto u ocultarse tras la dirección de FinStats.
- Quien Jellyfin limita a la red de casa (*Permitir conexiones remotas* desactivado) solo inicia sesión en FinStats desde ella, ya que Jellyfin juzgaba ese ajuste por la dirección de FinStats, y una lectura de usuarios que no dice si alguien está desactivado lo deja fuera.
- Una cookie de sesión se marca como Secure siempre que la página donde se inicia sesión está en https, también detrás de un proxy que no lo indica.
- Un inicio de sesión correcto ya no perdona los intentos anteriores. En https la cookie de sesión es una cookie `__Host-` que un sitio con un nombre vecino no puede sustituir y que no se acepta con ningún otro nombre, y cerrar sesión termina todas las sesiones que envió el navegador.
- Solo una clave de calendario abre el feed del calendario desde su dirección; cualquier otra clave va en la cabecera.
- Una clave de API ya no puede conceder permisos, cambiar quién entra o qué es público, restaurar una copia de seguridad, ni añadir, cambiar o probar un servicio o un destino de notificaciones: cada cosa de esas dura más que la clave, así que requiere un navegador con la sesión iniciada.
- Toda respuesta lleva `Cross-Origin-Opener-Policy: same-origin` y una `Permissions-Policy`, las respuestas de la API llevan `Cross-Origin-Resource-Policy: same-origin` y son `no-store` salvo que digan otra cosa, y la CSP ya no permite imágenes `blob:`. Una página que abría FinStats podía convertir después su pestaña en una página de inicio de sesión propia.
- Donde FinStats conoce el esquema por el que se le llega (el `X-Forwarded-Proto` de un proxy de confianza, o la dirección pública), una escritura desde otro esquema se rechaza como entre sitios.
- `/auth/me`, la página de un título y Estado de la biblioteca enlazan a Jellyfin en su dirección pública; la dirección en la que el propio FinStats lee Jellyfin solo se da a los administradores.
- Solo se piden imágenes a Jellyfin de algo que FinStats tiene, incluidos los pósteres y tarjetas de un perfil publicado, así que no se pueden pedir por ID imágenes de otras bibliotecas.
- Las claves de un tracker privado también se ocultan en los errores de la cola de descargas, tanto por su forma como por su nombre, y a quien solo tiene *Ver qué se está descargando* se le dice qué servicio falló y cómo, nunca su dirección interna ni adónde redirige.
- La página Servidor nombra al último usuario de un dispositivo solo con *Ver la actividad de todos*, y Contenido sin vincular lo requiere además de *Gestionar FinStats*. Sin él, la sección Tareas cuenta solo las reproducciones y streams propios, una biblioteca eliminada se lista por las reproducciones propias, y un tablero de Tautulli en espera no se puede cancelar.
- Los ajustes muestran el servicio de dirección pública solo por su host, y la dirección de Jellyfin y la carpeta de datos solo a los administradores, y un título que la biblioteca perdió se describe solo a partir de las reproducciones que puede ver quien lo consulta.
- Una copia de seguridad de Tautulli cuyas tablas no eran tablas normales podía dejar esperando cualquier otra escritura hasta que FinStats se reiniciaba. Un archivo así se rechaza, y SQLite lee la copia de forma defensiva.
- Una subida de más de 4 GB se rechaza, y una copia de seguridad de Tautulli comprimida en zip no se descomprime más allá del tamaño que declara; cualquiera de las dos cosas podía llenar el disco donde está la propia base de datos de FinStats.
- Una respuesta de Jellyfin, Sonarr, Radarr, Seerr, la consulta de la dirección pública o la descarga de GeoIP no se lee más allá del máximo permitido, contado después de descomprimir.
- Cualquier usuario de Jellyfin podía hacer que FinStats dejara de registrar las reproducciones de todos informando de miles de reproducciones cortas de un título. Averiguar quién vio algo en compañía se hace ahora en una sola pasada, sin mantener ningún bloqueo mientras trabaja.
- Las reproducciones seguidas, o un título reproducido por turnos de un dispositivo a otro, ya no juntan a desconocidos en un grupo, y a quien solo puede verse a sí mismo solo se le habla de las personas junto a las que corrió su propia reproducción, y durante cuánto tiempo.
- Un enlace aún podía llevar a alguien desde la bienvenida a otro sitio tras iniciar sesión, mediante una ruta como `/.//`.
- Quien gestiona FinStats ya no puede cambiar cuántas copias de seguridad se guardan ni cada cuánto se escriben, la ventana de fusión (ahora una hora como máximo) ni cuándo se leen los usuarios, el registro de actividad y las copias de seguridad, y sus copias nunca desplazan las de un administrador; las direcciones de casa y las reglas de viaje requieren lo mismo que las alertas, e importar historial requiere además *Ver la actividad de todos*.
- Una clave, un token o una contraseña guardados solo se envían al servidor para el que se guardaron, y un destino de notificaciones solo se crea o cambia con la sesión iniciada, nunca con una clave de API. Un administrador puede desactivar o quitar el destino personal de otra persona, pero no redirigirlo, desactivar su comprobación de certificados ni leer su tema de ntfy o su clave de usuario de Pushover.
- Los webhooks de Discord y Slack deben ser `https`, ya que su dirección es su token.
- Una avalancha de inicios de sesión fallidos ya no puede ocultar uno real ni enterrar las alertas: los nombres que no son ninguna cuenta generan un aviso cada cuarto de hora que no nombra a nadie, las alertas salen primero, y Slack y Discord muestran el contenido de un mensaje como texto y no mencionan a nadie.
- El puesto en el Resumen anual, Capacidad, una notificación y un perfil publicado no cuentan más que las páginas: ningún puesto ni parte del tiempo de todos para quien solo puede verse a sí mismo, los momentos de más actividad solo como días de tres streams o más sin *Ver la actividad de todos*, ninguna tarea ni conexión fallida para quien no puede gestionar FinStats, y un perfil público cambia una vez al día.
- Una copia de seguridad de Tautulli con una columna generada se rechaza, y la imagen de una reproducción importada solo se pide cuando la biblioteca tiene su título.
- Una restauración nunca abre FinStats más de lo que está: los ajustes solo vuelven si se pide, el inicio de sesión, los permisos y los perfiles públicos pasan a ser lo que permiten a la vez esta instalación y la copia, y las direcciones públicas siguen siendo las de esta instalación. `finstats restore <file>` recupera solo el historial, como la página, y también los ajustes con `--with-settings`.
- Una restauración solo del historial ya no recupera direcciones de casa, alertas silenciadas, entradas de auditoría que se hacen pasar por propias de esta instalación ni títulos localizados que la biblioteca no tiene, y ninguna restauración recupera reproducciones, filas del registro o dispositivos con fecha anterior a 1990 o posterior a mañana. Una reproducción restaurada cuenta como importada sea cual sea la fuente que declare, así que un reinicio nunca la continúa.
- La carpeta de datos es solo del propio usuario de FinStats (0700, sus archivos 0600, `umask 077` en la imagen). FinStats se niega a ejecutarse como root sobre una carpeta de datos de otro usuario e indica el `docker exec -u` que hay que usar, y el punto de entrada cambia el propietario solo de la carpeta y de las entradas propias de FinStats.
- El nombre de un destino no puede contener caracteres de control, y el nombre de un dispositivo, una persona o un título ya no puede falsificar una línea del registro.
- El parámetro de ruta de un enlace ya no puede llegar a otra ruta de la API que la de su propia página, y el constructor de elementos de FinUI solo pone una dirección cuando es http(s) o relativa, nunca `srcdoc`.
- La búsqueda, los perfiles publicados y el trabajo iniciado a mano ya no pueden tener ocupado a FinStats: una búsqueda se corta en 200 caracteres, un perfil se guarda por idioma, las tarjetas esperan en una cola corta, y una tarea, la consulta de la dirección, la descarga de la base de datos de ubicaciones y un mensaje de prueba solo se inician cada cierto tiempo, se pidan como se pidan, contando los mensajes de prueba por remitente. Descargar la base de datos de ubicaciones requiere además lo que requiere leer la página Seguridad.
- El cuerpo de una petición debe seguir llegando una vez pasados sus primeros 15 segundos, una respuesta que nadie lee se abandona a los 30 segundos y una dirección mantiene como mucho 64 conexiones, salvo un proxy de confianza. Unos cientos de peticiones que nunca enviaban su cuerpo podían impedir que FinStats respondiera a nadie.
- Un servidor de correo no se lee más allá de lo que puede ocupar una respuesta (1 MB para una carta entera), y un mensaje de prueba se detiene tras el tiempo que puede tardar uno en cola. Un servidor que nunca terminaba una línea podía hacer que FinStats retuviera cada vez más memoria hasta que se detenía.
- La lectura de una biblioteca se juzga por los títulos que las lecturas han guardado, no por los que trajo una importación, y un título de Jellystat en una biblioteca ya leída entra como desaparecido. Una importación de títulos inventados podía hacer que cada lectura de biblioteca detuviera FinStats, en cada arranque.
- El límite de 60 inicios de sesión fallidos por minuto en todo el servidor solo cuenta los intentos enviados a Jellyfin, incluidos los que aún esperan su respuesta. Los nombres que FinStats rechazaba por sí mismo permitían a cualquiera con unas pocas direcciones impedir que nadie iniciara sesión, y muchos intentos enviados a la vez llegaban todos a Jellyfin.
- Quien Jellyfin limita a la red de casa sigue limitado en cada petición, su sesión y sus claves incluidas, no solo al iniciar sesión; casa es una red privada o la dirección pública del hogar tal como la encontró la consulta, nunca una dirección escrita como de casa, y tras un proxy que `FINSTATS_TRUST_PROXY` no nombra se queda fuera. Una cookie o clave sacada de casa abría todo desde cualquier lugar, y un gestor podía abrir la casa a la dirección que quisiera.
- Una clave de API ya no puede desactivar lo que notaría que se abusa de ella, ni las copias de seguridad que desharían lo que hizo: cuántas copias se guardan y cada cuánto, la ventana de fusión, las direcciones de casa y las reglas de viaje, las programaciones de usuarios, registro de actividad y copias, silenciar o resolver todas las alertas, una copia a la papelera o quitar un destino de notificaciones. Cada uno sobrevivía a la clave una vez revocada.
- Los pósteres de Sonarr y Radarr, y el próximo episodio de una lista de deseos, siguen también el acceso a las bibliotecas: una serie que Jellyfin oculta a alguien no tiene para esa persona ni póster ni fecha. Contar los identificadores de los servicios revelaba qué series ocultas llegaban, y una lista nombraba el próximo episodio de una serie oculta.
- El capítulo En compañía del resumen solo nombra a las personas cuya reproducción coincidió con la tuya, como hace la página En compañía. Un grupo al que llegaste por otra persona nombraba a gente con la que nunca viste nada.
- Una importación de Jellystat o Streamystats lee todo su archivo antes de tomar el bloqueo de escritura de la base de datos y después solo escribe las filas que conserva. Leído bajo el bloqueo, un archivo de unas pocas filas reales y gigabytes de cualquier otra cosa dejaba esperando cualquier otra escritura, reproducciones e inicios de sesión incluidos.
- Una restauración guarda un permiso por persona, para no más personas que un hogar muchas veces, y busca duplicados de una entrada de auditoría sin leer cada entrada que acaba de escribir. Una copia de seguridad fabricada podía retener memoria y el bloqueo de escritura de la base de datos cada vez más tiempo durante la restauración.
- Una restauración con sus ajustes activa la consulta de la dirección pública y la descarga de la base de ubicaciones solo donde esta instalación también las tiene activadas, y nunca guarda menos copias de seguridad ni las escribe con menos frecuencia que esta instalación. Una copia que guardaba una podía hacer que la siguiente ejecución quitara todas las demás.
- Un tamaño de archivo que ningún disco podría contener se lee como ningún tamaño, y los tamaños se suman de modo que ningún total pueda desbordarse. Un solo tamaño importado hacía que las páginas de bibliotecas respondieran con un error para siempre.
- En cuanto FinStats sabe que se llega a él por https (su dirección pública es https, o un proxy de confianza lo dice), una sesión solo se toma de su cookie `__Host-`, y con una dirección pública https cada sesión nueva es una sesión https. Un inicio de sesión enviado sin `Origin` a través de un proxy que no decía nada creaba una sesión normal que un sitio vecino podía colar en el navegador de alguien.
- La imagen se construye con cada acción de GitHub fijada a un commit y las dos imágenes base a un resumen, movidos solo por Dependabot, y una versión se construye sin la caché de compilación compartida. Una etiqueta movida en la acción de otra persona podría haber publicado cualquier imagen como FinStats.
- La hoja de estilo de FinUI, dibujada con el aspecto propio de quien visita, solo pueden cargarla las páginas de FinStats. Una página en otro puerto del mismo equipo podía leer en ella si la persona había iniciado sesión y qué ajuste preestablecido eligió.
- Cada inicio de sesión rechazado se responde en un mismo momento contado desde que se envió, lo haya rechazado FinStats o Jellyfin, y un nombre se reconoce sin importar mayúsculas o minúsculas, también más allá de la A a la Z. Lo rápido que llegaba un rechazo revelaba qué nombres eran de alguien, y «Åse» escribiendo `åse` era rechazada sin preguntar a Jellyfin.

### Performance
- Un perfil publicado se calcula como mucho una vez por minuto, no de nuevo para cada imagen y tarjeta que contiene.
- El historial de una persona se analiza en busca de alertas de seguridad una sola vez a la vez, por muchas reproducciones que empiecen.
- Iniciar o cerrar sesión, revocar una clave o cambiar los permisos de una persona solo pide que vuelvan a comprobar las conexiones en directo a las que afecta.
- El tablero de conexiones de Tautulli lee el historial una vez, no una vez por usuario de Plex.
- Una imagen que falta en la caché se pide con una sola solicitud mientras las demás esperan, y una que el origen no pudo dar no se vuelve a pedir durante 30 segundos.
- Los archivos de la interfaz web se comprimen con gzip una vez y se guardan, y las fuentes no se vuelven a comprimir.

### Stability
- Una copia de seguridad en el formato antiguo de Jellystat se lee fila a fila en lugar de entera, y una línea más larga que la de cualquier copia se rechaza.
- Dos subidas de Tautulli a la vez ya no escriben en el mismo archivo.
- Un número de página muy posterior al final de los títulos de una biblioteca o de Estado de la biblioteca da una página vacía, no un error.
- Una importación o una restauración cuya subida se cortó ya no rechaza todas las demás hasta un reinicio, y ni ella ni una importación abandonada antes de empezar deja un archivo atrás.
- Una fila de una copia de seguridad no se lee más allá de 8 MB, y solo llega una copia de seguridad de Tautulli a la vez.
- Una reproducción guarda como máximo 2.000 eventos, y una importación omite una reproducción que no empieza y termina entre 1990 y mañana.
- Un solo número de episodio muy por debajo de cero ya no bloquea todas las lecturas de la biblioteca: Estado de la biblioteca (también su resumen), Capacidad, la revinculación y la localización suman y comparan números sin desbordarse, y una compilación de release comprueba los desbordamientos, así que lo que se haya pasado por alto detiene una tarea en lugar de quedarse en bucle.
- Un número de temporada, de episodio o final solo se guarda donde Jellyfin podría contenerlo (de 0 a 2.147.483.647), venga de Jellyfin, de Jellystat o de Tautulli.
- Una copia de seguridad de Tautulli con IDs repetidos se rechaza, porque unos pocos MB de filas bajo un mismo ID dejaban el proceso sin memoria, y una copia se abre como inmutable, sin dejar `-wal` ni `-shm` atrás.
- Una conexión debe enviar su solicitud en 15 segundos y como máximo se atienden 512 a la vez: unos cientos de sockets medio abiertos dejaban FinStats fuera de servicio. Una trama del canal en directo se limita a 4 KB.
- Unos pocos kilobytes de Sonarr, Radarr o Seerr ya no pueden convertirse en gigabytes de memoria: una lista no guarda más de lo que FinStats lee de ella.
- Un destino de notificaciones que nunca responde solo se retrasa a sí mismo: los destinos se atienden en paralelo, y uno que agota el tiempo una y otra vez se deja en paz durante diez minutos.
- El tiempo restante y la velocidad total de una cola, los números de una reproducción restaurada y el día en que una reproducción sale de la papelera ya no pueden desbordarse.
- Una restauración lee el archivo entero antes de tomar el bloqueo de escritura, rechaza uno que se descomprime en mucho más que su tamaño, y escribe por lotes, así que el recolector sigue registrando mientras tanto.
- Las copias de seguridad se nombran en UTC y nunca sobrescriben un archivo existente, y solo se podan las copias previas a una actualización hechas por el propio FinStats.
- Una fila del registro del servidor con fecha futura ya no impide leer el registro de actividad: se lee a partir de su fila más reciente que no esté en el futuro.
- Las solicitudes que llegan a la vez a un enlace publicado calculan su respuesta una sola vez, y sus tarjetas se dibujan en un carril propio, así que quien tenga el enlace ya no puede hacer que las tarjetas del Resumen anual del propietario respondan 503.
- Las reproducciones de una biblioteca que falta en la lista de bibliotecas de Jellyfin esperan dos días antes de pasar a la copia del mismo título en otra biblioteca. Una sola lista que dejaba fuera una biblioteca las movía al instante, y seguían movidas cuando la biblioteca volvía.
- Un disco que Jellyfin declara más grande de lo que cabe en un número ya no hace fallar cada lectura de los detalles del servidor.

### Fixed
- Los inicios de sesión fallidos en una cuenta son un solo aviso por ventana que se convierte en alerta a los veinte, y un nombre solo cuenta contra una cuenta cuando se escribe entero («x alice» ya no cuenta contra alice).
- Los mensajes de Discord y Slack se escapan primero y se recortan después: un nombre de dispositivo lleno de Markdown hacía que Discord rechazara toda alerta que lo nombrara.
- Se acepta una dirección IPv6 pública para un destino personal.
- Detrás de un proxy que inicia la sesión de la gente por sí mismo (`auth_basic` de nginx), la cookie de FinStats ya no se rechaza.
- Un nombre visible que contiene `{{url}}` se muestra tal como está escrito en la vista previa de una página publicada.
- Un resumen pedido para el año 9999 muestra el año por defecto en lugar de un error.
- Una página en directo cuyo permiso se retiró y se devolvió mientras estaba abierta vuelve a mantenerse al día sin recargarla.

### Removed
- **Synnmørsk** ya no es uno de los idiomas de FinStats: un navegador que lo había elegido lee nynorsk, la lengua escrita de la que surgió.

## [2.3.0] - 2026-10-07 to 2026-10-26

Capacidad, un canal en directo, el modo sencillo e idiomas

### Added
- **Idiomas**: FinStats habla 23 idiomas. El inglés británico y estadounidense y el noruego bokmål y nynorsk los escribe y revisa el mantenedor; el danés, alemán, español, finés, francés, hindi, islandés, italiano, japonés, coreano, neerlandés, polaco, portugués (Brasil), ruso, sueco, turco, ucraniano y chino (simplificado y tradicional) son traducciones que nadie aquí ha revisado, y dicen en inglés lo que aún no tienen. Todas las páginas de ambos diseños, el perfil público, las Novedades y lo que el servidor responde con palabras (un error, las tarjetas del Resumen anual, lo que hace una tarea de Jellyfin) aparecen en el idioma elegido en Ajustes → Apariencia → Idioma, por navegador; mientras no se elija ninguno, decide el idioma del propio navegador. Los números, las fechas, las unidades y las listas se escriben como los escribe ese idioma. Un script puede pedir uno con `Accept-Language`.
- **Modo sencillo**: un segundo diseño para quienes vienen de Plex, organizado como un servidor clásico de estadísticas de Plex. Una barra en la parte superior en lugar del menú, y solo las páginas de estadísticas: Inicio, Bibliotecas, Usuarios, Historial, Gráficos y Ajustes, con un engranaje para tu perfil, Ajustes, Novedades, el tema y cerrar sesión. Se elige por navegador en Ajustes → Apariencia → Diseño. Carga solo su propia mitad de FinStats, así que arranca más rápido; Resumen anual, En compañía, En camino, Seguridad y las páginas del servidor siguen en el diseño FinStats. Lleva los colores propios de FinStats, en claro y en oscuro, y nada en él se mueve.
- **Inicio del modo sencillo**: lo que se está reproduciendo, con una tarjeta por reproducción (cómo se reproduce, desde casa o desde fuera, cuándo termina); las estadísticas de visualización como tarjetas clasificadas por número de reproducciones o por duración en los últimos N días; lo que contiene cada biblioteca y lo que ha llegado últimamente.
- **Gráficos del modo sencillo**: reproducciones por tipo de contenido (día a día, por día de la semana y hora, las diez aplicaciones y personas principales), por cómo se reprodujeron (día a día, el máximo de reproducciones a la vez de cada día, la resolución del archivo y la enviada, aplicaciones y personas) y mes a mes, por número de reproducciones o por duración, de todo el mundo o de las personas elegidas.
- **Bibliotecas, Usuarios y títulos en el modo sencillo**: tablas de cada biblioteca y cada persona con lo último que se reprodujo, en qué y desde dónde (la dirección, solo con *Ver detalles de red*), y una página para cada biblioteca, persona y título con sus cifras y su propio historial.
- Para scripts: `/api/stats/split` (reproducciones por aplicación, persona, resolución, día o mes, desglosadas por tipo de contenido o por cómo se reprodujeron); listas de los más vistos por espectadores distintos (`sort=users`); la actividad limitada a las reproducciones vistas (`watched=1`); solo las estadísticas pedidas (`parts=`); el máximo desglosado en transcodificaciones, transmisiones directas y reproducciones directas, también por día; lo que contiene cada biblioteca y lo último que se reprodujo en ella; lo añadido recientemente por tipo (`kind=`); el mapa de calor por tipo de contenido; la última reproducción de una persona; si una reproducción se hace desde casa.
- **Un canal en directo**: el panel, las descargas de En camino y la barra de estado se mantienen al día a través de un único WebSocket por pestaña (`/api/live`), así que lo que se reproduce y lo que se descarga cambia en pantalla en el momento en que cambia, y no en la siguiente consulta. Detrás de un proxy inverso que no deja pasar las conexiones WebSocket, o en una conexión que se ha cortado sin avisar, las páginas preguntan cada pocos segundos, como antes. `/api/now-playing`, `/api/downloads` y `/api/summary` siguen igual, para los scripts.
- **Servidor → Capacidad**: ¿aguantará este servidor?, según lo que FinStats ha visto. Los momentos de más actividad registrados y los de los últimos 90 días: reproducciones a la vez y cuántas transcodificaron la imagen con hardware, sin él o sin que conste cómo (un historial importado de Tautulli es de Plex y queda fuera). Cada disco en el que están las bibliotecas y a qué ritmo lo llena lo que Jellyfin ha añadido en seis meses (un disco que empezó a llenarse hace poco se mide a partir de un mes después de su primer archivo, porque su primer mes es el de llenarlo) y, donde Jellyfin 10.11 o posterior informa del espacio libre, cuánto durará a ese ritmo. FinStats nunca dice cuántas reproducciones podría aguantar el servidor.
- **Exportar .ics** en En camino → Próximamente: la lista tal como la has filtrado, descargada como archivo de calendario para importarla una vez. No se actualiza sola; una clave de calendario de Ajustes → Claves de API sigue dando un calendario que sí lo hace.
- **Merece un vistazo**: Capacidad avisa cuando, aun siendo frecuente la transcodificación, todas las transcodificaciones de las que Jellyfin informó en 90 días se hicieron sin aceleración por hardware, cuando el escaneo de biblioteca de Jellyfin está en marcha una cuarta parte del tiempo o más, y cuando a un disco le quedan menos de seis meses de espacio. Cada aviso lleva sus cifras y se puede apartar, como los hallazgos de Estado de la biblioteca.
- **De qué está hecha tu biblioteca** se puede limitar a una biblioteca o a varias, en Bibliotecas y en la página de cada biblioteca (`library_id=a,b`).
- **Formatos de fecha y hora** en Ajustes → Apariencia: una fecha como 31.01.2026, 31/01/2026, 31-01-2026, 01/31/2026, 2026-01-31, 2026.01.31 o 2026/01/31, y una hora de 24 o 12 horas, con o sin cero delante, o cada una como la escribe el idioma. Se guarda en este navegador.
- **Una bienvenida**: la primera vez que alguien inicia sesión, en cualquier navegador, y al final de la configuración, FinStats pregunta el idioma, los formatos de fecha y hora y el diseño, con una imagen de cada uno. Omitir deja inglés británico, 31/01/2026, 13:05 y FinStats.

### Changed
- El logotipo dentro de una página es la marca sola junto a su nombre, escrito en Inter sea cual sea la fuente que elija un preset; donde aparece solo (el favicon) conserva su mosaico redondeado.
- Una reproducción transcodificada por software se registra como tal cuando Jellyfin lo indica; antes se veía igual que una de un servidor que no lo dice. Las páginas siguen llamando Software a ambas.
- Ahora una actualización ejecuta, una sola vez y empezando por la versión más antigua, lo que cada versión necesita hacer en una base de datos escrita por un FinStats anterior, además de sus migraciones. La de 2.3.0 lee los datos del servidor en el primer arranque, así que Capacidad está ahí desde el principio.

### Performance
- **Una página abierta no pide nada**: el panel hacía 30 peticiones por minuto para mantenerse al día y las descargas de En camino, 18; con el canal en directo, ninguna hace ni una. Haya las pestañas abiertas que haya, FinStats no pide a Jellyfin, Sonarr ni Radarr nada más que antes, y una pestaña oculta sigue permitiendo que la cola de descargas se consulte con menos frecuencia.
- La página de una persona lee solo las reproducciones de esa persona: sus totales y su última reproducción se calculaban sobre todas las reproducciones del servidor y luego se limitaban a ella, unos cuatro segundos con un millón de reproducciones.
- La página de una biblioteca lee solo los títulos y reproducciones de esa biblioteca, cuando antes contaba los de todas y se quedaba con una: un tercio del tiempo con un millón de reproducciones.
- Con el reloj de Jellyfin por delante del de FinStats, *Después del escaneo de biblioteca de Jellyfin* volvía a leer toda la biblioteca cada minuto hasta que pasaba la diferencia; ahora cada escaneo se sigue una sola vez.
- Un archivo GeoIP que no se podía abrir se volvía a copiar (cien megabytes o más) y se volvían a situar todas las direcciones, cada cuarto de hora; ahora solo se reintenta cuando cambia.
- Estado de la biblioteca se abre al instante después de dejar el puntero sobre su enlace: lo que se había cargado por adelantado se guardaba donde la página nunca miraba.

### Stability
- Una actualización que fallaba después de ejecutar una migración, y que Docker reiniciaba enseguida, hacía una nueva copia de la base de datos a medio actualizar en cada arranque, y tres arranques eliminaban la única copia que podía abrir la versión anterior. Ahora se conserva la copia de antes de la actualización.
- Un título eliminado en Jellyfin mientras se leía la biblioteca (un archivo sustituido por Radarr) podía hacer que la lectura se saltara otro título que seguía ahí y lo marcara como eliminado, y sus reproducciones podían pasar entonces a otra copia de él. Lo que una lectura no ha visto ahora se consulta primero por su ID.
- Restaurar una copia de seguridad hecha antes de renombrar o mejorar un archivo restauraba por segunda vez todas las reproducciones movidas desde entonces al archivo nuevo, duplicadas para siempre.
- Una solicitud de la respuesta de Seerr que FinStats no podía leer (un campo renombrado en una actualización de Seerr) quedaba marcada como eliminada en la lectura completa diaria.
- Una tarea que se topaba con un error interno se quedaba «en marcha» para siempre y ningún disparador volvía a iniciarla hasta reiniciar FinStats; ahora termina como fallida.
- Un zip de Tautulli dañado detenía la subida antes de eliminar el archivo, que contiene los tokens de todos los usuarios de Plex, hasta la limpieza de las seis horas.
- Una importación o una restauración interrumpida mientras llegaba su archivo dejaba ese archivo en la carpeta de datos para siempre, direcciones incluidas; ahora se elimina en el siguiente arranque.
- Una imagen podía servirse a medio escribir mientras se guardaba en caché, y el navegador se quedaba entonces con ella así.
- Las claves de API revocadas se guardaban para siempre, por muchas que se crearan y revocaran.
- Una lectura de las marcas de visto y favorito de alguien que volvía vacía o ilegible las borraba todas hasta la siguiente lectura; ahora se rechaza, como una lectura vacía de una biblioteca.
- Un episodio con un número final que ningún episodio puede tener (una fecha escrita ahí) podía mantener ocupado el Estado de la biblioteca sin fin tras una lectura de la biblioteca.

### Fixed
- Una nota de versión con código dentro de una negrita (las de 2.0.2 y 2.0.3) mostraba las comillas invertidas en lugar de dar formato al código.
- Un gráfico de recuentos podía numerar su eje 0, 3, 5, 8, 10, o incluso 0, 0, 1, 1, 1: pasos de 2,5 escritos redondeados. Los recuentos suben ahora en pasos enteros.
- Una clave de calendario creada por alguien que no es gestor venía con un enlace a la dirección del propio navegador en lugar de a la dirección pública de FinStats, así que solo funcionaba desde dentro; además, la página pedía unos ajustes que se le denegaban.
- La última reproducción de una persona podía durar unos segundos menos de lo necesario para contar como reproducción en cualquier otro sitio, y nombraba un título que su Historial no muestra.
- Una restauración desde Eliminadas recientemente que fallaba no decía nada y dejaba su botón girando; lo mismo pasaba con un *Recuperar* fallido en Estado de la biblioteca y con un *Reabrir* o un *Resolver todas* fallidos en Seguridad.
- El feed de calendario se llama FinStats en una aplicación de calendario, no finstats.
- La cola de descargas mostraba quién había pedido cada descarga a cualquiera que pudiera verla; ahora la solicitud de otra persona requiere *Ver la actividad de todos*, como en En camino.
- Quitar *Recibir notificaciones* no detenía lo que se enviaba a los destinos de esa persona, y además ya no podía desactivarlos.
- Una reproducción pausada y reanudada dentro de la ventana de fusión se notificaba como detenida una vez, y su reanudación y su final real, nunca.
- Las reproducciones de una pista podían pasar a una pista con el mismo nombre de otro artista («Intro») cuando su álbum salía de la biblioteca.
- Un destino de notificaciones escrito como una dirección IPv4 dentro de una IPv6 (NAT64, 6to4) pasaba por pública aunque llegaba a la red local.
- Una conexión de Sonarr o Radarr apuntada a otro servidor seguía mostrando durante una semana los pósteres del servidor anterior.
- Las alertas de Seguridad decían que no había nada que revisar cuando se elegían dos personas o más, y una página que se quedaba vacía al resolver su última alerta no tenía forma de volver atrás.
- Todas las claves de API que no habían caducado decían que caducaban «justo ahora».
- Un cuadro de diálogo seguía sobre la página después de volver atrás, y la página no se desplazaba hasta cerrarlo.
- El registro de auditoría mostraba ocho tipos de entrada (una reproducción o una copia de seguridad recuperada, la papelera vaciada, los perfiles públicos, un cambio de apariencia) por su ID, sin nada en Detalle.
- Añadir un título a Mi lista desde su menú contextual fallaba sin decir nada cuando el servidor lo rechazaba.
- *Visto en compañía* en el panel trataba a varias personas elegidas a la vez como una sola.
- El resumen anual escribía las horas y los días por debajo de diez con la coma decimal del navegador junto a sus propias comas de millares.
- Capacidad decía que un disco se llenaría "al ritmo de los últimos 0 meses".
- En la página Seguridad, las direcciones sin lugar contaban las reproducciones de todos desde siempre, fuera cual fuera la persona y el periodo al que se limitaba la página.
- Alguien con *Gestionar FinStats* pero sin *Ver la actividad de todos* podía mandar a la papelera la reproducción de cualquiera.
- Una tarjeta del Resumen anual o de un perfil público podía salir de la caché del navegador en el idioma elegido antes.
- Las descargas que enviaba el canal en directo estaban en inglés británico, fuera cual fuera el idioma de la página.
- Un inicio de sesión que cambiaba lo que alguien puede ver llegaba a sus otras pestañas abiertas solo en la siguiente comprobación de medio minuto.
- Una pestaña que cedía su conexión en directo a otra más nueva recuperaba una en su siguiente página, de modo que las pestañas de una persona se cerraban unas a otras sin parar.
- Las tablas ordenaban como texto los tamaños de 1000 a 1023 unidades en los idiomas que agrupan cifras con un espacio, y lo mismo los porcentajes en turco.
- Un título con un marcador en su nombre, como "Big {device} Bunny", se rellenaba en los mensajes del servidor.
- Las tablas ordenaban como texto las duraciones y las cuentas sobre un total (5 de 26) en todos los idiomas salvo el inglés y los nórdicos.
- Una pestaña aún abierta en un idioma recibía al servidor en otro en cuanto otra pestaña lo elegía.
- Una palabra junto a un número tomaba la forma equivocada en polaco, ruso y ucraniano ("5 wieczory", "2 потоков").
- Capacidad contaba como crecimiento el primer llenado de una biblioteca y dividía lo que añadía cada biblioteca entre seis meses, fuera cual fuera el periodo medido; un archivo que ocupaba el lugar de otro (una mejora de Radarr o Sonarr, o un archivo al que Jellyfin dio un id nuevo) contaba como nuevo entero, cuando solo la diferencia lo era.
- Quienes veían juntos con SyncPlay no formaban un grupo cuando uno se unía minutos después del otro; las reproducciones que terminan a la vez en el mismo punto van juntas ahora, y el historial se agrupa de nuevo una vez.
- Alguien sin *Ver la actividad de todos* podía deducir de *Nunca vistos* de una biblioteca que otra persona había reproducido un título; ahora cuenta solo lo que ha visto esa persona.

## [2.2.0] - 2026-10-01 to 2026-10-07

Estado de la biblioteca y FinUI

### Added
- **Un menú a tu gusto en el ordenador**: Ajustes → Apariencia ofrece ocho: la barra lateral de siempre, otra agrupada según para qué sirven las páginas, un riel de iconos que se despliega como barra lateral bajo el puntero, una barra lateral que empieza por la búsqueda, un dock, una barra de comandos, páginas fijadas arriba y mosaicos de colores. Las barras laterales pueden ir a la derecha. El dock se esconde mientras te desplazas hacia abajo y vuelve cuando te desplazas hacia arriba o señalas el borde inferior de la ventana; con él, la barra de estado pasa arriba.
- **Un menú a tu gusto en el móvil**: una barra de pestañas con Más (la nueva opción predeterminada), una hoja que se abre hasta la mitad, la pantalla completa, las páginas en abanico alrededor del pulgar o una barra de direcciones que se convierte en el menú. Cada uno busca entre sus páginas y pasa cualquier otra palabra a la búsqueda.
- Ambos los guarda el navegador en el que se eligieron, así que un móvil y un ordenador conservan cada uno el suyo; los dos son componentes de FinUI, `desktop-nav` y `mobile-nav`.
- **Un menú contextual**: haz clic con el botón derecho en un título, una persona o una biblioteca en cualquier parte de FinStats (un póster del panel, una fila de una lista, un nombre en una tabla, una película de Próximamente que aún no está en la biblioteca), o mantén el dedo sobre ello, o pulsa la tecla de menú, y un menú surge del punto que has pulsado: abrirlo, abrirlo en una pestaña nueva, añadir una película o una serie a Mi lista, abrirlo en Jellyfin o en TMDB y TVDB, copiar su enlace. Mayús con el botón derecho sigue siendo el menú del propio navegador. El `context-menu` de FinUI.
- **Búsqueda sin cuadro de diálogo**: la búsqueda surge de tu menú (una barra lateral se ensancha hasta convertirse en ella, el dock se eleva hacia ella, el nombre de la barra de comandos se transforma en ella, el menú del móvil la contiene) o, si lo eliges en Ajustes → Apariencia, ocupa el lugar de la página mientras el menú se queda. Ctrl+Espacio la abre como antes; la ventana por encima de todo ha desaparecido. El `search` de FinUI.
- **Todo lo que hay en una biblioteca**: la tarjeta Añadido recientemente de una biblioteca ofrece *Todo su contenido*, cada película, serie o álbum que contiene como pósteres, por nombre, año, fecha de adición o tamaño, filtrado según escribes. Llegan más al desplazarte.
- **Servidor → Estado de la biblioteca**: FinStats compara cada archivo con sus vecinos y enumera lo que destaca, cada cosa con las cifras que la respaldan. Solo informa; en Jellyfin no se cambia, elimina ni vuelve a escanear nada.
- **Huecos**: episodios que faltan entre dos archivos de una temporada («Temporada 2: faltan los episodios 4 y 5 entre el 3 y el 6»). Un archivo que contiene dos episodios cuenta como ambos; los especiales y el final de una temporada se dejan en paz.
- **Temporadas distintas** en resolución, HDR o códec del resto de su serie, y **episodios raros**: un archivo distinto del resto de su temporada, como un episodio SD en una temporada a 1080p. La resolución se mide por el ancho o por el alto, así que una película recortada a 1920×800 es 1080p.
- **Copias y versiones**: la misma película o el mismo episodio dos veces, según los ID de proveedor que ya usan Mi lista y En camino. Dos copias en la misma resolución muestran el espacio ocupado dos veces; una copia 4K junto a una 1080p se presenta como dos versiones y nunca se cuenta como desperdicio.
- **Archivos ligeros**: una tasa de bits muy por debajo de lo que suelen necesitar su resolución y su códec («4K HEVC a 1,1 Mbps»); los episodios ligeros de una temporada forman un solo hallazgo.
- **Doblajes que se cortan**: un idioma de audio que algunas temporadas tienen en todos los episodios y otras en ninguno.
- **Nunca identificados**: películas y series que Jellyfin emparejó sin ID de TMDB, TVDB ni IMDb.
- **Descartar** un hallazgo que está bien a propósito, con una nota. Vuelve solo cuando cambian sus archivos. Pueden descartar quienes pueden gestionar FinStats y ver el servidor; los descartes quedan en el registro de auditoría y forman parte de una copia de seguridad.
- La página de una biblioteca indica cuántas cosas hay que revisar en ella y lleva hasta ellas.
- **FinUI**, los componentes con los que está hecho FinStats, es un proyecto propio (`github.com/finstats/finui`): todos los botones, tarjetas y controles en todos sus estados y en ambos temas en `finui.finstats.no`, y **FinUI create** para cambiar sus colores, el radio, la densidad y más mientras lo ves, y llevártelo con un solo comando. **Ajustes → Sistema** lleva hasta allí.
- **Ajustes → Apariencia**: cada persona elige cómo ve FinStats mediante el código de un preset de FinUI: color base, acento, colores de los gráficos, contraste, radio, densidad, bordes, tarjetas, botones, campos, tablas, resaltado, movimiento, iconos, el menú, la página que hay detrás de todo, el anillo de foco, cómo se componen los encabezados y las fuentes del texto, los encabezados y los números. Tu elección es solo tuya; muestra lo que significa el código y lleva a FinUI create para cambiarlo.
- **Estilos**: once aspectos completos para empezar con una sola elección en FinUI create, cuyo código acepta Ajustes → Apariencia. Son Washi (FinStats tal como viene), Studio, Noir, Matinee, Console, Pastille, Ledger, Gazette, Arcade, Canopy y Harbour. FinStats en sí trae un único aspecto y no ofrece ninguna lista de estilos.
- **Iconos que se mueven**: cada icono de un enlace, un botón o una pestaña hace su propio movimiento una vez al señalarlo (actualizar gira, la flecha de descargar cae en su bandeja, la papelera levanta la tapa), y un botón de actualizar pulsado sigue girando hasta que llega su respuesta. Quietos con movimiento reducido y con Movimiento: Desactivado.
- **Movimiento, con FinMotion**: los componentes de FinUI ahora se mueven: el botón de un interruptor salta al otro lado, el subrayado de una pestaña avanza hasta la siguiente, un campo rechazado niega con la cabeza, una copia se marca de un solo trazo, los avisos se apilan como una mano de cartas y se abren en abanico al señalarlos, la hoja de un móvil cede como una goma al tirar de ella, las filas de una tabla ordenada se mueven a su sitio, una cifra que cambia rueda como un cuentakilómetros y los gráficos se dibujan solos la primera vez que se ven. Todo sigue la opción Movimiento de Ajustes → Apariencia y se queda quieto con movimiento reducido. FinMotion es un proyecto propio (`github.com/finstats/finmotion`), colocado encima de FinUI.
- **Más iconos en FinUI**: ahora 239, frente a 71, cada uno con su propio movimiento (subir y bajar el volumen, unos auriculares y unos silenciados, un lápiz que escribe, una campana que suena, un teléfono que vibra), y todos se pueden descargar desde la galería de FinUI como SVG, estáticos o animados.
- **Fuentes**: treinta y cuatro familias de licencia libre para elegir (Geist, Roboto, Manrope, IBM Plex Sans, Montserrat, Lora, Playfair Display, Fira Code y más), incluidas como Inter, nunca descargadas de otro sitio y cargadas solo cuando se eligen.
- **Más bases y acentos**: Nord, Solarized, Gruvbox, Catppuccin, Rosé Pine, Tokyo Night, Dracula, Everforest y Ocean; dieciséis acentos más, diez de ellos de un color de día y de otro de noche. Ahora cada base tiene su propio papel de día: los grises se parecían en el tema claro.
- **Eliminar se puede deshacer durante 30 días**: una reproducción o una copia de seguridad eliminada espera en la papelera, y **Deshacer** en el aviso que aparece la recupera. Actividad y Ajustes → Copias de seguridad enumeran lo eliminado recientemente, con la fecha en que desaparecerá para siempre, siempre que haya algo. Una reproducción en la papelera queda fuera de todos los totales, gráficos, resúmenes anuales y perfiles, y una importación o una restauración no la devuelve como nueva; quien la vio con alguien cuenta como si la hubiera visto a solas hasta que vuelva.

### Changed
- Dark Reader y extensiones parecidas dejan en paz las páginas de FinStats (`darkreader-lock`): FinStats dibuja ambos temas por sí mismo, y una página repintada se peleaba con ellos.
- En el móvil, el menú ya no se desliza desde la izquierda: la barra de pestañas está abajo, salvo que se elija otro.
- Tras esta actualización, FinStats lee tu biblioteca una vez para saber dónde terminan los archivos que contienen varios episodios.
- En la página de un título, **Añadir a Mi lista** es tan alto como **Abrir en Jellyfin**, a su lado; antes era un tamaño más pequeño.
- Los filtros de periodo y de personas se quedan en la parte superior de la pantalla mientras se desplaza la página, así que 7d pasa a Todo desde el final de una página larga; lo mismo ocurre con los filtros de solicitudes y de próximamente de En camino.
- El botón de volver arriba se aparta del menú que ocupe la parte inferior de la ventana: abajo en la esquina junto al dock, justo encima de la barra de pestañas, el botón del pulgar o la barra de direcciones de un móvil y, en los demás casos, encima de la barra de estado. Es el `to-top` de FinUI.
- Eliminar una reproducción o una copia de seguridad la manda a la papelera en lugar de borrarla en el acto (`PUT` con `{"deleted": true}`; `DELETE` ya no existe en la API). Las copias de seguridad que superan el número que se conserva se siguen borrando para siempre.
- **Novedades** es una sola hoja en lugar de un montón de tarjetas: cada nota es una línea propia con su tipo (Añadido, Cambiado, Corregido y los demás) y el icono de ese tipo en una columna delante, bajo un encabezado para cada serie. La lista de versiones sigue a la derecha.

### Performance
- Abrir FinStats descarga solo lo que ha cambiado: una hoja de estilos, un script o una fuente que el navegador ya tiene se confirma en unos pocos bytes en lugar de enviarse de nuevo. Cada visita descargaba enteros todas las hojas de estilos y módulos de FinStats, aunque cada uno llevaba una etiqueta para distinguirlos.

### Stability
- Restaurar una copia de seguridad lee los ajustes solo de la fila de ajustes: un archivo de un FinStats más reciente que guarda más cosas ahí perdía sus ajustes sin avisar.
- Una pestaña abierta ya no crece con cada página visitada: cada página que se dejaba se quedaba en memoria detrás de la que estaba en pantalla (unos 2600 elementos por visita), retenida por la lista de lo que mueve FinMotion y por el detector de desplazamiento de la barra de filtros, hasta cerrar la pestaña.
- Una orden mal escrita (`finstats bakcup`), o `restore` sin su archivo, ya no crea ni actualiza una base de datos antes de decir que no sabe qué hacer.

### Fixed
- El pequeño gráfico de un mosaico ya no se sale del mosaico cuando el número que tiene al lado es ancho («56d 23h») o cuando cuatro mosaicos comparten una fila más estrecha: el gráfico cede y el número conserva su ancho.
- En el tema claro, un botón principal se volvía violeta bajo el puntero; ahora conserva el color del sello.
- En Ajustes, un interruptor queda a la altura del nombre y la ayuda de su ajuste; antes colgaba de la parte superior de la fila.
- En una lista clasificada sin nada bajo un nombre, como Usuarios más activos del panel, el nombre quedaba arriba en su fila junto a una imagen centrada; ahora está centrado.
- Una fila de mosaicos de estadísticas con un ancho entre móvil y escritorio (cuatro mosaicos de tres en tres) dejaba el último solo en una fila; ahora cuatro mosaicos van de cuatro, de dos o de uno en uno, y tres van de tres o de uno en uno.
- La lectura del servidor contaba un dispositivo en el que nadie había iniciado sesión, que no se guarda: la tarea decía tres dispositivos donde la página Servidor enumeraba dos.
- Un segundo, un minuto, una hora: las alertas decían «1 segundos de diferencia» y «en 1 minutos», y una tarea de Jellyfin con un intervalo de un minuto se ejecutaba «cada 1 minutos».
- `finstats --help` nombra todas las variables que lee FinStats; faltaban `FINSTATS_PUBLIC_IP_URL`, `FINSTATS_ALLOW_LIBRARY_SHRINK` y `FINSTATS_SKIP_PREUPDATE_BACKUP`.
- El asistente de configuración decía de una dirección que no lo era que «no se pudo conectar», en una frase que se juntaba con la siguiente; ahora dice que no es una dirección.
- En Ajustes → Copias de seguridad, una pregunta hecha mientras se escribía una copia desaparecía al terminar la copia.
- Con el menú como riel que se abre, cerrar la búsqueda mostraba a la vez la búsqueda abierta y el riel cerrado, y luego dejaba el riel abierto hasta que el puntero entraba y salía. Ahora Esc devuelve el foco del teclado a donde estaba cuando se abrió la búsqueda, y deja ir el foco que haya dejado un clic.
- En el móvil, cerrar la búsqueda mantenía la página oscurecida hasta que la búsqueda desaparecía y luego volvía a la claridad de golpe; ahora el oscurecimiento se desvanece con ella.
- Los pasos de la importación de Tautulli decían que la copia de seguridad se descarga después de **Backup Database**. Tautulli la guarda en su propia máquina: ahora los pasos dicen dónde (su carpeta `backups`, `/config` con Docker), qué archivo tomar y que hay que copiarlo.
- Cerrar la búsqueda antes de que terminara de abrirse dejaba el riel que se abre sin su animación: a partir de ahí se abría y se cerraba de golpe.
- El riel que se abre seguía abierto después de elegir una página en él hasta hacer clic en otro sitio y, en algunos navegadores o con zoom, dibujaba una barra de desplazamiento bajo sus iconos. Ahora se abre solo para el puntero y el teclado, y no dibuja ninguna barra de desplazamiento mientras está cerrado.
- En la página Seguridad, una dirección que la base de datos sitúa en un país pero no en el mapa cuenta para ese país, en lugar de entre las direcciones sin lugar conocido.

## [2.1.2] - 2026-10-01

### Added
- **Importar desde Tautulli**: trae tu historial de Plex. Sube la copia de seguridad de la base de datos de Tautulli (el `.db`, o el `.zip` que lo contiene) en **Ajustes → Importar**; sus reproducciones tienen su propio lugar en el filtro *Registrado por* de Actividad.
- **Un panel de cables para decir quién es quién**: los usuarios de Plex a un lado y los de Jellyfin al otro. Arrastra un cable desde cada usuario de Plex hasta quien es ahora, o haz clic en uno y luego en el otro, o usa el teclado. Quien se quede sin cable no se importa, y dos cuentas de Plex pueden ir a una misma persona.
- Las películas y los episodios se emparejan con tu biblioteca por nombre (una película por su título y año, un episodio por su serie y su número), y uno que aún no está en tu servidor se guarda y se vincula cuando llega. Un visionado reanudado es una sola reproducción; la música no se importa.
- La copia de seguridad contiene los tokens de acceso de Plex: FinStats nunca los lee y elimina el archivo en cuanto importas o vuelves a empezar.
- **Contenido sin vincular** en **Ajustes → Datos**: cada título de Tautulli, Jellystat o Streamystats que no coincide con nada de tu biblioteca, con su procedencia y dónde es más probable que ya esté. **Localizar** lo vincula a partir de las mejores sugerencias o de una búsqueda (una película se puede localizar como especial de una serie), y FinStats recuerda la elección, así que el mismo historial importado de nuevo se vincula solo. Las elecciones forman parte de una copia de seguridad.

### Performance
- **El resumen anual se abre de tres a cinco veces más rápido**: los actores y directores más vistos del año se calculaban recorriendo de nuevo el año una vez por cada persona que aparecía en él. Medido con un historial real, un año pasó de 0,29 s a 0,09 s, y los últimos 12 meses, de 0,53 s a 0,12 s.

### Fixed
- **El historial importado encuentra títulos que se llaman de otra manera**: una reproducción también coincide con el nombre en el idioma original de un título, así que una serie que Plex o Tautulli conocían como 오징어 게임 se vincula a El juego del calamar, e *Im Westen nichts Neues*, a *Sin novedad en el frente*. Los títulos originales de tu biblioteca se leen en su próximo escaneo.
- Los nombres se comparan como los compara la búsqueda (mayúsculas, acentos, puntuación, un año escrito en el nombre, un «The» inicial), y el año de estreno puede diferir en uno, así que «Star Wars: Episode I - The Phantom Menace» encuentra «… – The Phantom Menace», y un Kingsman de 2014, el de 2015. Un título conocido por un nombre totalmente distinto sigue sin adivinarse nunca.
- Una importación o restauración de Streamystats fallida se anunciaba en las notificaciones con su nombre interno.
- El botón *Compartir el año en tarjetas* del resumen anual y la posición del espectador a su lado no tenían espacio entre ellos.

## [2.1.1] - 2026-10-01

### Changed
- **Mi lista se ofrece allí donde hay un título**: en las tarjetas de Próximamente del panel, igual que en En camino → Próximamente, y en la página de un episodio o de una temporada, que añade su serie a la lista.
- En un póster del panel, el botón de Mi lista aparece mientras señalas la tarjeta o llegas a ella con el teclado, en lugar de tapar la esquina de cada póster; en una pantalla táctil está siempre visible.

### Stability
- Guardar una conexión de Sonarr, Radarr o Seerr mientras alguien tenía abierto **Ajustes → Conexiones salientes** podía dejar a ambos esperándose mutuamente para siempre.
- **Unos ajustes que no se pueden leer detienen FinStats al arrancar**, nombrando el ajuste culpable, en lugar de devolver en silencio todos los ajustes a sus valores predeterminados.
- **Dos cambios en los ajustes a la vez** (la página de ajustes y la programación de una tarea, por ejemplo) ya no se anulan el uno al otro.
- Una pausa, un salto o un cambio de pista detectados mientras la base de datos estaba ocupada se escriben en la siguiente pasada en lugar de perderse.
- Una lectura de la biblioteca ya no falla cuando Mi lista de alguien tiene varias entradas que resultan ser un mismo título.

### Fixed
- **Un archivo sustituido no está «ya en el servidor»**: mejorar el archivo de una película que alguien tiene en Mi lista ya no le dice que acaba de llegar.
- Una sola lectura fallida de los complementos, las tareas programadas o el almacenamiento de Jellyfin ya no vacía esa lista en la página Servidor hasta la siguiente lectura.
- **Una reproducción de Streamystats se detiene donde se detuvo en todas las páginas**: la curva de abandono de un título y el «terminado» de un episodio se basaban en su duración y la contaban como estimada.

## [2.1.0] - 2026-09-29 to 2026-09-30

Mi lista

### Added
- **Una lista propia**: añade una película o una serie desde su página, desde Próximamente, desde Añadido recientemente o desde la búsqueda (Ctrl+Intro), y encuéntrala en **Mi lista**, en tu propio perfil.
- **Cada entrada dice en qué punto está ahora**: en el servidor, «5 de 26 episodios», solicitada, «Llega el viernes», no está en el servidor, ha salido de la biblioteca o ya vista. Se calcula cada vez que miras, con el mismo criterio de «visto» que el progreso de tus series. Los títulos vistos se quedan, en un grupo propio, hasta que los quitas.
- **Un título que aún no está en tu servidor** puede ir a la lista directamente desde lo que esperan Sonarr o Radarr. Se vincula solo al título cuando llega, y un archivo renombrado se lleva su entrada consigo.
- **«De Mi lista: ya disponible»**: una notificación para tus propios destinos cuando llega algo que estabas esperando. Desactivada hasta que la marques.
- **Nadie más ve tu lista**, tampoco un administrador, y su notificación va solo a tus destinos. No se escribe nada en Jellyfin, Seerr, Sonarr ni Radarr.
- Las listas forman parte de una copia de seguridad, y una restauración las combina sin duplicar nada.

### Changed
- Las entradas de `/api/upcoming` llevan los ID de TVDB y TMDB de su título, y `/api/auth/me` indica si el servidor guarda listas.

## [2.0.4] - 2026-09-29

### Added
- **Programación de tareas, como lo hace Jellyfin**: cada trabajo que FinStats hace por su cuenta tiene disparadores (a diario a una hora, semanalmente un día, a intervalos, al arrancar o después del escaneo de biblioteca de Jellyfin), cada uno con un límite de tiempo opcional. Haz clic en una tarea para ver y cambiar su programación; *Añadir disparador* abre el mismo cuadro de diálogo que tiene Jellyfin.
- **Tareas es una sección propia** en Ajustes, con la última ejecución de cada trabajo en palabras («Se ejecutó hace 38 minutos y tardó 6 minutos») y un botón Ejecutar; antes estaba en la parte superior de Sistema.
- **Cambios de metadatos**: cada hora, FinStats lee lo que Jellyfin ha guardado desde la última consulta (nombres, sinopsis, géneros, valoraciones, reparto y equipo, datos de archivo e imágenes), así que una edición en Jellyfin aparece sin esperar a un escaneo de biblioteca.
- La copia de seguridad y la base de datos de geolocalización también se pueden ejecutar desde Tareas; Esc vuelve de la programación de una tarea a la lista.

### Changed
- **El tema es un interruptor con tres posiciones**, Claro · Dispositivo · Oscuro, donde antes estaba el botón que pasaba por ellas: haz clic en un tercio, arrastra el botón deslizante o usa las teclas de flecha. El botón deslizante lleva el icono de la elección, que entra con un movimiento cuando cambia.
- **Cerrar sesión está en Ajustes → Cuenta**, que dice quién ha iniciado sesión aquí; la barra lateral conserva el interruptor del tema.
- **Los antiguos ajustes de programación ahora son disparadores**: *Seguir el escaneo de biblioteca de Jellyfin* con su intervalo de reserva, *Hacer copia de seguridad cada N días* y el interruptor *Mantener la base de datos actualizada* de GeoIP se han convertido en los disparadores correspondientes de sus tareas, con los valores que tenías. Recopilación, Copias de seguridad y Seguridad enlazan ahora a la programación.
- Un arranque ya no vuelve a ejecutar todos los trabajos: se guarda cuándo terminó cada uno por última vez.
- Una tarea de Jellyfin que aún no tiene estimación muestra «quedan unos …», con los puntos girando donde irá el tiempo, en lugar de «ETA…».

### Performance
- **El mapa de Seguridad ya no se atasca**: arrastrar y hacer zoom mueven el mapa ya dibujado y lo dibujan una sola vez cuando termina el gesto, y el pulso de lo que se reproduce ahora ya no hace que todo el mapa se repinte 120 veces por segundo mientras nadie lo toca. Un arrastre pasó de 34 a 60 fotogramas por segundo en un ordenador lento.

### Fixed
- **Un póster o un retrato sustituido en Jellyfin aparece en FinStats**: las imágenes se guardaban en caché una semana en el servidor y otra en el navegador, tuviera lo que tuviera Jellyfin desde entonces. Ahora se guardan en caché según la etiqueta de imagen de Jellyfin, así que una imagen nueva es un archivo nuevo, y el navegador comprueba que su copia está al día cada vez que muestra una. Sustituir un retrato solo vuelve a guardar a la persona, así que Cambios de metadatos lee también a las personas.
- **Servidor → Tareas muestra una estimación nada más abrir la página**: el tiempo restante solo se medía a partir de las lecturas que la propia página hacía mientras estaba abierta, así que cada visita empezaba con «ETA…» hasta haber visto moverse el porcentaje. Ahora FinStats cronometra por su cuenta una tarea de Jellyfin en marcha, cada 10 segundos mientras dura, y la página recibe la estimación. Una tarea de la que Jellyfin informa al 0 % todo el tiempo (la primera fase de un escaneo de biblioteca) mantiene sus puntos hasta que se mueve: no se adivina nada.
- **Las descripciones emergentes del mapa aparecen sobre el punto**: en Firefox, la del punto de lo que se reproduce ahora podía quedar muy lejos, y donde los puntos se solapan (alguien que reproduce en casa) la que se mostraba dependía de un píxel. Ahora una sola descripción nombra todo lo que hay bajo el puntero.
- **La barra de estado tiene una sola forma al pasar el puntero**: cada enlace (las reproducciones, Repo, la versión) se ilumina como la misma cajita, y los puntos entre los elementos quedan centrados en los huecos. Al pasar por la versión se iluminaban también el punto y el hueco anteriores.
- **Una ruta de archivo ocupa todo el ancho de su tarjeta**: en la tarjeta Archivo de un título, y en las carpetas de Jellyfin de la página Servidor, una ruta se apretaba en una celda estrecha y bajaba nueve líneas junto a espacio vacío.
- Un cuadro de diálogo que se desvanece ya no se traga un clic en la página que hay debajo.

## [2.0.3] - 2026-09-28

### Security
- La página Servidor daba el ID de cada dispositivo a cualquiera con *Ver el servidor*; los ID requieren además *Ver detalles de red*, como en todas partes.
- Dos interruptores de permisos accionados antes de que se guardara el primero enviaban dos veces la misma lista de partida, y el segundo deshacía el primero: un permiso retirado seguía concedido y una sección del perfil público desactivada seguía publicada, mientras ambos interruptores mostraban lo elegido.

### Stability
- **Solo una importación o restauración a la vez**: dos subidas enviadas juntas pasaban ambas la comprobación, y una restauración nunca miraba las importaciones; la segunda agotaba la espera de la base de datos y fallaba tras minutos de subida.
- **Una dirección con un escape `%` roto abre la página de no encontrado**, en lugar de dejar un indicador de carga para siempre.
- **Un número de página grande en una lista es una página vacía**: se desbordaba y volvía a ser la primera página.

### Fixed
- **Las tablas ordenan bien los números en un navegador en alemán o noruego**: «1,204» (mil doscientos cuatro) se leía como uno coma doscientos cuatro, y todo lo que pasaba del millón, como texto.
- **La vista de tabla de un gráfico ordena las fechas, los días de la semana y las posiciones por cuándo**, no por sus letras.
- **El historial de descargas indica de dónde vinieron las importaciones**: Sonarr y Radarr solo nombran el indexador y el protocolo al capturar, así que esos desgloses siempre estaban vacíos. Un paquete de temporada fallido es un solo fallo, no uno por episodio.
- **Eliminar una reproducción vista en compañía** vuelve a calcular los grupos de ese título, en lugar de dejar un «grupo» de uno.
- **El tiempo en compañía de una persona nunca supera el tiempo que se quedó**: la página En compañía le atribuía la velada entera.
- **Una velada en compañía que cruza la medianoche de Nochevieja** (o el límite de «últimos N días») pertenece entera al periodo en que empezó, en lugar de contar como dos veladas a solas.
- **Las reproducciones de episodios importadas llevan el nombre del episodio** cuando Jellystat solo guardaba el de la serie.
- **La geolocalización encuentra la base de datos de ciudades junto a las de países y ASN** que escribe `geoipupdate`, en lugar de desactivarse.
- **El punto más rebobinado sigue los filtros de la página**, como sus recuentos.
- **«Tu año está listo»** solo llega a las personas cuyo resumen anual tiene algo.
- **Volver a resolver una alerta silenciada mantiene silenciada su pareja de lugares**; solo reabrirla deshace el silencio.
- **Una clave de API caducada ya no cuenta para el límite de veinte.**
- **El límite de sesiones deja ir la sesión más antigua** incluso cuando muchos inicios de sesión llegan en el mismo segundo.
- **Una dirección de casa encontrada por la consulta se queda** cuando esa misma se quita de la lista escrita a mano.
- **Fotos en el registro de auditoría y en la lista de claves**, donde solo había iniciales.
- **La lista de notificaciones de un administrador dice de quién es un destino personal**, en lugar de «Tuyo» en todos; **Enviados recientemente** muestra todos los destinos de su último evento.
- **Esc en un título o una persona nunca sale de FinStats**, y solo cierra el cuadro de diálogo de encima.
- **Una página se dibuja aunque un guardado cancele la carga anticipada que estaba esperando.**
- **Reproduciendo ahora muestra a un espectador en pausa que se desplaza por el vídeo** en el punto donde está ahora, no cuando pulsa reproducir.
- **Quitar una conexión actualiza qué páginas existen**, como lo hace guardar una.
- **La fecha de compilación de la base de datos de geolocalización conserva su año** en inglés estadounidense.

## [2.0.2] - 2026-09-28

### Security
- Detrás de un proxy, el límite de inicios de sesión contaba la primera dirección de `X-Forwarded-For`, que es la que envía el cliente: una dirección inventada nueva en cada intento, y el límite nunca se aplicaba. Ahora es la última, la que escribió el proxy.
- *Incluir direcciones IP y lugares* enviaba direcciones a un destino personal cuyo propietario no tiene *Ver detalles de red*, que ninguna página le mostraría. Los inicios de sesión fallidos de otra persona ahora requieren también *Ver el servidor*, como en la página Seguridad.

### Stability
- **Una detención nunca se pierde**: cuando se rechazaba una lectura de la biblioteca en los primeros momentos tras el arranque, antes de que el servidor estuviera a la escucha, el rechazo se anotaba y FinStats seguía funcionando en lugar de detenerse.
- **Un Jellyfin que responde con una forma que FinStats no sabe leer se rechaza**, no se cree: cada usuario, o cada título de una biblioteca, que llegaba con claves desconocidas (`id` en vez de `Id`) pasaba la protección con un recuento completo y luego se marcaba como eliminado.
- **El final de una reproducción nunca se pierde**: cuando la base de datos estaba ocupada más allá de su espera (por una importación), una reproducción que terminaba en ese momento seguía marcada como en curso hasta el siguiente reinicio. Ahora se escribe en la siguiente pasada.
- **Los ajustes que una restauración no sabe leer se dejan como están**: una copia de seguridad cuyos ajustes ya no encajaban con esta versión restablecía todos los ajustes al restaurarla: ventana de fusión, permisos, direcciones de casa, programación.
- **Un carácter de control en un nombre o un título ya no rompe las tarjetas**: la tarjeta del perfil, cada tarjeta de la historia que mostraba ese título y su ZIP respondían con un error.

### Fixed
- **Las notificaciones se detienen para alguien desactivado o eliminado en Jellyfin**: sus destinos personales seguían recibiendo todo lo que permitían sus antiguos permisos.
- **Una notificación en cola se vuelve a comprobar antes de enviarse**: a un destino desactivado, o de un propietario que había perdido un permiso, se le seguía enviando lo que esperaba a reintentarse.
- **Un destino que siempre responde `Retry-After` se abandona** tras cinco intentos como cualquier otro, en lugar de reintentarse durante treinta días.
- **Los ID de dispositivo en la página de una persona** solo se muestran con *Ver detalles de red*, como en todas partes.
- **Un año publicado espera un día para todo**: sus series terminadas y abandonadas seguían cada reproducción, así que pulsar reproducir en una serie abandonada cambiaba la página pública en segundos.
- **Una serie solicitada cuenta como vista** cuando se ha visto uno de sus episodios: todas las series solicitadas figuraban como nunca vistas, en el capítulo Solicitudes y en su tarjeta.
- **El capítulo En compañía del resumen anual deja fuera la TV en directo**, como el resto del año: un canal que se quedaba puesto en compañía podía convertirse en el título principal, y el porcentaje podía pasar del 100 %.
- **La tarjeta Día a día de «los últimos 12 meses» termina en el día más reciente**: la cuadrícula se saltaba las tres o cuatro semanas más recientes.
- **Los episodios nuevos de una serie se anuncian una vez por velada**: los episodios a uno y otro lado de la medianoche UTC se anunciaban dos veces en un servidor situado al este o al oeste de ella.
- **Conectar Seerr no anuncia solicitudes antiguas como listas para ver**: una solicitud que ya estaba disponible la primera vez que se leía, sin ningún otro registro de cuándo, figuraba como recién llegada en ese momento.
- **Los títulos de una biblioteca eliminada en Jellyfin se van con ella**: seguían en la búsqueda, en el estante Añadido recientemente y en los totales.
- **Un remux sigue siendo una transmisión directa** cuando una lectura pierde un momento sus datos de transcodificación (un salto), en lugar de convertirse en una transcodificación el resto de la reproducción.
- **«Jellyfin no responde» significa lecturas fallidas seguidas**: tiempos de espera agotados sueltos, con días de diferencia, en un servidor inactivo en el socket acababan sumando lo suficiente para declararlo.
- **Una importación de Streamystats muestra su progreso en su propia tarjeta**, no sobre el último resultado de la importación de Jellystat.
- **Una dirección cuyo host empieza por «web»** (`http://web:8096`) se lee como host, no se recorta como si fuera la ruta del cliente web de Jellyfin.
- **Un error de la cola oculta por completo el usuario y la contraseña** de una dirección aunque la contraseña lleve una barra.

## [2.0.1] - 2026-09-28

### Changed
- **Los idiomas de un título no estorban**: la cabecera nombra los tres primeros idiomas de audio y de subtítulos, una línea cada uno, y una nueva tarjeta **Idiomas** los enumera todos: los completos juntos y los parciales con cuántos episodios los tienen. Las dos docenas de idiomas de subtítulos de una serie larga empujaban la ilustración una pantalla hacia abajo.

### Fixed
- **El póster de un título se queda junto a su nombre**: cuando el texto de al lado era más largo que el póster (una sinopsis larga, muchos estudios), el póster se hundía hasta el fondo de la cabecera, bajo una ilustración vacía.
- **Una versión que FinStats no sabe leer se llama así**: un `app_version` editado a mano hasta quedar ilegible hacía que FinStats se negara a arrancar como si fuera «el FinStats 2.0.1 más antiguo», que no lo era. Sigue negándose, y ahora dice qué valor no ha podido leer y cómo escribirlo.

## [2.0.0] - 2026-09-25 to 2026-09-28

Resumen anual 2026, perfiles públicos y ver en compañía

### Added
- **Con quién lo has visto**, en el resumen anual: veladas en compañía, el tiempo pasado en ella, el título que más ha reunido a la gente y hasta tres acompañantes, nombrados en la aplicación y enlazados a sus perfiles.
- **Series vistas hasta el final**, en el resumen anual: el último episodio de cada una visto ese año, con el mismo criterio de «visto» que las barras de progreso del perfil, y también las empezadas y dejadas para más adelante: menos de la mitad vista y nada reproducido en dos meses.
- **Lo que pediste**, en el resumen anual: las solicitudes de Seerr hechas en el año, cuántas llegaron y cuántas viste después. Solo donde existen solicitudes.
- **El año frente al anterior**: horas, reproducciones y días, y cuánto más o menos.
- **El año como historia**: cada capítulo como una tarjeta de 1080×1920, el formato de las historias del móvil (el año, las cifras, las series, películas y música más vistas, los géneros, la personalidad, las horas y los días, cada día del año, los récords, la compañía, las series terminadas, las solicitudes y el año anterior). Guarda una, o todas en un ZIP, desde «Compartir el año en tarjetas».
- **Una tarjeta nunca nombra a nadie más**: las tarjetas se dibujan a partir de una copia del año sin ningún campo para otra persona, una posición o una aplicación. En la aplicación, una tarjeta no lleva ningún nombre.
- **El año del servidor**, para los administradores de Jellyfin: todos los títulos y totales y la personalidad de la casa, sin clasificar ni nombrar a nadie. Un interruptor en la página del resumen anual, `scope=server` en la API.
- **«Tu resumen del año está listo»**: una notificación en diciembre, una vez a cada persona que vio algo ese año.
- **Perfiles públicos**: una persona puede publicar parte de su perfil en un enlace que cualquiera puede abrir sin cuenta (totales y títulos más vistos, rachas y cuándo ve, el año entero como su historia de tarjetas, las diez últimas reproducciones), cada parte desactivada hasta que su propietario la active en Ajustes → Perfil público. Los dispositivos, las direcciones, las aplicaciones, los métodos de reproducción, las rutas de archivo y las demás personas nunca se publican, se active lo que se active, y el nombre que se muestra es uno que escribe el propietario, nunca su usuario de Jellyfin.
- **Nada en una página publicada tiene menos de un día**, ni las reproducciones recientes ni ningún total, así que una página que alguien vigila nunca dice quién está viendo algo en este momento.
- **Tarjetas para compartir**: cada perfil publicado viene con una imagen de 1200×630, del perfil y del año, que Discord, Signal, Mastodon y los demás muestran al pegar el enlace y que se puede descargar para publicarla. Se dibuja en el servidor a partir de lo que muestra la página y nada más.
- **Un enlace que puedes retirar**: 22 caracteres aleatorios que se restablecen con un clic; el enlace antiguo responde entonces exactamente igual que uno que nunca existió.
- **Los perfiles públicos están desactivados hasta que un administrador los permite**: un interruptor para todo el servidor, una lista de quién publica qué y una forma de retirar un perfil. Las publicaciones, los cambios, los restablecimientos y las retiradas quedan en el registro de auditoría, y una clave no puede hacer público nada.
- **En compañía**, una página propia: quién ve con quién, cuánto y cómo ha cambiado. El tiempo en compañía, las veladas y la parte del tiempo de visualización pasada en compañía, cada uno frente al periodo anterior; horas en compañía frente a horas a solas por día o por semana; cada pareja de personas con sus veladas, su tiempo en compañía, cuándo vieron algo juntas por última vez y lo que más ven; los títulos más vistos en compañía; las veladas recientes y, para quien puede ver a todo el mundo, el tiempo a solas de cada persona frente a su tiempo en compañía.
- Cada pareja cuenta: una velada de tres cuenta para cada una de sus tres parejas, cuando antes la tarjeta solo contaba el grupo exacto de personas. El tiempo en compañía de una pareja es el menor de los dos tiempos que estuvo cada uno.
- La tarjeta **Visto en compañía** del panel y de un perfil enlaza a la página, y la de un perfil dice qué parte del tiempo de visualización de esa persona fue en compañía.
- `GET /api/stats/groups` incorpora `pairs`, `series`, `people`, `previous`, `bucket`, y `watch_s` y `share` en sus totales; las claves que ya tenía no cambian.
- **Dónde lo dejan**, en la página de cada película y episodio: la parte de las reproducciones que sigue viendo en cada minuto, desde todo el mundo al principio hasta quien llegó al final, con los rebobinados y las activaciones de subtítulos de esas reproducciones marcados en el mismo eje. La tarjeta dice cuántas paradas se midieron (reproducciones que registró FinStats) y cuántas se estimaron (reproducciones importadas, según cuánto duraron), y necesita tres reproducciones antes de dibujar nada.
- **Quién sigue viendo**, en la página de una serie: cuántas personas empezaron cada episodio, en orden, y una columna **Terminado** por episodio. «Todo el mundo lo deja en el tercer episodio» son las personas que nunca pulsan reproducir en el cuarto.
- **Archivos que nunca se reproducen**, en Reproducción: una película o un episodio empezado tres veces o más que nunca pasó de los treinta segundos, con quién lo intentó y en qué aplicaciones. El registro de Jellyfin no guarda errores de reproducción, así que las reproducciones son el testigo; aquí no se aplica la duración mínima de reproducción, porque es justo lo que esas reproducciones nunca alcanzan.
- **Lo más rebobinado** y **Subtítulos activados**, en Reproducción: títulos clasificados por saltos hacia atrás por reproducción, con el minuto en que se concentran, y por la parte de las reproducciones en las que se activaron los subtítulos en los diez primeros minutos. Las tres listas requieren *Ver la actividad de todos*. Los saltos de menos de 20 segundos nunca se registraron, así que repetir brevemente una frase es invisible a propósito.
- Un salto guarda de dónde venía como un número junto a su etiqueta (`from_s` en la cronología de una reproducción). Cada salto ya registrado obtiene su número a partir de la etiqueta, y lo mismo una copia de seguridad restaurada de antes de esto.
- `GET /api/stats/files` para las tres listas; `GET /api/items/{id}` incorpora `insights`, y los episodios de una serie incorporan `users` y `finished`.
- **Claves de API**: cualquiera que haya iniciado sesión puede crear claves para sí en Ajustes → Claves de API, para un script, un panel o un móvil. Una clave se envía como `Authorization: Bearer fs_…`, se muestra una sola vez, se guarda como hash, puede llevar caducidad y se revoca con un clic; un administrador ve las de todo el mundo y puede revocarlas. Corresponde a la misma persona con los mismos permisos que un inicio de sesión, leídos en directo en cada petición, así que perder el derecho a iniciar sesión detiene todas las claves a la vez. Una clave no puede crear ni revocar claves.
- **Un feed de calendario**: `GET /api/calendar.ics?key=…` es lo que Sonarr y Radarr tienen previsto, como un iCalendar al que se puede suscribir un móvil. Una clave con el ámbito *calendario* abre el feed y nada más; el feed lee la clave de la dirección porque un calendario suscrito no puede enviar cabeceras, nunca de la cookie, y no nombra a nadie.
- **Un registro de auditoría** en Servidor → Auditoría, para los administradores de Jellyfin: cada inicio de sesión e intento fallido con su dirección, cada ajuste o permiso cambiado y a qué, cada clave creada, usada o revocada, cada conexión, destino, copia de seguridad e importación, cada reproducción eliminada y alerta resuelta, con quién, desde dónde, con qué clave y si funcionó. Se guarda un año y forma parte de las copias de seguridad. `GET /api/audit` lo recorre por páginas.
- **Importar desde Streamystats**, una sección propia en Ajustes junto a Jellystat. Toma su archivo de **Settings → Backup & Import → Download Backup** y suéltalo: años de historial llegan en un par de segundos, y nada de lo que ya tienes se vuelve a contar. `finstats import-streamystats <file>` lo hace sin interfaz, `POST /api/import/streamystats` a través de la API; solo se ejecuta una importación a la vez, sea del tipo que sea.
- **¿Usabas los dos rastreadores? Importa los dos archivos.** Una reproducción se reconoce como una que FinStats ya tiene por el ID que le da el propio rastreador y, si no, por la misma persona viendo el mismo elemento con cualquiera de los dos extremos de la reproducción cerca de uno del historial, así que se pueden importar una exportación de Jellystat y una de Streamystats de las mismas veladas, en cualquier orden, y una velada que FinStats registró por sí mismo tampoco se sobrescribe con ninguna de las dos. «Cerca» es la ventana de fusión de Ajustes → Recopilación.
- Las filas importadas reconocen lo que no pueden saber. Una reproducción que Streamystats registró por sí mismo no guarda nada del archivo (ni códec, ni resolución, ni tasa de bits, ni contenedor), así que esos campos se quedan vacíos en lugar de rellenarse con una suposición, y las tarjetas que los mostrarían se ocultan. Una reproducción que había tomado de Jellystat lleva la sesión entera, y se lee esa en su lugar: fechada por el momento en que terminó, el único que guardaba cualquiera de los dos rastreadores, y descrita como transcodificación solo cuando la sesión realmente lo fue.
- Las filas que Streamystats escribió porque Jellyfin marcó un elemento como visto, por un visionado que el propio Streamystats nunca vio, se cuentan y se omiten: duran tanto como la película entera y serían tiempo de visualización en una velada en la que nadie vio nada. El resultado dice cuántas había.
- Cómo se interpretan los datos de Streamystats, campo por campo, aprendido de una exportación real y no de la documentación: `docs/streamystats-import.md`.
- **De qué rastreador vino cada reproducción**, como filtro en Actividad: lo que registró el propio FinStats, lo que vino de Jellystat, lo que vino de Streamystats. Solo se ofrecen las que tu historial contiene de verdad, y se limita como cualquier otra lista, así que a alguien que solo puede ver sus propias reproducciones solo se le dice de dónde vino su propio historial.
- **Los filtros de Actividad son listas desplegables con casillas**, así que puedes pedir películas *y* episodios sin música, dos métodos de reproducción o dos personas a la vez, en lugar de uno de cada cosa o todo. Tipo de contenido incorpora **Otros**, ahora que se puede combinar.
- Varias personas a la vez, en cualquier sitio donde aparezca el filtro de usuarios (el panel, Reproducción, las bibliotecas, Seguridad y los demás), y el mapa y todas las estadísticas lo siguen. Un filtro de usuarios sigue sin poder hacer otra cosa que reducir lo que alguien puede ver: sin *Ver la actividad de todos*, una petición queda fijada a quien la hace, nombre a quien nombre la dirección y a cuántos.
- **Un tema claro**: papel washi, con texto de tinta y un acento bermellón. FinStats sigue el ajuste claro u oscuro del dispositivo hasta que eliges uno.
- **Un botón de tema junto a Cerrar sesión** pasa por Dispositivo, Claro y Oscuro. La elección se guarda en este navegador y se aplica antes de dibujar la página, así que nunca aparece un destello del otro tema.
- **Abrir en Jellyfin**: cada página de película, serie y episodio tiene un botón que abre la página de ese título en Jellyfin, en una pestaña nueva.
- **Dirección de Jellyfin para las personas** en Ajustes → Jellyfin: adónde apuntan esos botones, para cuando la dirección a la que se conecta FinStats no es una a la que pueda llegar un navegador. Si se deja vacía, se usa la dirección a la que se conecta FinStats; solo un administrador de Jellyfin puede configurarla.

### Changed
- **FinStats tiene una nueva casa**: `github.com/finstats/finstats`, y la imagen es `ghcr.io/finstats/finstats`. Cambia el nombre de la imagen en tu `docker run` o en tu archivo de compose para seguir recibiendo actualizaciones; la carpeta de datos se queda como está.
- **Ajustes muestra una sección a la vez.** La página eran catorce tarjetas en una columna y seiscientas palabras de ayuda. Ahora cada sección tiene su propia dirección (`/settings/collection`), una lista a la izquierda pasa de una a otra y, en el móvil, la lista se convierte en fichas. Los enlaces a los antiguos anclajes siguen llegando a su sitio.
- **Un ajuste es una fila**: qué es, una línea que dice la regla y el control a la derecha. Los números de una sección se guardan juntos con un botón al final; los interruptores se siguen guardando solos. Las tarjetas de estado (Tareas, Conexiones salientes, Base de datos) y el enlace a las licencias están en **Sistema**, y los dos importadores tienen una sección propia.
- **Busca un ajuste** desde el cuadro de la cabecera (`/` lo enfoca) por cualquier palabra de su nombre; indica en qué sección está el ajuste, e Intro te lleva a la fila.
- **Ajustes es para todo el que haya iniciado sesión**: las secciones que gestionan el servidor siguen siendo de quien puede gestionarlo, y una persona a la que solo se le permitían notificaciones por fin puede llegar a sus propios destinos.
- **La página Servidor muestra una sección a la vez**, como Ajustes: Resumen (sistema y almacenamiento), Tareas, Dispositivos, Complementos, Registro y Auditoría, cada una en su propia dirección. Los enlaces a `/server#jobs` siguen llegando a su sitio.
- **El registro del servidor es una sección de la página Servidor** (`/server/log`) en lugar de una página propia en el menú; `/events` redirige allí con sus filtros.
- Una notificación sobre una tarea fallida, una copia de seguridad fallida o una conexión que ha dejado de responder enlaza a la sección de ajustes que la contiene, en lugar de a la parte superior de la página.
- Lo que tu biblioteca puede decir de una reproducción importada (qué tipo de cosa era, qué episodio, su duración, a qué biblioteca pertenece) se rellena después de **cada** lectura de la biblioteca, no solo en el momento de una importación. El historial suele importarse antes de que FinStats haya leído nunca la biblioteca, así que esas reproducciones se quedaban para siempre supuestas o desconocidas.
- Restaurar una copia de seguridad de FinStats ya no añade una reproducción que llegó de un rastreador con otro ID, y sigue conservando dos visionados auténticos de lo mismo con minutos de diferencia.
- Los detalles de una reproducción nombran el rastreador del que vino, en lugar de llamar a todo lo importado reproducción de Jellystat.

### Performance
- **Un reinicio es casi instantáneo con cualquier historial.** Cada arranque calculaba quién había visto en compañía sobre todo el historial y volvía a decidir qué reproducciones eran locales, ambas cosas para no cambiar nada: 24 s y 5,5 s con diez millones de reproducciones. Ahora un arranque solo hace lo que dejó pendiente la última ejecución, y todo de nuevo solo después de una actualización o cuando ha cambiado el ajuste en que se basa.
- **Una multitud que llega a la vez se registra en segundos.** Cada reproducción nueva, cada guardado de progreso y cada dispositivo se escribía por separado, y cada reproducción terminada se cerraba mirando todo el historial de su título; ahora una pasada escribe cada tipo de una vez. 100.000 reproducciones que empiezan juntas tardan 3 s, cuando 25.000 que terminaban juntas tardaban más de cuatro minutos.
- **Reproduciendo ahora sigue siendo rápido con miles de personas viendo.** Para saber quién ve en compañía se comparaba cada reproducción con todas las demás y se enumeraba a cada acompañante por su nombre: cuarenta segundos con 10.000 reproducciones, y gigabytes de memoria aparte. Ahora las reproducciones se comparan dentro de un mismo título, y cada una muestra unos pocos nombres y cuántos más hay.
- Calcular quién había visto en compañía reescribía todas las reproducciones agrupadas del historial cada vez que FinStats arrancaba, importaba o veía terminar una reproducción; ahora solo escribe lo que ha cambiado. Averiguar qué reproducciones habían terminado ya no se ralentiza con el cuadrado del número de reproducciones en curso.
- **Reconocer una reproducción que el historial ya tiene pasa por un índice**, así que una importación de 150.000 reproducciones tarda segundos, y volver a vincular tras una lectura de la biblioteca recorre los títulos en lugar de leer cada reproducción mientras bloquea la base de datos.
- **Las estadísticas de Reproducción y el resumen son más rápidos sobre todo el historial**: sus partes independientes se calculan en paralelo, el gráfico de simultaneidad busca el día local una vez por cuarto de hora en lugar de una vez por reproducción, y los géneros se cuentan por título. Con un millón de reproducciones: de 4,1 s a 2,4 s y de 1,6 s a 1,1 s.

### Stability
- **Una biblioteca de televisión ya no parece una biblioteca vaciada.** Leer una biblioteca hace dos pasadas (todos los elementos y luego otra más pequeña solo para el reparto y el equipo de películas y series), y la comprobación que decide si se puede confiar en una lectura para eliminar lo que no ha visto miraba la segunda. En una biblioteca de televisión o de música los dos recuentos no se parecen en nada, así que la comprobación rechazaba cada lectura y detenía FinStats, y la instalación se reiniciaba y volvía a hacer lo mismo. Nunca se eliminó nada por error, pero ninguna lectura de la biblioteca podía terminar. Si tu biblioteca de series o de música no estaba recogiendo títulos nuevos, este es el motivo.
- Una importación podía fallar con «database is locked» cuando otra escritura (una reproducción que se actualizaba) caía entre su primera lectura y su primera escritura. Ambos importadores toman el bloqueo de escritura al empezar, y el diario de la base de datos ya no se queda tan grande como la mayor importación que se haya hecho nunca.
- **Un archivo de base de datos dañado, cortado, de otro programa o de solo lectura se rechaza al instante**, nombrando el archivo y diciendo que no se ha cambiado nada, en lugar de 30 segundos de silencio; nunca se escribe en la base de datos de otro programa.
- Una copia de seguridad o una instantánea previa a la actualización interrumpida a medias ya no deja un archivo atrás, uno que podía ocupar el lugar de una instantánea buena entre las tres que se conservan.
- **Sobrescribir la base de datos de lugares mientras FinStats está en marcha ya no lo bloquea**: FinStats lee de una copia privada del archivo.
- Una aplicación de Jellyfin que informaba de un ID de dispositivo nuevo cada vez podía hacer crecer la memoria de FinStats mientras estuviera en marcha; ahora los dispositivos que no se ven en cinco minutos se olvidan.
- Una persona conserva como máximo 30 sesiones iniciadas; la más antigua se cierra en el siguiente inicio de sesión.

### Fixed
- **La misma velada ya no se cuenta dos veces cuando se ha renombrado un elemento.** Volver a vincular el historial a un título que se había añadido de nuevo en Jellyfin podía convertir una reproducción importada de un rastreador en un duplicado de otra que ya estaba, porque la comprobación que la habría detectado se hacía antes de que cambiara el ID. Ahora se vuelve a hacer allí donde se revinculan ID. Solo se eliminan reproducciones importadas, nunca una que registrara el propio FinStats, y cuántas se fueron queda en el registro. El primer arranque de 2.0 repasa una vez todo el historial en busca de las que ya había.
- **Una sesión sigue ahora a la persona tal como la tiene Jellyfin en este momento.** Un administrador al que Jellyfin había degradado conservaba el acceso completo a FinStats el resto de su sesión de 30 días, y alguien desactivado o eliminado en Jellyfin seguía con la sesión iniciada. Ahora lo pierden en cuanto FinStats lee los usuarios de Jellyfin (cada 15 minutos, o en el acto desde Ajustes), como siempre ha pasado con las claves de API.
- Iniciar sesión escribe en la lista de usuarios de FinStats lo que Jellyfin dice de la persona, así que una copia de seguridad importada de un rastreador ya no puede convertir a alguien en administrador antes de que la primera lectura de los usuarios lo corrija.
- Alguien eliminado en Jellyfin se marca como eliminado aunque dos lecturas de los usuarios caigan en el mismo segundo.
- **Los errores de descarga nunca muestran la contraseña de un cliente de descargas ni la clave de API de un indexador**, que Sonarr y Radarr pueden citar en ellos.
- Las reproducciones revinculadas a un título renombrado durante una lectura de la biblioteca no se comprobaban para ver si se habían visto en compañía hasta el siguiente reinicio; ahora se comprueban en el acto.

## [1.6.5] - 2026-09-24

### Added
- **Se hace automáticamente una copia de seguridad completa antes de cada actualización.** La primera vez que un FinStats más reciente abre tu base de datos, escribe una copia completa de ella (todo tu historial, ajustes incluidos) en `data/pre-update-backups/` *antes* de cambiar nada. Si alguna vez una actualización rompe algo, tus datos están a salvo: detén FinStats, vuelve a poner la copia en lugar de `finstats.db`, arranca la versión que tenías y comunica el error. No se pierde nada. Se conservan las pocas copias más recientes; define `FINSTATS_SKIP_PREUPDATE_BACKUP=1` para desactivarlo si vas justo de espacio en disco.

## [1.6.4] - 2026-09-24

### Added
- **Cinco sitios más a los que enviar notificaciones**: Telegram, Slack, Pushover, Pushbullet y correo electrónico, junto a los destinos webhook, Discord, ntfy y Gotify que ya había. Cada uno pide solo lo que necesita (un ID de chat, una clave de usuario, un buzón), y a los tres que viven en un único servicio (Telegram, Pushover, Pushbullet) solo se puede llegar allí, así que un token no puede enviarse a un host que se haga pasar por ellos. El correo sale por TLS desde el primer byte o con `STARTTLS`, y no hay tercera opción.
- **Licencias**, en **Ajustes**: la GPL del propio FinStats y todas las licencias de terceros en las que se basa (253 crates de Rust, las dos fuentes incluidas, los contornos del mapa y la base de datos de geolocalización), cada texto completo, tal como lo escribió su propio proyecto. Elige un componente y su licencia se abre en la ventana junto a la lista; un crate que trae varios archivos pasa de uno a otro en el mismo sitio. La lista se genera a partir de los crates con los que realmente se compila FinStats, así que no se puede añadir una dependencia sin que quede registrada su licencia.

## [1.6.3] - 2026-09-24

### Fixed
- La protección ahora también actúa cuando Jellyfin no enumera *ninguna* biblioteca o *ningún* usuario (una lectura rota, nunca un día normal), algo que el ajuste de 1.6.2 había dejado pasar. Perder algunas bibliotecas sigue considerándose normal; perderlas todas a la vez, no.

## [1.6.2] - 2026-09-24

### Fixed
- La protección de integridad de datos añadida en 1.6.1 era demasiado celosa: una biblioteca que de verdad perdía la mayoría de sus elementos (un puñado de clips cuyos archivos desaparecieron, por ejemplo) se tomaba por una lectura rota, así que FinStats se negaba a actualizarla y se detenía. Ahora solo actúa ante una lectura *claramente* rota: una biblioteca lo bastante grande como para importar que se lee completamente vacía, o una grande vaciada casi del todo. Los cambios del día a día se aplican como antes, y el caso de «Jellyfin no ha devuelto nada» se sigue detectando.

## [1.6.1] - 2026-09-24

### Added
- **FinStats ya no se fía de una lectura de Jellyfin que borraría tu historial.** Si una biblioteca de la que FinStats guarda miles de elementos se lee de pronto vacía o casi (una actualización de Jellyfin que cambió su API, un error disfrazado de resultado vacío), FinStats conserva los datos que tiene, marca la sincronización como fallida (y lo notifica) y se detiene limpiamente con un mensaje claro en lugar de marcarlo todo como eliminado. Reinicia cuando Jellyfin vuelva a ser el de siempre, o define `FINSTATS_ALLOW_LIBRARY_SHRINK=1` si de verdad has vaciado una biblioteca. La misma protección cubre la lista de bibliotecas y la de usuarios.

### Security
- Una cabecera `X-Forwarded-Host` enviada por un cliente podía avalar un `Origin` ajeno y colar una escritura entre sitios saltándose la protección de mismo origen. Ahora esa cabecera solo se acepta detrás de un proxy de confianza (`FINSTATS_TRUST_PROXY`), la misma regla que ya usa la consulta de la dirección del cliente. La cookie de sesión `SameSite=Lax` era, y sigue siendo, la primera línea de defensa.
- Un destino de notificaciones personal podía apuntar a `0.0.0.0` o `::`, que llegan a esta misma máquina. La comprobación que mantiene los destinos personales apuntando a direcciones públicas ahora los cuenta como locales, junto al loopback y los rangos privados.

### Fixed
- La página **Reproducción** ya no deja de cargar en una biblioteca que contiene una reproducción sin método registrado (una fila restaurada de una copia de seguridad anterior a esa columna); esa reproducción cuenta simplemente como reproducción directa.
- Una palabra muy larga en una búsqueda o un filtro ya no provoca un error del servidor: SQLite rechaza un patrón `LIKE` de más de 50.000 caracteres, así que las palabras de los filtros se recortan a una longitud razonable antes de la consulta.
- La página **Ajustes** ya no se desplaza de lado en el móvil cuando una tarjeta devuelve una frase entera, como el resultado de una restauración.

## [1.6.0] - 2026-09-24

Notificaciones

### Added
- Tarjeta **Notificaciones** en Ajustes: añade un destino, marca de qué debe enterarse y pulsa Probar. Sin destinos, no se hace ninguna petición a nada.
- Cuatro tipos de destino: un **webhook** simple (JSON, documentado en `docs/api.md`, con cabecera `Authorization` opcional), **Discord** (un embed a través de la URL del webhook de un canal), **ntfy** (ntfy.sh o el tuyo propio, a un tema) y **Gotify** (tu propio servidor, con un token de aplicación). Los títulos viajan en el mensaje y no en una cabecera.
- Once tipos de evento, cada uno con su propia casilla: viaje imposible, una cuenta vista en un país nuevo, una ráfaga de inicios de sesión fallidos, una tarea que ha fallado, una copia de seguridad que no se ha podido escribir, algo que ha dejado de responder (Jellyfin, Sonarr, Radarr o Seerr) y cuando vuelve a responder, títulos nuevos en la biblioteca (agrupados por serie y día, como los agrupa el estante del panel), una solicitud que ya se puede ver y una reproducción que empieza o termina. Los dos últimos están desactivados de forma predeterminada.
- Una solicitud que ya se puede ver nombra el título, quién lo pidió y cuánto esperó.
- Destinos personales: concede **Recibir notificaciones** y alguien puede añadir uno propio, al que solo se envía lo que ya puede ver: sus propias solicitudes y alertas siempre, las de otras personas con *Ver la actividad de todos* y las direcciones con *Ver detalles de red*. Un destino personal debe apuntar a una dirección pública; solo un administrador puede dirigir uno hacia dentro de la red.
- **Incluir direcciones IP y lugares**, por destino y desactivado de forma predeterminada: si no, un mensaje sobre un viaje imposible solo nombra lugares («Oslo, Noruega y Londres, Reino Unido»).
- **Enviados recientemente**, en la misma tarjeta: qué se dijo, a qué destino y qué respondió el otro lado cuando no llegó. Un mensaje fallido se reintenta cuatro veces a lo largo de más o menos una hora, y a un destino que limita la frecuencia se le espera.
- **La dirección de FinStats**, un ajuste para administradores, es con lo que se construye el enlace de una notificación. Si está vacía, los mensajes no llevan enlace.
- Las tareas programadas de Jellyfin en la página Servidor: qué hace cada una, en palabras y no con el nombre de su código, y si es de las pesadas. Una tarea en marcha muestra una barra de progreso, un porcentaje y cuánto le queda; las demás, cuándo se ejecutarán, cuándo se ejecutaron por última vez y si salió bien. Se incluyen las tareas que Jellyfin oculta en su propio panel, y nada en FinStats puede iniciar ni detener una tarea.
- Lo que le queda a una tarea se mide a partir del ritmo al que ha avanzado realmente su porcentaje, frente a la lectura más reciente lo bastante antigua como para significar algo, y no con la media de toda la ejecución, así que una tarea que se acelera o se frena se describe por el ritmo que lleva ahora.
- Las listas de tareas que FinStats ya lee por otros motivos (la comprobación del escaneo de biblioteca cada 5 minutos, los datos del servidor cada 15) alimentan la misma medición, así que una tarea que lleva una hora en marcha suele tener una estimación en cuanto se abre la página. No se le pide nada más a Jellyfin para ello.
- Una tarea en marcha sin nada medido todavía muestra **ETA** y unos puntos suspensivos que giran en lugar de un número, y una cuyo porcentaje no se ha movido en medio minuto muestra «68 % durante 4m» al lado. Nada sustituye a una estimación que no se ha medido.
- Una lista de todas las versiones junto a las novedades: un clic lleva a cualquier versión, marca la que se está leyendo y, en una pantalla estrecha, se convierte en una fila de series encima de las notas.

### Changed
- Las novedades ya no se pliegan. Todas las series quedan abiertas y la lista de versiones es la forma de moverse, en lugar de un grupo abierto cada vez y un clic para llegar a cualquier otro.
- **Ajustes → Conexiones salientes** enumera cada destino de notificaciones junto a Jellyfin y las conexiones de servicios: qué es, su host, si está activado y cuándo aceptó un mensaje por última vez.
- No cambia nada de la recopilación, del historial ni de ninguna cifra en pantalla. FinStats sigue limitándose a *leer* de Jellyfin, Sonarr, Radarr y Seerr.

## [1.5.2] - 2026-09-22

### Changed
- Una lectura de la lista de sesiones de Jellyfin y el reinicio del reloj que la pidió son una sola llamada, así que ya no pueden separarse. Esa era la forma del fallo de peticiones desbocadas que se encontró mientras se preparaba 1.5.0.
- Demostrarlo lleva una fracción de segundo en lugar de siete minutos contra un servidor real, así que se comprueba cada vez que cambia cualquier cosa. Las mismas peticiones, el mismo historial, las mismas cifras en pantalla.

## [1.5.1] - 2026-09-22

### Fixed
- Una transcodificación se registra una vez, no una vez por segundo. Cuando una reproducción empieza transcodificando y vuelve a la reproducción directa, Jellyfin deja los datos de transcodificación en la sesión; cada lectura se comparaba con un valor que FinStats acababa de sobrescribir, así que cada lectura parecía un cambio y la página Actividad se llenaba de líneas de «Transcodificación» con «Reproducción directa: …» debajo.
- Las líneas ya escritas se limpian al arrancar: donde una serie de ellas dice lo mismo, se conserva la primera. Una reproducción que necesitó transcodificación sigue contando como transcodificación.

## [1.5.0] - 2026-09-22

Seguimiento de sesiones por WebSocket

### Changed
- FinStats mantiene una conexión abierta con Jellyfin desde el momento en que arranca, y así es como se entera de una reproducción. 1.4.0 lo trajo como un ajuste, desactivado de forma predeterminada, para que demostrara su valía con servidores reales; ahora está siempre activo. Las mismas reproducciones, pausas y saltos, calculados por el mismo código.
- Sin nada reproduciéndose, FinStats no le pregunta nada a Jellyfin. Una velada en la que nadie ve nada no cuesta ninguna petición.
- Con algo reproduciéndose, la lista de sesiones se lee una vez por segundo. De ahí sale la precisión de una pausa, un salto o un cambio de pista, y es lo que termina una reproducción cuyo cliente desapareció sin despedirse.
- Con todo lo cargado en pausa durante tres lecturas seguidas, se deja de preguntar y la conexión se encarga, cuando antes una película en pausa costaba la misma pregunta cada segundo, unos 16 MB por hora. Una reanudación, o que otra persona empiece algo, se atiende en más o menos un segundo, y el tiempo en pausa sigue sin contar nunca como tiempo de visualización.
- Cada lectura saliente pide respuestas comprimidas y las descomprime (Jellyfin, Sonarr, Radarr y Seerr comprimen todos cuando se les pide), lo que quita más de la mitad a las dos lecturas más grandes: la lista de sesiones durante una reproducción y la lectura de la biblioteca. La base de datos de geolocalización se sigue descargando tal como está guardada.
- La tarjeta de Jellyfin en Ajustes dice cuál de las dos mitades está en marcha, y el punto de directo indica si la conexión está transmitiendo y no en qué mitad estás.
- Solo una de las dos funciona a la vez: mientras FinStats pregunta, le dice a Jellyfin que deje de enviar, y le pide que vuelva a empezar en cuanto termina la última reproducción. La conexión permanece abierta todo el tiempo.
- A un Jellyfin que no responde nada a la suscripción se le cree mientras responda a los keep-alive: FinStats lo dice una vez, lee por su cuenta la lista de sesiones mientras tanto y sigue suscrito hasta que el primer envío lo aclara. «No puede enviar» y «se estaba reiniciando cuando preguntamos» se parecen, así que el silencio nunca es un veredicto.
- La alternativa no cambia: una conexión que se cierra, se queda callada o resulta no hablar este protocolo vuelve a preguntar con un temporizador en la siguiente pasada y sigue reconectando, con los intervalos de **Mientras alguien está viendo algo, comprobar cada** y **Mientras no se reproduce nada, comprobar cada**.

### Added
- Lo que está haciendo el recopilador, en `GET /api/status`: en cuál de sus cuatro estados está (`idle_socket`, `playing_poll`, `paused_socket`, `fallback`), si la conexión está abierta y suscrita, con qué frecuencia pregunta, cuántas lecturas de sesiones salieron en el último minuto y cuándo cambió eso por última vez. Se responde desde la memoria (sin petición ni consulta) y no lleva nombres, títulos ni recuentos.
- Esos campos se contrastan entre sí en cada pasada, y un desacuerdo que se prolonga se escribe en el registro.
- Un único limitador para todas las lecturas de sesiones del proceso: como máximo 2 por segundo y 70 por minuto; lo que pase de ahí se retrasa, se cuenta y se registra como mucho una vez por minuto. Un uso normal queda muy por debajo.

### Fixed
- Las actualizaciones en directo sustituyen de verdad al sondeo. Activado, 1.4.0 seguía pidiendo la lista de sesiones cada cinco segundos y reabría la conexión cada veinte a cuarenta, porque juzgaba la conexión por lo reciente de la última lista recibida, y Jellyfin envía una cuando algo *cambia* y nada entre medias. Ahora la conexión se da por viva si Jellyfin responde, sin más, y se comprueba con un keep-alive unas dos veces por minuto.
- Una lista de sesiones que llegaba mientras FinStats estaba entre dos pasadas se tiraba, suponiendo que otra llegaría segundo y medio después. Ahora se guarda y se usa.
- **Última comprobación** en Ajustes, y la tarjeta Conexiones salientes, ya no se quedan desfasadas mientras la conexión está callada: muestran la respuesta de Jellyfin a un keep-alive.

### Removed
- El ajuste **Dejar que Jellyfin envíe lo que se reproduce** de Ajustes → Recopilación, y `live_socket` de la API de ajustes.
- `socket_enabled` del estado del recopilador: ya no existe «desactivado».
- `last_reconcile_at` del estado del recopilador, sustituido por `socket_live`.

## [1.4.1] - 2026-09-22

### Changed
- Seerr solo se lee entero cuando algo ha cambiado: cada cinco minutos, FinStats pide una fila, la solicitud cambiada más recientemente, y la pasada termina ahí si ya la conoce. Eso es más o menos un kilobyte frente a unos sesenta de una página de cincuenta. Las solicitudes que siguen en camino se miran cada cuarto de hora, todo se enumera una vez al día y una solicitud nueva sigue apareciendo en menos de cinco minutos.
- Las colas de Sonarr y Radarr se leen cada cinco segundos mientras una página las muestra, cada minuto mientras hay algo en la cola y cada cinco minutos mientras está vacía, en lugar de cada minuto a todas horas. Abrir la página, conectar un servicio o una solicitud nueva en Seerr las actualiza en el acto.

## [1.4.0] - 2026-09-21

Sesiones opcionales por WebSocket, y todo lo que FinStats puede alcanzar

### Added
- **Dejar que Jellyfin envíe lo que se reproduce**, en Ajustes → Recopilación, desactivado hasta que lo actives: una conexión transporta la lista de sesiones, y unas 17.000 peticiones al día se quedan en casi ninguna. Las reproducciones, las pausas, los saltos y los grupos se calculan exactamente con el mismo código que antes. Mientras está activo, FinStats sigue haciendo una petición normal por minuto *mientras se reproduce algo*, y ninguna cuando no. Una conexión que se cierra, se queda callada quince segundos o resulta no hablarlo vuelve al temporizador en la siguiente pasada y sigue reconectando.
- **Conexiones salientes**, una tarjeta en Ajustes para los administradores de Jellyfin: cada destino al que puede llegar FinStats (tu Jellyfin, el servicio de «cuál es mi IP», la base de datos de DB-IP y cada Sonarr, Radarr o Seerr), con para qué sirve, si está activado y cuándo respondió por última vez. Se construye con lo que FinStats ya sabe y solo muestra direcciones: nunca una clave, nunca una ruta base.
- **Consultar ahora** en Ajustes → Red doméstica, para el día en que cambie tu dirección pública.

### Changed
- La consulta de «cuál es mi IP» se hace una vez, la primera vez que FinStats necesita una dirección, en lugar de cada quince minutos.
- La tarjeta de Jellyfin en Ajustes dice cómo se informa al recopilador y, cuando la conexión está activada pero no transmite, por qué.

## [1.3.0] - 2026-09-20

Sonarr, Radarr y Seerr, en una sola página

### Added
- **Conexiones** en Ajustes, para los administradores de Jellyfin: **Sonarr**, **Radarr** y **Seerr** (o Jellyseerr/Overseerr), varios del mismo tipo, cada uno probado antes de guardarse. Tu cliente de descargas no necesita configuración propia, porque Sonarr y Radarr ya hablan con él y FinStats lee lo que ellos saben.
- **Solicitudes**: quién pidió qué, cuánto tardó en llegar y si quien lo pidió llegó a verlo. Mosaicos con los títulos en espera y la espera habitual, un gráfico de cómo ha cambiado mes a mes, y listas de lo que llegó hace semanas y nunca se reprodujo, y de quién pide más.
- **Próximamente**: episodios nuevos y estrenos de películas de Sonarr y Radarr como una agenda por días, marcados según si *tú* ves esa serie y, para quien puede ver a todo el mundo, quién más la ve. Una serie cuenta como vista cuando alguien ha reproducido uno de sus episodios en los últimos cuatro meses. Una fila «Próximamente» en el panel, una por perfil para las series que ve esa persona, y lo siguiente de un título en su propia página.
- **Descargas**: la cola en directo de cada Sonarr y Radarr (qué es, cuánto lleva, a qué velocidad, qué falló al importar), con la persona que lo pidió al lado. Un paquete de temporada es una sola línea tenga los episodios que tenga, y lo que necesita atención va arriba. Debajo, lo que entró en una semana, un mes o un año por indexador, calidad y cliente de descargas, y qué descargas fallaron. El panel muestra la misma lista, abreviada.
- **Ver qué se está descargando**, un permiso nuevo. Sin él, las personas siguen viendo cuánto ha avanzado su *propia* solicitud y cuánto le queda, y nada más: ni nombres de archivo de las descargas, ni velocidades, ni otras descargas. Las solicitudes de otras personas requieren *Ver la actividad de todos*.

### Changed
- FinStats solo envía `GET` a estos servicios y nada más: no contiene ningún código que pueda aprobar una solicitud, iniciar una búsqueda ni añadir, pausar o quitar una descarga.
- Sus claves de API se guardan en la propia base de datos de FinStats, nunca se devuelven al navegador, nunca se escriben en un registro y nunca forman parte de una copia de seguridad. No se sigue ninguna redirección, así que una clave no puede viajar a un sitio que no hayas introducido tú, y los certificados se verifican salvo que lo desactives para una conexión.

## [1.2.2] - 2026-09-20

### Added
- Idiomas de audio y de subtítulos en cada película y episodio («Audio: japonés · inglés»).
- Hasta dónde llega un doblaje: una serie o una temporada dice cuántos de sus episodios tienen cada idioma («Inglés: 13 de 26 episodios»), y la lista de episodios tiene una columna **Audio**.
- **Idiomas de audio** e **Idiomas de subtítulos** en las páginas de las bibliotecas: cuántos archivos se pueden reproducir en cada uno.
- FinStats vuelve a leer la biblioteca una vez tras esta actualización para recoger los idiomas. Nunca dice «doblado», porque Jellyfin no da el idioma original de un título, y una pista sin etiqueta de idioma figura como «Desconocido».

## [1.2.1] - 2026-09-20

### Changed
- Las imágenes del README se vuelven a capturar en esta versión, con una nueva de la página **Seguridad**. No cambia nada en la aplicación.

## [1.2.0] - 2026-09-20

Desde dónde ve la gente, en un mapa

### Added
- **Página Seguridad**, para quienes pueden ver los detalles de red y la actividad de todos: un mapa del mundo con un punto por cada lugar desde el que ven tus usuarios, del tamaño de lo que ocurre allí. Casa aparece en verde, los lugares fuera de casa en morado, las reproducciones en directo laten y los inicios de sesión fallidos desde fuera aparecen en rojo. Arrastra para moverte, haz zoom con los botones, las teclas o Ctrl + rueda, y los puntos que se solapan se funden hasta que acercas. Debajo, cada lugar con sus personas, reproducciones, tiempo de visualización e inicios de sesión, y los países por reproducciones.
- **Viaje imposible**: una alerta cuando una cuenta aparece en dos lugares que ningún vuelo puede unir, o en dos lugares alejados a la vez, con ambas apariciones, la distancia, el tiempo entre ellas y la velocidad que habría hecho falta. **Mostrar en el mapa** dibuja el trayecto.
- **País nuevo**: la primera vez que alguien reproduce algo o inicia sesión desde un país en el que no se le había visto.
- Resuelve una alerta con una nota, o silencia una pareja de lugares para esa persona, por una VPN o un móvil cuyo operador está en la capital. Las alertas encontradas en historial antiguo, tras una importación o en una base de datos nueva, se archivan como resueltas en lugar de inundar la lista. Las alertas y lo que decidiste sobre ellas forman parte de las copias de seguridad.
- Los lugares salen de una base de datos de ciudades leída en local: ninguna dirección se envía nunca a ningún sitio, y el mapa se dibuja con contornos incluidos en FinStats en lugar de con un servicio de mapas. Descarga la base de datos gratuita de DB-IP desde la página Seguridad o desde **Ajustes → Seguridad**, deja que FinStats la actualice cada mes (desactivado hasta que lo actives) o suelta tu propio `.mmdb` (DB-IP, MaxMind GeoLite2-City) en `data/geoip/`. `FINSTATS_GEOIP_DB` indica un archivo en otro sitio.
- **Ajustes → Seguridad**: la base de datos en uso, la actualización mensual, y a qué velocidad (900 km/h) y a qué distancia (500 km) deben estar dos apariciones para contar como viaje imposible. Las bases de datos de ciudades suelen desviarse unos cientos de kilómetros, y para eso está la distancia.

## [1.1.3] - 2026-09-20

### Added
- Ctrl+Espacio también busca en el reparto y el equipo de todo lo que hay en tu biblioteca, en **Reparto y equipo**, con su foto, si actúan o dirigen y en cuántos de tus títulos aparecen; al elegir a alguien se abre su página. Es tan tolerante como la búsqueda de títulos (palabras en cualquier orden, sin importar los acentos, se perdona un dedo que resbala), y cuando varias coinciden igual de bien, va primero la que aparece en más de tus títulos.

## [1.1.2] - 2026-09-20

### Added
- **Añadido recientemente** en el panel, encima del gráfico de Actividad: las 30 novedades más recientes como una fila de pósteres que se puede desplazar de lado, con flechas en el ordenador. Cada una dice cuándo llegó y qué es (una película, un álbum o «Temporada 4 · 3 episodios»), y un único episodio nuevo dice cuál, con el póster propio de la temporada cuando lo tiene.
- **Mayús + rueda del ratón** y las teclas de flecha mueven un póster; Re Pág y Av Pág, una pantalla; Inicio y Fin van a uno y otro extremo, y se puede deslizar con el dedo. La rueda sola sigue desplazando la página.
- Los episodios nuevos de una misma serie que llegan el mismo día comparten una entrada, y una serie entera añadida de una vez es una sola entrada («3 temporadas · 60 episodios»), así que una gran importación no echa todo lo demás de la lista.
- La fila no tiene en cuenta el periodo ni la persona del panel, porque trata de la biblioteca y no de las reproducciones, y se oculta mientras la biblioteca está vacía.

## [1.1.1] - 2026-09-20

### Added
- Las páginas se abren al instante: un momento después de que FinStats cargue, descarga discretamente lo que probablemente abrirás después (el panel, tu perfil y tu cronología, Actividad, Usuarios, Bibliotecas y cada biblioteca, Reproducción, las páginas del servidor, el resumen anual y las personas más activas), así que al abrir una se muestra en el acto y se sigue actualizando en segundo plano.
- Las películas, las series y los miembros del reparto solo se descargan cuando hay intención, porque son miles: dejar el puntero sobre un enlace, tocarlo o llegar a él con el teclado descarga esa página.
- Espera a que termine de cargar la página que has abierto, trabaja página a página mientras el navegador está desocupado, se detiene en una pestaña en segundo plano, no hace nada con el ahorro de datos o una conexión muy lenta, y se olvida al cerrar sesión.
- Un pequeño enlace **Repo** con el logotipo de GitHub en la barra de estado, junto a la versión.

### Fixed
- La **Cronología** olvidaba qué bibliotecas estaban desactivadas al recargar la página o abrirla desde un marcador, aunque la elección estaba en la dirección.

## [1.1.0] - 2026-09-20

Una cronología de todo lo que has visto

### Added
- **Cronología**, junto a **Resumen** en cada perfil: lo que has visto como un único recorrido desde hoy hasta la primera reproducción que conoce FinStats, con una velada de episodios agrupada en una sola parada. Cada parada es un póster con lo que se vio y cuándo («Temporada 2 · Episodios 3-6», una película y si llevó más de una sesión, o un álbum y cuántas pistas), y abre el título.
- El recorrido se adapta al ancho de la ventana: tres paradas por fila en una pantalla ancha, dos en una más estrecha, con una fila de cada dos en sentido inverso y una curva que la une a la siguiente, hasta quedar en una sola línea recta en el móvil. La primera parada de cada mes lleva el mes.
- Las paradas más antiguas se cargan solas al desplazarte, hasta donde empieza el historial.
- Los interruptores de **Bibliotecas** encima del recorrido dejan fuera lo que no quieres ver, como la música, y la elección forma parte de la dirección.
- Lo que todavía se está reproduciendo dice **Reproduciendo ahora**.
- Ves tu propia cronología; las de otras personas requieren el permiso «ver a todos».

## [1.0.5] - 2026-09-20

### Added
- Protección contra versiones anteriores: la base de datos recuerda la versión más reciente de FinStats que la ha abierto, y una versión más antigua se niega a arrancar con ella en lugar de trabajar en silencio con datos que no entiende del todo. El mensaje dice qué hacer. Las versiones hasta la 1.0.4 se publicaron antes de que existiera esta comprobación.

## [1.0.4] - 2026-09-20

### Fixed
- En los detalles de una reproducción, el botón de copiar junto a un **ID del dispositivo** largo había caído a una línea propia.
- En la página Reproducción, la última columna de **Qué clientes transcodifican** quedaba cortada («Transc…»); ahora los encabezados de dos palabras pasan a la línea siguiente.

### Changed
- Las capturas de pantalla del README vuelven a estar al día; todavía mostraban la versión 0.5.0.

## [1.0.3] - 2026-09-20

### Added
- La página de un episodio muestra la imagen propia de ese episodio, la que muestra Jellyfin en su lista de episodios, en lugar del póster de la serie. Un episodio para el que Jellyfin no tiene imagen conserva el póster.

## [1.0.2] - 2026-09-20

### Fixed
- En **Novedades**, los grupos v0.9 y v0.10 no tenían titular junto a su versión. Ahora el titular de un grupo es siempre la primera frase de lo que trataba su primera versión, así que una introducción larga no se corta a mitad de frase.

## [1.0.1] - 2026-09-20

### Fixed
- La imagen publicada no arrancaba en una máquina recién instalada: «unable to open database file: /data/finstats.db», tras quedarse colgada medio minuto. Docker crea como root una carpeta `data` que falte, y FinStats se ejecuta como un usuario normal. Ahora el contenedor hace suya la carpeta y luego pasa a ese usuario, así que `docker run` funciona a la primera. Para archivos que pertenecen a alguien que no es el usuario 1000, define `PUID` y `PGID`; `--user` funciona como antes.
- Una carpeta de datos en la que de verdad no se puede escribir se notifica en el acto, con la orden que lo arregla, en lugar de 30 segundos después como un error de la base de datos.

## [1.0.0] - 2026-09-20

Listo para todo el mundo

### Added
- Una imagen lista para usar, ahora en `ghcr.io/finstats/finstats`, para máquinas Intel/AMD y ARM de 64 bits (sirve una Raspberry Pi 4 o 5): `:latest` es la versión más reciente, una etiqueta de versión se queda donde está y `:edge` es la versión en desarrollo.
- Cada versión aparece en la página de versiones del proyecto en GitHub con estas novedades.

### Changed
- Los números de versión ahora significan lo que dicen: las actualizaciones 1.x no romperán tus datos, tus copias de seguridad ni tus ajustes.
- El archivo de Docker Compose y todas las instrucciones usan la imagen publicada. Un `finstats:latest` compilado por tu cuenta solo necesita cambiar el nombre de la imagen; la carpeta `data` se mantiene intacta.

## [0.10.2] - 2026-09-20

### Fixed
- En el móvil, Ajustes y la página de una serie podían desplazarse de lado: una etiqueta pensada solo para lectores de pantalla se salía de su tabla, y las listas de episodios y la fila del reparto hacían lo mismo. Ahora se desplazan dentro de sí mismas.
- El resumen anual ya no se desplaza de lado cuando la palabra de fondo de un capítulo es más ancha que la página.
- Una línea larga de «viendo en…» bajo **Reproduciendo ahora** termina en puntos suspensivos en lugar de cortarse a mitad de letra.
- El póster delante de cada serie en un perfil era un segundo enlace sin nombre para los lectores de pantalla; ahora se les oculta y el enlace es el título que tiene al lado.

## [0.10.1] - 2026-09-20

### Fixed
- Descargar una copia de seguridad fallaba en el navegador («no se pudo leer el archivo de origen» en Firefox) sin nada en la página que explicara por qué: FinStats volvía a comprimir una copia de seguridad que ya está comprimida, y la transferencia doblemente empaquetada se cortaba antes del final. Ahora las copias se envían tal cual, con su tamaño por delante, para que el navegador pueda mostrar el progreso real.

## [0.10.0] - 2026-09-20

Copias de seguridad y mudanza a una instalación nueva

### Added
- Una copia de seguridad cada semana, conservando las cinco más recientes: cada reproducción con su cronología de pausas y saltos, las marcas de visto, los permisos, las direcciones de casa, el registro del servidor y tus ajustes, en un archivo pequeño, unos cientos de KB para miles de reproducciones. **Ajustes → Copias de seguridad** las enumera, cada una con un botón **Descargar**, hace una cuando se lo pides y define cada cuánto y cuántas. Están en la carpeta `backups` de tu directorio de datos. Una copia de seguridad no contiene ninguna clave de API de Jellyfin ni ninguna sesión iniciada, pero es un historial de visualización completo con direcciones IP, así que guárdala en un sitio privado.
- Restaurar combina en lugar de sobrescribir: las reproducciones que ya están se omiten, así que se puede hacer dos veces, o en una instancia que lleva un tiempo en marcha, sin peligro. Desmarca «Restaurar también los ajustes y los permisos» para recuperar solo el historial. Las copias funcionan entre versiones en ambos sentidos. Desde un terminal: `finstats backup` y `finstats restore <file>`.

### Changed
- La recopilación comprueba cada segundo mientras alguien está viendo algo y cada 5 segundos mientras nadie lo hace, cuando antes eran 5 segundos todo el tiempo, así que las pausas, los saltos y los cambios de pista se registran al segundo. Ambos están en **Ajustes → Recopilación**; si habías cambiado el intervalo antiguo, hay que volver a configurarlo.

### Fixed
- Una dirección IP en los detalles de una reproducción se partía en dos líneas por la mitad. La marca Local/Remoto pasa debajo cuando no hay sitio.

## [0.9.1] - 2026-09-20

### Fixed
- Las personas que estaban en casa aparecían como remotas. Un dispositivo de tu propia red que llega a Jellyfin a través de su nombre público, como un proxy inverso o un dominio, llega con la IP pública del hogar, y solo las direcciones privadas contaban como locales. Ahora FinStats averigua la dirección pública y cuenta esas reproducciones como locales en todo el historial, y recuerda las direcciones anteriores.

### Added
- **Ajustes → Red doméstica**: las direcciones que FinStats considera de casa; puedes añadir las tuyas (una dirección anterior, una segunda residencia, una salida de VPN) o desactivar la consulta. La consulta pregunta a un servicio sencillo de «cuál es mi IP» cada 15 minutos, probando varios (Amazon, Cloudflare y otros) porque los DNS que bloquean anuncios, como Pi-hole, suelen bloquearlos, y uno de ellos no necesita DNS. La petición no contiene nada sobre ti ni sobre tu servidor y, con el interruptor desactivado, FinStats no habla con nada más que Jellyfin. `FINSTATS_PUBLIC_IP_URL` define un servicio propio.

## [0.9.0] - 2026-09-20

Todas las tablas, en el orden que quieras

### Added
- Todas las tablas se ordenan: haz clic en un encabezado para ordenar por él, otra vez para invertir el orden y una tercera para volver al orden original. Se entiende el significado de cada celda, así que «3d 2h» se ordena como tiempo y «1,4 GB», como tamaño, y las celdas vacías van al final. Abarca la lista de usuarios, los espectadores, los episodios, los dispositivos, los complementos, las tareas programadas, la vista de tabla de cada gráfico, la matriz de permisos y las listas de barras (códecs, resoluciones, clientes y demás), que reciben un encabezado fino propio.
- Actividad y el Registro del servidor se ordenan sobre todas sus páginas y no solo sobre las filas en pantalla, y el orden se guarda en la dirección, así que sobrevive a una recarga o a un enlace compartido.
- Las tablas de diez filas o más llevan encima un pequeño campo de filtro.

## [0.8.2] - 2026-09-20

### Changed
- El selector de año del resumen anual es el mismo interruptor que todos los demás selectores de periodo de FinStats (7d · 30d · 90d …).

## [0.8.1] - 2026-09-19

### Added
- Páginas para actores y directores, desde «Las personas más vistas» del resumen anual o desde la nueva fila **Reparto y equipo** de cualquier película, serie o episodio: todo aquello en lo que aparecen en tu servidor, cuánto se ha visto y quién, y qué sigue esperando. Sigue el periodo que elijas, las personas que solo pueden ver sus propias estadísticas solo ven ahí lo que han visto ellas, y Esc te lleva de vuelta.

## [0.8.0] - 2026-09-19

El resumen anual estrena aspecto y tiene más que contar

### Added
- **Las personas más vistas**: los actores con los que más tiempo pasaste y los directores de lo que viste, como una fila de retratos. El reparto y el equipo se leen junto con la biblioteca y aparecen tras la primera lectura de la biblioteca posterior a esta actualización.
- **Tu año en días**: cada día del año como un cuadrado, más brillante cuanto más viste, con tu racha más larga y tu día más intenso encima. Pasa el puntero sobre un día, o recorre el calendario con las teclas de flecha, para ver qué hubo en él.
- **En cifras**: reproducciones, tiempo de visualización, títulos distintos, días con algo visto, la racha más larga y cuánto fue un **revisionado**, además de cómo se reparte el año entre episodios, películas y música.
- El capítulo de géneros dice cuántos géneros tocaste y qué parte se llevó el principal.

### Changed
- Un diseño nuevo: se abre con tu año «reproducido de nuevo», el titular junto a los pósteres que lo llenaron, sobre una forma de onda del año con una barra por semana y la más alta iluminada, y eliges el año justo encima. Cada capítulo tiene un titular con su palabra clave en violeta, una frase que lleva las cifras y la palabra del capítulo, grande y tenue, detrás. Termina como terminan las películas, con los créditos: protagonizada por, dirigida por, proyectada en, duración.
- Las horas, los días de la semana y los meses comparten un único capítulo **Patrones de actividad** con un interruptor entre ellos, y el titular sigue lo que estás mirando («El viernes se llevó la corona»).

### Fixed
- **Reproduciendo ahora** en el panel se quedaba en blanco cuando nadie estaba viendo nada, así que el encabezado parecía pertenecer a los filtros de debajo. Vuelve su recuadro «No se está reproduciendo nada ahora mismo».

## [0.7.10] - 2026-09-19

### Added
- Ilustraciones de las bibliotecas: cada tarjeta de biblioteca, y la página de la propia biblioteca, muestra la imagen que usa Jellyfin para ella, a través del proxy de imágenes de FinStats como los pósteres. Una biblioteca sin imagen conserva su icono.

## [0.7.9] - 2026-09-19

### Changed
- La tarjeta Géneros vuelve a abrirse como **Lista**, con el radial como segunda opción del interruptor. Se respeta una elección ya hecha.

## [0.7.8] - 2026-09-19

### Added
- Una vista radial para la tarjeta Géneros, en el panel y en los perfiles: un radio por género, que se alterna entre **Radial** y **Lista** en la esquina de la tarjeta, y la elección se recuerda. Pasar el puntero sobre un punto, o usar las teclas de flecha, muestra el tiempo de visualización y la proporción de ese género.

## [0.7.7] - 2026-09-19

### Fixed
- Las películas que pertenecen a una colección aparecían como «Ya no está en la biblioteca». Con *Agrupar películas en colecciones* activado, Jellyfin entrega la colección en lugar de las películas que contiene, así que FinStats nunca las veía; en un servidor, eso ocultaba 556 de los 1238 elementos de la biblioteca de películas. Ahora FinStats pide las propias películas, y la lectura que provoca esta actualización las recupera y vuelve a vincular sus reproducciones.

## [0.7.6] - 2026-09-19

### Changed
- En los perfiles, la tarjeta **Series** baja hasta justo encima de Géneros, así que una lista larga de series ya no saca de la vista el tiempo de visualización, el gráfico de actividad y los títulos más vistos. Las rachas de días se quedan arriba, y una serie que has desplegado sigue desplegada cuando cambias el periodo.

## [0.7.5] - 2026-09-19

### Changed
- La búsqueda coincide palabra por palabra, en cualquier orden y con lo que sea entre medias, en lugar de necesitar la frase exacta, así que «alya hides» encuentra *Alya Sometimes Hides Her Feelings in Russian*. Las mayúsculas, los acentos y la puntuación no importan, se ignora un «The» inicial y se perdona un dedo que resbala («mentalsit» encuentra *The Mentalist*); las palabras de tres letras o menos siguen teniendo que escribirse bien. Las mejores coincidencias van primero, y la música también se encuentra por su artista.
- Los cuadros de búsqueda de Actividad y del Registro del servidor buscan palabra por palabra en todas sus columnas: «alya opera» encuentra reproducciones de esa serie en Opera.

## [0.7.4] - 2026-09-19

### Changed
- Las novedades se agrupan por series: cada serie `0.x` es un bloque plegable con sus versiones, de qué trataba la serie, cuántas hay y cuándo. Solo la más reciente empieza abierta, y un bloque cerrado sigue mostrando cuál estás usando.

## [0.7.3] - 2026-09-19

### Fixed
- El reloj de Reproduciendo ahora titubeaba cada pocos segundos: la actualización cada 5 segundos también lo repintaba, así que, siempre que el servidor iba una fracción por delante, el número se adelantaba y luego se quedaba quieto un segundo. Ahora solo lo mueve el tic de cada segundo, exactamente un segundo cada vez (medido entre 999 y 1001 ms), y la actualización sigue corrigiéndolo tras una pausa, un salto o un desfase real, cayendo en el tic.

## [0.7.2] - 2026-09-19

### Added
- Ver en compañía, en directo: cada tarjeta de Reproduciendo ahora dice «Con maria» mientras varias personas ven lo mismo juntas, cuando antes un grupo solo se reconocía una vez terminadas las reproducciones. En directo, las personas cuentan como juntas cuando están en el mismo título y, o bien empezaron dentro de la ventana de grupo, o bien están casi en la misma posición, lo que sigue funcionando si FinStats se reinició a mitad de la reproducción.

## [0.7.1] - 2026-09-19

### Changed
- Reproduciendo ahora cuenta cada segundo: el reloj avanza y la barra se desliza entre medias, en lugar de saltar cada vez que se pregunta al servidor. Se sigue preguntando al servidor cada 5 segundos y sigue siendo la fuente de verdad: la pantalla se vuelve a sincronizar tras una pausa, un salto o cuando el servidor va por delante, pero no por una posición que solo lleva unos segundos de retraso, porque los clientes solo informan más o menos cada diez segundos. Las reproducciones en pausa se quedan quietas.

## [0.7.0] - 2026-09-19

Quién ve en compañía

### Added
- Ver en compañía: cuando varias personas empiezan el mismo título a la vez, FinStats lo cuenta como verlo juntas. Una tarjeta **Visto en compañía** en el panel muestra los grupos, los títulos que comparten y sus horas juntas; cada perfil muestra con quién ve más esa persona; las reproducciones compartidas llevan una pequeña marca de personas en Actividad, y los detalles de la reproducción dicen con quién se vio. Funciona sobre todo tu historial, reproducciones importadas incluidas.
- Un ajuste para lo cerca que deben estar los inicios, 60 segundos de forma predeterminada: aproximadamente un tercio de las sesiones de grupo auténticas empiezan con entre 6 y 60 segundos de diferencia. Además, las personas tienen que seguir viendo juntas un par de minutos, así que dos personas que abren el mismo episodio por casualidad no son un grupo.

### Changed
- La búsqueda se abre con **Ctrl + Espacio** en lugar de Ctrl/⌘ + K.

## [0.6.1] - 2026-09-19

### Added
- Los administradores pueden abrir el resumen anual de otra persona, en la parte superior de la página Resumen anual o con **Abrir el resumen anual** en su perfil. Entonces habla de esa persona por su nombre en lugar de «tú».

### Changed
- Todos los demás siguen viendo solo su propio resumen anual, tengan los permisos que tengan, y sigue sin haber resumen anual de todo el servidor.

## [0.6.0] - 2026-09-19

Tu perfil, y en qué punto vas de cada serie

### Added
- Progreso de las series en los perfiles: cada serie que has tocado como una barra con un segmento por episodio (visto, empezado o todavía no), que se puede desplegar temporada a temporada, ordenada según lo cerca que estás de terminarla, con vistas aparte para las series terminadas y para todo.
- **Marcar como visto** en tu propio perfil, por episodio, temporada o serie, para lo que viste mientras no se registraba nada. También se usan las marcas de visto del propio Jellyfin. Las marcas se quedan en FinStats y nunca cambian nada en Jellyfin; una reproducción registrada no se puede desmarcar, tus propias marcas sí.
- Rachas de días en los perfiles: tu racha más larga de días seguidos con alguna reproducción, tu racha actual y cuántos días has visto algo.
- **Mi perfil** en la barra lateral.
- Actividad muestra dónde se detuvo la reproducción (por ejemplo, `44:15 / 45:00`) bajo la barra de progreso, y la fecha y la hora bajo «hace 3 h». Las reproducciones importadas de Jellystat no tienen posición de parada, porque Jellystat nunca la registró.

### Changed
- Los episodios que faltan o que aún no se han emitido ya no se tratan como parte de tu biblioteca. Jellyfin los enumera como marcadores sin archivo, y antes contaban para los totales de episodios.

## [0.5.1] - 2026-09-19

### Added
- Un logotipo: un botón de reproducir cortado en tres barras de gráfico, en la barra lateral y en las pantallas de inicio de sesión, como un SVG nítido con un `favicon.ico` de varios tamaños para los navegadores y las barras de marcadores que lo quieran, además de un icono de pantalla de inicio para móviles y tabletas.

## [0.5.0] - 2026-09-19

Tú decides quién ve qué

### Added
- Permisos en Ajustes → Acceso, para todo el mundo a la vez o persona a persona: **Ver la actividad de todos** (las estadísticas y el historial de los demás, la página Usuarios, todas las reproducciones en directo), **Ver detalles de red** (direcciones IP, ID de dispositivos, local o remoto), **Ver el servidor** (la página Servidor, el registro del servidor, los inicios de sesión fallidos, las rutas de archivo) y **Gestionar FinStats** (ajustes, tareas, importación, eliminar reproducciones).
- Inicio de sesión por persona, para dejar entrar a unas pocas personas sin abrir FinStats a todos los usuarios de Jellyfin.
- Los cambios se aplican de inmediato, incluido retirar el acceso, y todo lo hace cumplir el servidor en lugar de ocultar botones.

### Changed
- Los administradores de Jellyfin siguen teniéndolo todo y son los únicos que pueden cambiar permisos; alguien que puede gestionar FinStats no puede conceder nada, ni a sí mismo ni a nadie.
- El resumen anual sigue siendo personal se conceda lo que se conceda.
- Nada cambia en las instalaciones existentes hasta que concedas algo.

## [0.4.5] - 2026-09-19

### Changed
- FinStats se distribuye con la licencia **GNU General Public License v3.0**. Nunca se publicó una versión anterior, así que nada salió con otras condiciones. Las licencias de las fuentes incluidas se distribuyen junto a ellas.

## [0.4.4] - 2026-09-19

### Added
- Esc vuelve atrás: en una biblioteca, un usuario o un título abierto, te devuelve a donde estabas, con la lista exactamente como la dejaste. Lo que esté abierto se cierra primero (los detalles de una reproducción, la búsqueda, una lista desplegable, el menú del móvil), y Esc no hace nada mientras escribes o usas un gráfico con el teclado.

## [0.4.3] - 2026-09-19

### Changed
- FinStats recuerda lo último que mostró cada página: volver a un perfil, una biblioteca o un periodo que ya has visto lo pinta al instante, foto de perfil incluida, mientras las cifras nuevas cargan detrás, y la página solo se vuelve a dibujar si algo ha cambiado. Lo recordado vive en la pestaña del navegador y se descarta al cerrar sesión o al cambiar cualquier cosa.
- Los marcadores de posición de carga solo aparecen cuando la carga es realmente lenta. Antes, cada página los mostraba al menos un tercio de segundo.

### Fixed
- Las bibliotecas que se eliminaron en Jellyfin y nunca tuvieron una reproducción ya no aparecen en la lista. Una biblioteca eliminada con historial se sigue mostrando, marcada como eliminada.

## [0.4.2] - 2026-09-19

### Fixed
- En la página Usuarios, la etiqueta **Administrador** quedaba en una línea propia y hacía esa fila más alta que las demás. Ahora las etiquetas van junto al nombre.

## [0.4.1] - 2026-09-19

### Fixed
- La palabra «null» ya no aparece bajo el título del panel fuera de diciembre y enero, cuando no hay ningún aviso del resumen anual que mostrar.

## [0.4.0] - 2026-09-19

Mira qué ha cambiado sin salir de la aplicación

### Added
- Pestaña **Novedades** que muestra todas las versiones, con la que estás usando marcada.
- La versión de la barra de estado enlaza a ella, y la pestaña muestra un punto tras una actualización hasta que la miras.

### Changed
- Volver a vincular el historial a elementos renombrados es mucho más rápido: de 18 segundos a un tercio de segundo con 666 títulos huérfanos, así que el arranque sigue siendo instantáneo a medida que crece el historial.

## [0.3.3] - 2026-09-19

### Changed
- La biblioteca se vuelve a leer justo cuando termina la tarea **Scan Media Library** de Jellyfin, nunca con un temporizador aparte y nunca mientras hay un escaneo en marcha. FinStats siempre ha sido de solo lectura respecto a Jellyfin, y nunca inicia un escaneo allí.
- Una lectura semanal de respaldo cubre los servidores que dependen de la supervisión en tiempo real, donde puede que la tarea de escaneo no se ejecute nunca.
- El intervalo antiguo ahora es solo una alternativa, para cuando el seguimiento está desactivado o el servidor no informa de ninguna tarea de escaneo.

### Added
- Interruptor **Seguir el escaneo de biblioteca de Jellyfin** en Ajustes → Recopilación, activado de forma predeterminada.

## [0.3.2] - 2026-09-19

### Fixed
- El historial sobrevive a los cambios de nombre. Jellyfin da un ID nuevo a un archivo renombrado, lo que dejaba sus reproducciones anteriores abandonadas bajo el nombre de carpeta en bruto (`Title (2010) [tmdbid-…] [imdbid-…]`) y sin póster. Ahora esas reproducciones se vuelven a vincular a la entrada actual de la biblioteca: por ID de TMDB/IMDb/TVDB cuando el nombre antiguo lleva uno y, si no, por título limpio y año, y los episodios por serie más número de temporada y de episodio. La coincidencia tiene que ser inequívoca, así que los títulos que de verdad se eliminaron se quedan como están.
- Los canales de TV en directo importados de Jellystat se contaban como películas, porque Jellystat no registra ningún tipo de elemento. Ahora se reconocen como TV en directo, y las reproducciones nuevas de TV en directo se registran con el tipo que da el propio Jellyfin.

### Changed
- La TV en directo queda totalmente fuera del resumen anual: listas de lo más visto, totales, personalidad, récords y posición.

### Added
- `finstats relink` para volver a vincular el historial cuando se pida. También se ejecuta al arrancar y después de cada lectura de la biblioteca.

## [0.3.1] - 2026-09-19

### Changed
- El resumen anual es estrictamente personal: todo el mundo, administradores incluidos, ve solo el suyo, y desaparecen la vista «Todos», el selector de usuario y el capítulo del servidor.
- El resumen anual se abre en el año que está «listo»: el año en curso durante diciembre y, si no, el anterior. Los demás años y «Últimos 12 meses» siguen a un clic.

## [0.3.0] - 2026-09-19

Tu año en resumen

### Added
- Pestaña **Resumen anual**, la historia de un año que se recorre desplazándose: horas vistas, las series, películas, pistas y géneros más vistos, y una personalidad de visionado (ave nocturna, alma madrugadora, fan del fin de semana, fan de los maratones, amante del cine, amante de la música, animal de costumbres) con gráficos por hora del día y por día de la semana.
- El año mes a mes, con el título que definió cada mes.
- Récords: el día más intenso, el mayor maratón, la racha diaria más larga, la reproducción más larga, el título más repetido, la primera reproducción del año y el título más antiguo visto.
- Descubrimientos: series nuevas empezadas, películas y episodios terminados, y las series de las que se vio exactamente un episodio.
- Un aviso de una línea en el panel en diciembre y enero cuando hay un resumen anual listo.

## [0.2.0] - 2026-09-19

Más de lo que Jellystat registró nunca

### Added
- Una cronología para cada reproducción: inicio, pausa, reanudación, salto, cambios de audio y de subtítulos, una reproducción directa que pasa a transcodificación, y parada. Además, recuentos de pausas y saltos, desde dónde se reanudó la reproducción y si el espectador estaba en la red local.
- Pestaña **Servidor**: versión de Jellyfin, actualizaciones y reinicios pendientes, uso del disco por biblioteca (Jellyfin 10.11+), complementos, resultados de las tareas programadas y todos los dispositivos registrados.
- Estadísticas de **Reproducción**: reproducciones simultáneas a lo largo del tiempo, qué clientes fuerzan transcodificaciones, hasta dónde llega la gente antes de parar, reproducciones locales frente a remotas y hábitos de visionado.
- De qué está hecha tu biblioteca: resoluciones, códecs, rango dinámico, contenedores, títulos por década, incorporaciones por mes, los títulos más grandes y los títulos que nadie ha visto nunca, contrastados tanto con el historial de FinStats como con las marcas de visto del propio Jellyfin.
- Panel: máximo de reproducciones simultáneas, datos transmitidos estimados, proporción de reproducciones remotas, géneros por tiempo de visualización e inicios de sesión fallidos.
- Páginas de elementos: enlaces a IMDb y TMDB, estudios, profundidad de bits y velocidad de fotogramas, y quién tiene el título marcado como visto en Jellyfin.
- Páginas de usuario: géneros, y las películas, los episodios y los favoritos que Jellyfin tiene registrados.

### Fixed
- La etiqueta de tiempo de una tarjeta de Reproduciendo ahora ya no se sale de la tarjeta.
- Las etiquetas largas de la página Reproducción se cortaban demasiado pronto.
- Una serie que ha salido de la biblioteca sigue mostrando su lista de temporadas y episodios.

## [0.1.0] - 2026-09-19

Primera versión

### Added
- Recopilación en directo de las sesiones de Jellyfin: usuario, título, cliente, dispositivo, dirección IP, método de reproducción, motivos de la transcodificación y aceleración por hardware, datos de vídeo, audio y subtítulos, y tiempo visto frente a tiempo en pausa.
- Inicio de sesión con tu cuenta de Jellyfin. Una configuración en dos pasos crea la clave de API propia de FinStats, y las contraseñas nunca se guardan. Los usuarios que no son administradores pueden, opcionalmente, iniciar sesión para ver solo sus propias estadísticas.
- Páginas de Panel, Actividad, Usuarios, Bibliotecas, Reproducción y Registro del servidor, una paleta de comandos (Ctrl/⌘ K) y una barra de estado en directo.
- Importación de Jellystat: las copias de seguridad grandes se vuelcan al disco poco a poco y se importan en segundos, e importar el mismo archivo dos veces no supone ningún peligro.
- Un único binario pequeño con una interfaz web integrada y un solo archivo SQLite, además de una imagen de Docker.
