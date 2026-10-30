# Notas da versão

Tudo o que mudou no FinStats, do mais novo ao mais antigo. Este arquivo é compilado no binário e
é o que a aba **Notas da versão** do app mostra.

Formato: `## [version] - date`, ou `## [version] - first day to release day` para uma versão feita
ao longo de vários dias, seguido de `### Added`, `### Changed`, `### Security`, `### Performance`, `### Stability`,
`### Fixed` ou `### Removed` (Adicionado, Alterado, Segurança, Desempenho, Estabilidade, Corrigido, Removido) com um
item por mudança. Segurança é algo que alguém poderia ter usado de forma indevida, Desempenho algo que ficou mais rápido
ou mais leve, Estabilidade algo que não pode mais travar, parar, vazar ou perder dados, e Corrigido algo que dava uma
resposta errada. Uma versão `x.y.0` traz uma linha curta de título sob o cabeçalho: esse título é o que o app usa como
manchete de toda a série.

## [2.3.1] - 2026-10-26 to 2026-10-30

### Security
- **A configuração pede um código de uso único**: enquanto não estiver configurado, o FinStats escreve um código no seu log ao iniciar (`docker logs finstats`), e o assistente o pede antes de testar um endereço ou configurar qualquer coisa. Do contrário, quem chegasse primeiro à porta podia apontar o FinStats para um Jellyfin próprio e ser o administrador dele, ou usar o teste para explorar a rede.
- **O acesso às bibliotecas segue o Jellyfin**: cada pessoa vê só as bibliotecas que o Jellyfin abre para ela e, nelas, só os títulos que o Jellyfin lista para ela dentro do limite de classificação, das tags e das regras para itens sem classificação, nas páginas de biblioteca, na busca, no elenco, nas imagens, em Adicionados recentemente, na Minha lista, em Em breve, no feed do calendário e nas notificações de títulos novos. Qualquer pessoa com sessão iniciada podia navegar por todas as bibliotecas.
- Um link podia mandar alguém das boas-vindas para outro site logo depois de entrar. Agora só as próprias páginas do FinStats são seguidas.
- O log do servidor e os logins com falha nos insights entregavam os endereços de login e a atividade de todos a qualquer um com *Ver o servidor*. As linhas de outra pessoa exigem *Ver a atividade de todos*, e um endereço *Ver detalhes de rede*, como em todo o resto.
- Um destino pessoal de notificações podia alcançar a rede local por meio de um nome que respondia diferente na segunda consulta. A conexão é feita ao endereço que foi verificado, mais faixas que nunca chegam à internet contam como locais, o próprio endereço público da casa também é recusado, e um nome que não pode ser consultado é recusado com as mesmas palavras que um local.
- Atrás de um proxy que adiciona uma linha `X-Forwarded-For` própria, ou a partir de uma rede IPv6, cada tentativa de login podia vir de um endereço novo e o limite nunca se aplicava. Um /48 IPv6 agora tem 50 tentativas em cinco minutos, um limite cheio recusa um endereço novo em vez de esquecer um antigo, o servidor inteiro para em 60 falhas por minuto, e um login com falha é auditado uma vez por endereço e janela.
- `FINSTATS_TRUST_PROXY` nomeia os proxies (endereços ou faixas) cujos `X-Forwarded-For`, `X-Forwarded-Host` e `X-Forwarded-Proto` são aceitos, e `1` ou `true` agora só confia num par em loopback ou numa rede privada. Ligado, qualquer par podia escolher o próprio endereço; desligado atrás de um proxy, dez tentativas erradas bloqueavam todo mundo.
- Depois que os usuários foram lidos, um login com um nome desconhecido, desativado, removido ou sem acesso ao FinStats é respondido como senha errada sem perguntar ao Jellyfin, e não antes do que o Jellyfin teria respondido. Esses nomes podiam bloquear contas do Jellyfin, contornar a regra de acesso remoto do Jellyfin ou se esconder atrás do endereço do FinStats.
- Quem o Jellyfin mantém na rede de casa (*Permitir conexões remotas* desligado) só entra no FinStats a partir dela, já que o Jellyfin julgava a opção pelo endereço do próprio FinStats, e uma leitura de usuários que não diz se alguém está desativado deixa essa pessoa de fora.
- Um cookie de sessão é marcado como Secure sempre que a página de login está em https, também atrás de um proxy que não informa isso.
- Um login que dá certo não perdoa mais as tentativas anteriores. Em https o cookie de sessão é um cookie `__Host-` que um site com um nome vizinho não consegue substituir e que não é aceito com nenhum outro nome, e uma saída encerra todas as sessões que o navegador enviou.
- Só uma chave de calendário abre o feed do calendário pelo endereço; qualquer outra chave vai no cabeçalho.
- Uma chave de API não pode mais conceder permissões, mudar quem entra ou o que é público, restaurar um backup, nem adicionar, alterar ou testar um serviço ou um destino de notificações: cada um desses dura mais que a chave, então exige um navegador com sessão iniciada.
- Toda resposta leva `Cross-Origin-Opener-Policy: same-origin` e uma `Permissions-Policy`, as respostas da API levam `Cross-Origin-Resource-Policy: same-origin` e são `no-store` salvo indicação contrária, e a CSP não permite mais imagens `blob:`. Uma página que abriu o FinStats podia depois transformar a aba dele numa página de login própria.
- Onde o FinStats sabe o esquema pelo qual é acessado (o `X-Forwarded-Proto` de um proxy confiável, ou o endereço público), uma gravação vinda de outro esquema é recusada como entre sites.
- `/auth/me`, a página de um título e a Saúde da biblioteca apontam para o Jellyfin no endereço público dele; o endereço pelo qual o próprio FinStats lê o Jellyfin vai só para administradores.
- Uma imagem só é buscada no Jellyfin para algo que o FinStats tem, incluindo os pôsteres e cartões de um perfil publicado, então as imagens de outra biblioteca não podem ser pedidas pelo id.
- As chaves de um tracker privado também são apagadas dos erros da fila de downloads, pelo formato e também pelo nome, e quem tem só *Ver o que está baixando* fica sabendo qual serviço falhou e como, nunca o endereço interno dele nem para onde ele redireciona.
- A página Servidor só mostra o último usuário de um dispositivo com *Ver a atividade de todos*, e Mídia sem vínculo também exige isso, além de *Gerenciar o FinStats*. Sem isso, a seção Tarefas conta só as próprias reproduções e streams, uma biblioteca removida é listada pelas próprias reproduções, e um quadro do Tautulli pendente não pode ser cancelado.
- As configurações mostram o serviço de endereço público só pelo host, e o endereço do Jellyfin e a pasta de dados só para administradores, e um título que a biblioteca perdeu é descrito apenas pelas reproduções que o leitor pode ver.
- Um backup do Tautulli cujas tabelas não eram tabelas comuns podia deixar todas as outras gravações esperando até o FinStats ser reiniciado. Um arquivo assim é recusado, e o SQLite lê o backup de forma defensiva.
- Um upload acima de 4 GB é recusado, e um backup do Tautulli compactado não é descompactado além do tamanho que declara; qualquer um dos dois podia encher o disco onde está o próprio banco de dados do FinStats.
- Uma resposta do Jellyfin, Sonarr, Radarr, Seerr, da consulta do endereço público ou do download do GeoIP não é lida além do máximo que pode ter, contado depois de descompactada.
- Qualquer usuário do Jellyfin podia fazer o FinStats parar de registrar reproduções de todos ao informar milhares de reproduções curtas de um título. Descobrir quem assistiu junto agora leva uma única passada e não segura nenhum bloqueio enquanto trabalha.
- Reproduções gravadas uma logo após a outra, ou um título reproduzido em rodízio de um dispositivo após o outro, não juntam mais desconhecidos em um grupo, e quem só pode ver a si mesmo só fica sabendo das pessoas ao lado de quem a própria reprodução correu, pelo tempo em que correu.
- Um link ainda podia mandar alguém das boas-vindas para outro site depois de entrar, por um caminho como `/.//`.
- Quem tem *Gerenciar o FinStats* não pode mais mudar quantos backups são mantidos nem com que frequência são gravados, a janela de mesclagem (agora no máximo uma hora) nem quando usuários, o log de atividades e os backups são lidos, e os backups dessa pessoa nunca empurram os de um administrador para fora; endereços de casa e as regras de viagem exigem o que os alertas exigem, e importar histórico exige também *Ver a atividade de todos*.
- Uma chave, token ou senha salva vai só para o servidor para o qual foi salva, e um destino de notificações só é criado ou alterado com a sessão iniciada, nunca com uma chave de API. Um administrador pode desativar ou remover o destino pessoal de outra pessoa, mas não movê-lo, desligar a verificação de certificados dele nem ler o tópico do ntfy ou a chave de usuário do Pushover.
- Webhooks do Discord e do Slack precisam ser `https`, já que o endereço deles é o token.
- Uma enxurrada de logins com falha não consegue mais esconder um verdadeiro nem soterrar os alertas: nomes que não são contas geram um aviso a cada quinze minutos que não nomeia ninguém, os alertas saem primeiro, e Slack e Discord mostram o que a mensagem diz como texto e não mencionam ninguém.
- A posição na Retrospectiva, a Capacidade, uma notificação e um perfil publicado não contam mais do que as páginas: nenhuma posição nem fatia do tempo de todos para quem só pode ver a si mesmo, momentos movimentados só como dias de três streams ou mais sem *Ver a atividade de todos*, nenhuma tarefa ou conexão com falha para quem não pode gerenciar o FinStats, e um perfil público muda uma vez por dia.
- Um backup do Tautulli com uma coluna gerada é recusado, e uma imagem só é buscada para uma reprodução importada quando a biblioteca tem o título.
- Uma restauração nunca abre o FinStats mais do que ele está: as configurações só voltam quando pedido, login, permissões e perfis públicos passam a ser o que tanto esta instalação quanto o backup permitem, e os endereços públicos continuam sendo os desta instalação. `finstats restore <file>` traz de volta só o histórico, como a página, e também as configurações com `--with-settings`.
- Uma restauração só do histórico não traz mais de volta endereços de casa, alertas silenciados, entradas de auditoria que se passam pelas desta instalação nem títulos localizados que a biblioteca não tem, e nenhuma restauração traz de volta reproduções, linhas de log ou dispositivos com data anterior a 1990 ou posterior a amanhã. Uma reprodução restaurada conta como importada, seja qual for a origem que alega, então um reinício nunca a continua.
- A pasta de dados pertence só ao usuário do próprio FinStats (0700, os arquivos 0600, `umask 077` na imagem). O FinStats se recusa a rodar como root sobre uma pasta de dados de outro usuário e indica o `docker exec -u` a usar, e o entrypoint muda o dono só da pasta e das próprias entradas do FinStats.
- O nome de um destino não pode conter caracteres de controle, e o nome de um dispositivo, de uma pessoa ou de um título não consegue mais forjar uma linha no log.
- O parâmetro de rota de um link não consegue mais alcançar outro caminho da API além do da própria página, e o construtor de elementos do FinUI só define um endereço quando ele é http(s) ou relativo, nunca `srcdoc`.
- Busca, perfis publicados e trabalho iniciado à mão não conseguem mais manter o FinStats ocupado: uma busca para em 200 caracteres, um perfil é guardado por idioma, os cartões esperam em uma fila curta, e uma tarefa, a consulta do endereço, o download da base de localização e uma mensagem de teste só começam de tempos em tempos, seja qual for a forma como são pedidos, com as mensagens de teste contadas por remetente. Baixar a base de localização também exige o que ler a página Segurança exige.
- O corpo de uma requisição precisa continuar chegando depois dos primeiros 15 segundos, uma resposta que ninguém lê é abandonada após 30 segundos e um endereço público mantém no máximo 64 conexões, enquanto proxies confiáveis, mesmo com endereços públicos, e endereços desta máquina ou de uma rede privada ficam fora do limite. Algumas centenas de requisições que nunca enviavam o corpo podiam impedir o FinStats de responder a qualquer pessoa.
- Um servidor de e-mail não é lido além do que uma resposta pode ter (1 MB para uma carta inteira), e uma mensagem de teste para depois do tempo que uma na fila pode levar. Um servidor que nunca terminava uma linha podia fazer o FinStats segurar cada vez mais memória até ser parado.
- A leitura de uma biblioteca é julgada pelos títulos que as leituras guardaram, não pelos que uma importação trouxe, e um título do Jellystat numa biblioteca já lida entra como removido. Uma importação de títulos inventados podia fazer cada leitura de biblioteca parar o FinStats, a cada início.
- O limite de 60 entradas com falha por minuto em todo o servidor conta só as tentativas enviadas ao Jellyfin, incluindo as que ainda esperam resposta. Nomes que o FinStats recusava sozinho deixavam qualquer pessoa com alguns endereços impedir todos de entrar, e muitos palpites enviados de uma vez chegavam todos ao Jellyfin.
- Quem o Jellyfin limita à rede de casa continua limitado em cada requisição, sessão e chaves incluídas, não só ao entrar; casa é uma rede privada ou o endereço público da casa encontrado pela consulta, nunca um endereço digitado como de casa, e atrás de um proxy que `FINSTATS_TRUST_PROXY` não nomeia a pessoa fica de fora. Um cookie ou chave levado para fora de casa abria tudo de qualquer lugar, e um gestor podia abrir a casa para um endereço à sua escolha.
- Uma chave de API não pode mais desligar o que perceberia seu uso indevido, nem os backups que desfariam o que ela fez: quantos backups são mantidos e com que frequência, a janela de mesclagem, endereços de casa e regras de viagem, os agendamentos de usuários, registro de atividade e backups, silenciar ou resolver todos os alertas, um backup na lixeira ou remover um destino de notificações. Cada um durava mais que a chave depois de revogada.
- Os pôsteres do Sonarr e do Radarr, e o próximo episódio de uma lista de desejos, também seguem o acesso às bibliotecas: uma série que o Jellyfin esconde de alguém não tem para essa pessoa nem pôster nem data. Contar os ids dos serviços revelava quais séries escondidas estavam chegando, e uma lista nomeava o próximo episódio de uma série escondida.
- O capítulo Juntos da retrospectiva nomeia só as pessoas cuja reprodução correu ao lado da sua, como faz a página Juntos. Um grupo em que você entrou por outra pessoa nomeava gente com quem você nunca assistiu.
- Uma importação do Jellystat ou Streamystats lê o arquivo inteiro antes de pegar o bloqueio de escrita do banco de dados e depois grava só as linhas que mantém. Lido sob o bloqueio, um arquivo de poucas linhas reais e gigabytes de qualquer outra coisa deixava esperando qualquer outra escrita, reproduções e entradas incluídas.
- Uma restauração guarda uma permissão por pessoa, para no máximo muitas vezes as pessoas de uma casa, e procura duplicatas de uma entrada de auditoria sem ler cada entrada que acabou de gravar. Um backup forjado podia segurar memória e o bloqueio de escrita do banco de dados por cada vez mais tempo durante a restauração.
- Uma restauração com as configurações liga a consulta do endereço público e o download do banco de localização só onde esta instalação também os tem ligados, e nunca mantém menos backups nem os grava com menos frequência que esta instalação. Um backup que mantinha um podia fazer a próxima execução remover todos os outros.
- Um tamanho de arquivo que nenhum disco poderia conter é lido como nenhum tamanho, e os tamanhos são somados de modo que nenhum total possa estourar. Um único tamanho importado fazia as páginas das bibliotecas responderem com erro para sempre.
- Quando o FinStats sabe que é acessado por https (é pedido no seu endereço público https, ou um proxy confiável diz isso), uma sessão só é tirada do seu cookie `__Host-`, e ali cada nova sessão é uma sessão https. Uma entrada enviada sem `Origin` por um proxy que não dizia nada criava uma sessão comum que um site vizinho podia plantar no navegador de alguém.
- A imagem é construída com cada ação do GitHub fixada em um commit e as duas imagens base em um digest, movidos só pelo Dependabot, e uma versão é construída sem o cache de build compartilhado. Uma tag movida na ação de outra pessoa poderia ter publicado qualquer imagem como FinStats.
- A folha de estilos do FinUI, desenhada na aparência escolhida por quem visita, só pode ser carregada pelas páginas do próprio FinStats. Uma página em outra porta do mesmo host podia ler nela se a pessoa estava conectada e qual predefinição escolheu.
- Toda entrada recusada é respondida em um mesmo momento contado a partir do envio, tenha sido o FinStats ou o Jellyfin a recusá-la, e um nome é reconhecido em maiúsculas ou minúsculas, também além de A a Z. A rapidez de uma recusa revelava quais nomes eram de alguém, e “Åse” digitando `åse` era recusada sem que o Jellyfin fosse consultado.
- Erros de download ocultam os endereços dos clientes de quem não é administrador. Quem podia ver a fila antes podia ver hosts, portas e caminhos internos.
- Os limites de chaves e destinos de notificação são aplicados de forma atômica. Antes, solicitações simultâneas podiam ultrapassá-los.

### Performance
- Um perfil publicado é calculado no máximo uma vez por minuto, e não de novo para cada imagem e cartão nele.
- O histórico de uma pessoa é verificado em busca de alertas de segurança uma vez de cada vez, não importa quantas reproduções comecem.
- Entrar ou sair, revogar uma chave ou mudar as permissões de uma pessoa pede só às conexões ao vivo envolvidas que olhem de novo.
- O quadro de ligação do Tautulli lê o histórico uma vez, e não uma vez por usuário do Plex.
- Uma imagem que falta no cache é buscada por uma única requisição enquanto as outras esperam, e uma que a origem não conseguiu entregar não é pedida de novo por 30 segundos.
- Os arquivos da interface web são compactados com gzip uma vez e guardados, e as fontes não são compactadas de novo.

### Stability
- Se um nome estiver ocupado, o backup tenta outro sem substituir um backup completo nem remover o arquivo parcial recente de outra exportação.
- Um backup no formato antigo do Jellystat é lido uma linha por vez em vez de inteiro, e uma linha mais longa do que a de qualquer backup é recusada.
- Dois uploads do Tautulli ao mesmo tempo não gravam mais em um único arquivo.
- Um número de página muito além do fim dos títulos de uma biblioteca ou da Saúde da biblioteca é uma página vazia, não um erro.
- Uma importação ou restauração cujo upload foi interrompido não recusa mais todas as outras até reiniciar, e nem ela nem uma importação abandonada antes de começar deixa um arquivo para trás.
- Uma linha de um backup não é lida além de 8 MB, e chega um backup do Tautulli por vez.
- Uma reprodução guarda no máximo 2.000 eventos, e uma importação ignora uma reprodução que não começa e depois termina entre 1990 e amanhã.
- Um número de episódio muito abaixo de zero não trava mais todas as leituras da biblioteca: a Saúde da biblioteca (o resumo também), a Capacidade, a revinculação e a localização somam e comparam números sem dar a volta, e um build de release verifica estouro, então o que escapar interrompe uma tarefa em vez de entrar em loop.
- Um número de temporada, de episódio ou final só é guardado onde o Jellyfin conseguiria guardá-lo (0 a 2.147.483.647), venha do Jellyfin, do Jellystat ou do Tautulli.
- Um backup do Tautulli cujos ids se repetem é recusado, já que alguns MB de linhas sob um id esgotavam a memória do processo, e um backup é aberto como imutável, sem deixar `-wal` ou `-shm` para trás.
- Uma conexão precisa enviar a requisição em 15 segundos e no máximo 512 são atendidas ao mesmo tempo: algumas centenas de sockets semiabertos tiravam o FinStats do ar. Um quadro no canal ao vivo é limitado a 4 KB.
- Alguns kilobytes vindos do Sonarr, Radarr ou Seerr não viram mais gigabytes de memória: uma lista não guarda mais do que o FinStats lê dela.
- Um destino de notificações que nunca responde atrasa só a si mesmo: os destinos recebem os envios lado a lado, e um que vive estourando o tempo é deixado de lado por dez minutos.
- O tempo restante e a velocidade total de uma fila, os números de uma reprodução restaurada e o dia em que uma reprodução sai da lixeira não estouram mais.
- Uma restauração lê o arquivo inteiro antes de pegar o bloqueio de gravação, recusa um que se descompacta em muito mais do que o seu tamanho, e grava em lotes, então o coletor continua registrando enquanto isso.
- Os backups recebem nomes em UTC e nunca sobrescrevem um arquivo existente, e só as cópias pré-atualização do próprio FinStats são podadas.
- Uma linha do log do servidor com data no futuro não impede mais a leitura do log de atividades: ele é lido a partir da linha mais nova que não está no futuro.
- Requisições que chegam juntas a um link publicado calculam a resposta dele uma vez, e os cartões dele são desenhados numa fila própria, então quem tem o link não consegue mais fazer os cartões da Retrospectiva do dono responderem 503.
- As reproduções de uma biblioteca que falta na lista de bibliotecas do Jellyfin esperam dois dias antes de passar para a cópia do mesmo título em outra biblioteca. Uma única lista que deixava uma biblioteca de fora as movia na hora, e elas continuavam movidas quando a biblioteca voltava.
- Um disco que o Jellyfin informa como maior do que qualquer número comporta não faz mais falhar toda leitura dos detalhes do servidor.
- As leituras do Jellyfin param quando uma página se repete sem novos IDs.

### Fixed
- Logins com falha numa conta são um aviso por janela que vira alerta aos vinte, e um nome só conta contra uma conta quando digitado inteiro ("x alice" não conta mais contra alice).
- As mensagens do Discord e do Slack são escapadas primeiro e cortadas depois: um nome de dispositivo cheio de Markdown fazia o Discord recusar todos os alertas que o mencionavam.
- Um endereço IPv6 público é aceito para um destino pessoal.
- Atrás de um proxy que faz o próprio login das pessoas (nginx `auth_basic`), o cookie do FinStats não é mais recusado.
- Um nome de exibição contendo `{{url}}` aparece como foi escrito na prévia de uma página publicada.
- Uma retrospectiva pedida para o ano 9999 mostra o ano padrão em vez de um erro.
- Uma página ao vivo cuja permissão foi retirada e devolvida enquanto estava aberta volta a ficar atualizada, sem recarregar.

### Removed
- O **Synnmørsk** não é mais um dos idiomas do FinStats: um navegador que o escolheu lê nynorsk, a língua escrita da qual ele foi feito.

## [2.3.0] - 2026-10-07 to 2026-10-26

Capacidade, um canal ao vivo, modo simples e idiomas

### Added
- **Idiomas**: o FinStats fala 23 idiomas. O inglês britânico e americano e o norueguês bokmål e nynorsk são escritos e revisados pelo mantenedor; dinamarquês, alemão, espanhol, finlandês, francês, hindi, islandês, italiano, japonês, coreano, holandês, polonês, português (Brasil), russo, sueco, turco, ucraniano e chinês (simplificado e tradicional) são traduções que ninguém aqui revisou, e dizem em inglês o que ainda não têm. Todas as páginas dos dois layouts, o perfil público, as Notas da versão e o que o servidor responde em palavras (um erro, os cards da Retrospectiva, o que uma tarefa do Jellyfin faz) aparecem no idioma escolhido em Configurações → Aparência → Idioma, por navegador; enquanto nenhum é escolhido, vale o idioma do próprio navegador. Números, datas, unidades e listas são escritos como esse idioma os escreve. Um script pode pedir um idioma com `Accept-Language`.
- **Modo simples**: um segundo layout para quem vem do Plex, organizado como um servidor clássico de estatísticas do Plex. Uma barra no topo no lugar do menu, e só as páginas de estatísticas: Início, Bibliotecas, Usuários, Histórico, Gráficos e Configurações, com uma engrenagem para o seu perfil, as Configurações, as Notas da versão, o tema e a saída. Escolhido por navegador em Configurações → Aparência → Layout. Ele carrega só a sua própria metade do FinStats, por isso abre mais rápido; Retrospectiva, Juntos, Pipeline, Segurança e as páginas do servidor ficam no layout do FinStats. Ele usa as cores do próprio FinStats, no claro e no escuro, e nada nele se mexe.
- **Início do modo simples**: o que está tocando, um card por stream (como é reproduzido, se de casa ou de fora, quando termina), as estatísticas de exibição em cards ranqueados por número de reproduções ou por duração nos últimos N dias, o que cada biblioteca contém e o que chegou recentemente.
- **Gráficos do modo simples**: reproduções por tipo de mídia (dia a dia, por dia da semana e hora, os dez apps e as dez pessoas principais), por forma de reprodução (dia a dia, o máximo de streams simultâneos de cada dia, a resolução do arquivo e a enviada, apps e pessoas) e mês a mês, por número de reproduções ou por duração, para todos ou para as pessoas escolhidas.
- **Bibliotecas, Usuários e títulos do modo simples**: tabelas de todas as bibliotecas e pessoas com o que foi reproduzido por último, em quê e de onde (o endereço só com *Ver detalhes de rede*), e uma página para cada biblioteca, pessoa e título com seus números e seu próprio histórico.
- Para scripts: `/api/stats/split` (reproduções por app, pessoa, resolução, dia ou mês, divididas por tipo de mídia ou por forma de reprodução); listas dos mais vistos por espectadores distintos (`sort=users`); atividade limitada às reproduções assistidas (`watched=1`); insights só das partes pedidas (`parts=`); o pico dividido em transcodificações, transmissões diretas e reproduções diretas, também por dia; o que cada biblioteca contém e o que reproduziu por último; adicionados recentemente por tipo (`kind=`); o mapa de calor por tipo de mídia; a última reprodução de uma pessoa; se um stream é reproduzido de casa.
- **Um canal ao vivo**: o painel, os downloads do Pipeline e a barra de status se mantêm atualizados por um único WebSocket por aba (`/api/live`), então o que está tocando e o que está baixando muda na tela no momento em que muda, e não na próxima consulta. Atrás de um proxy reverso que não repassa upgrades de WebSocket, ou numa conexão que caiu sem aviso, as páginas perguntam a cada poucos segundos, como antes. `/api/now-playing`, `/api/downloads` e `/api/summary` continuam como estão, para scripts.
- **Servidor → Capacidade**: se este servidor vai dar conta, a julgar pelo que o FinStats já viu. Os momentos mais movimentados já registrados e os dos últimos 90 dias: streams simultâneos, e quantos tiveram a imagem transcodificada com hardware, sem ele ou sem informação (um histórico importado do Tautulli é do Plex, e fica de fora). Cada disco onde estão as bibliotecas e a rapidez com que o que o Jellyfin adicionou em seis meses o enche (um disco ocupado mais recentemente é medido a partir de um mês depois do seu primeiro arquivo, já que o primeiro mês é o da carga inicial) e, onde o Jellyfin 10.11 ou mais novo informa o espaço livre, quanto tempo ele dura nesse ritmo. O FinStats nunca diz quantos streams o servidor aguentaria.
- **Exportar .ics** em Pipeline → Em breve: a lista do jeito que você a filtrou, baixada como arquivo de calendário para importar uma vez. Ela não se atualiza sozinha; uma chave de calendário em Configurações → Chaves de API continua dando um calendário que se atualiza.
- **Vale uma olhada**: a Capacidade avisa quando toda transcodificação que o Jellyfin relatou em 90 dias rodou sem aceleração de hardware embora transcodificar seja comum, quando a varredura de biblioteca do Jellyfin roda um quarto do tempo ou mais, e quando um disco tem menos de seis meses de espaço pela frente. Cada aviso vem com seus números e pode ser deixado de lado como os achados da Saúde da biblioteca.
- **Do que sua biblioteca é feita** pode ser limitado a uma biblioteca ou a várias, em Bibliotecas e na página de cada biblioteca (`library_id=a,b`).
- **Formatos de data e hora** em Configurações → Aparência: uma data como 31.01.2026, 31/01/2026, 31-01-2026, 01/31/2026, 2026-01-31, 2026.01.31 ou 2026/01/31, e uma hora em 24 ou 12 horas, com ou sem zero à frente, ou cada uma do jeito do idioma. Guardados por este navegador.
- **Uma boas-vindas**: na primeira vez que alguém entra, em qualquer navegador, e no fim da configuração, o FinStats pergunta o idioma, os formatos de data e hora e o layout, com uma imagem de cada. Pular mantém inglês britânico, 31/01/2026, 13:05 e FinStats.

### Changed
- O logo dentro de uma página é a marca sozinha ao lado do nome, composto em Inter seja qual for a fonte que a predefinição escolha; onde ele aparece sozinho (o favicon), mantém o bloco arredondado.
- Uma reprodução transcodificada em software é registrada como tal quando o Jellyfin informa isso; antes, ela parecia igual à de um servidor que não informa. As páginas continuam chamando as duas de Software.
- Uma atualização agora executa, uma vez e da versão mais antiga para a mais nova, o que cada versão precisa fazer num banco de dados gravado por um FinStats mais antigo, além das migrações. A da 2.3.0 lê os detalhes do servidor na primeira inicialização, para a Capacidade já estar lá de cara.

### Performance
- **Uma página deixada aberta não pede nada**: o painel fazia 30 requisições por minuto para se manter atualizado e os downloads do Pipeline, 18; com o canal ao vivo, nenhum dos dois faz nenhuma. Não importa quantas abas estejam abertas, o FinStats não pede ao Jellyfin, ao Sonarr e ao Radarr nada além do que pedia antes, e uma aba oculta continua deixando a fila de downloads desacelerar.
- A página de uma pessoa lê só as reproduções dessa pessoa: os totais e a última reprodução eram calculados sobre todas as reproduções do servidor e só depois filtrados para ela, cerca de quatro segundos com um milhão de reproduções.
- A página de uma biblioteca lê só os títulos e as reproduções dessa biblioteca, onde antes contava os de todas e ficava com uma: um terço do tempo com um milhão de reproduções.
- Com o relógio do Jellyfin adiantado em relação ao do FinStats, *Após a varredura de biblioteca do Jellyfin* lia a biblioteca inteira de novo a cada minuto até a diferença passar; agora cada varredura é seguida uma vez só.
- Um arquivo GeoIP que não podia ser aberto era copiado de novo, cem megabytes ou mais, e todos os endereços eram localizados outra vez, a cada quinze minutos; agora ele só é tentado de novo quando muda.
- A Saúde da biblioteca abre na hora depois que o ponteiro para sobre o link: o que era buscado antecipadamente ficava guardado onde a página nunca procurava.

### Stability
- Uma atualização que falhava depois de uma migração já ter rodado, e que o Docker reiniciava na mesma hora, fazia uma nova cópia do banco de dados meio atualizado a cada inicialização, e três inicializações descartavam a única cópia que a versão anterior conseguia abrir. A cópia de antes da atualização agora é mantida.
- Um título excluído no Jellyfin enquanto a biblioteca era lida (um arquivo substituído pelo Radarr) podia fazer a leitura deixar passar outro título que ainda estava lá e marcá-lo como removido, e as reproduções dele podiam então ser movidas para outra cópia. O que uma leitura não viu agora é consultado primeiro pelo id.
- Restaurar um backup feito antes de um arquivo ser renomeado ou trocado por uma versão melhor restaurava pela segunda vez toda reprodução movida desde então para o novo arquivo, que ficava duplicada para sempre.
- Um pedido na resposta do Seerr que o FinStats não conseguia ler (um campo renomeado numa atualização do Seerr) era marcado como removido pela leitura completa diária.
- Uma tarefa que esbarrava num bug ficava “em execução” para sempre, e nenhum gatilho a iniciava de novo até o FinStats ser reiniciado; agora ela termina como falha.
- Um zip do Tautulli danificado interrompia o envio antes de remover o arquivo, que guarda os tokens de todos os usuários do Plex, até a limpeza de seis horas.
- Uma importação ou restauração interrompida enquanto o arquivo chegava deixava esse arquivo na pasta de dados para sempre, com endereços e tudo; agora ele é removido na próxima inicialização.
- Uma imagem podia ser servida pela metade enquanto era guardada em cache, e o navegador então a mantinha assim.
- Chaves de API revogadas eram mantidas para sempre, por mais que se criassem e revogassem.
- Uma leitura das marcas de assistido e favorito de alguém que voltava vazia ou ilegível apagava todas até a próxima leitura; agora ela é recusada, como uma leitura vazia de uma biblioteca.
- Um episódio com um número final que nenhum episódio pode ter (uma data escrita ali) podia manter a Saúde da biblioteca ocupada sem fim depois de uma leitura da biblioteca.

### Fixed
- Uma nota de versão com código dentro de negrito (as da 2.0.2 e da 2.0.3) mostrava as crases em vez de exibir o código.
- Um gráfico de contagens podia numerar o eixo 0, 3, 5, 8, 10, ou até 0, 0, 1, 1, 1: passos de 2,5 escritos arredondados. As contagens sobem agora em passos inteiros.
- Uma chave de calendário criada por alguém sem permissão para gerenciar o FinStats vinha com um link no endereço do próprio navegador em vez do endereço público do FinStats, então só funcionava de dentro de casa; a página também pedia configurações que lhe eram negadas.
- A última reprodução de uma pessoa podia ser alguns segundos curta demais para contar como reprodução em qualquer outro lugar, citando um título que o Histórico dela não mostra.
- Uma restauração em Excluídos recentemente que falhava não dizia nada e deixava o botão girando; o mesmo acontecia com um *Trazer de volta* com falha na Saúde da biblioteca e com um *Reabrir* ou *Resolver todos* com falha na Segurança.
- O feed do calendário se chama FinStats num app de calendário, e não finstats.
- A fila de downloads mostrava quem tinha pedido cada download a qualquer um com permissão para vê-la; o pedido de outra pessoa agora exige *Ver a atividade de todos*, como no Pipeline.
- Retirar *Receber notificações* não parava o que era enviado aos destinos dessa pessoa, e ela também não conseguia mais desligá-los.
- Uma reprodução pausada e retomada dentro da janela de mesclagem era anunciada como encerrada uma vez, e a retomada e o fim real, nunca.
- As reproduções de uma faixa podiam ser movidas para a faixa de mesmo nome de outro artista (“Intro”) quando o álbum dela saía da biblioteca.
- Um destino de notificação escrito como endereço IPv4 dentro de IPv6 (NAT64, 6to4) passava por público enquanto chegava à rede local.
- Uma conexão do Sonarr ou do Radarr apontada para outro servidor continuava mostrando os pôsteres do servidor antigo por uma semana.
- Os alertas da Segurança diziam que nada precisava de atenção quando duas ou mais pessoas eram escolhidas, e uma página que ficava vazia ao resolver o último alerta não tinha como voltar.
- Toda chave de API que ainda não tinha expirado dizia que expirava “agora mesmo”.
- Um diálogo continuava sobre a página depois de Voltar, e a página não rolava até ele ser fechado.
- O log de auditoria mostrava oito tipos de entrada (uma reprodução ou um backup trazido de volta, a lixeira esvaziada, perfis públicos, uma aparência alterada) pelos seus ids, sem nada em Detalhe.
- Colocar um título na Minha lista pelo menu de contexto falhava sem aviso quando o servidor recusava.
- *Assistido junto* no painel tratava várias pessoas escolhidas de uma vez como uma só.
- A retrospectiva escrevia horas e dias abaixo de dez com a vírgula decimal do navegador ao lado dos seus próprios separadores de milhar.
- Capacidade dizia que um disco encheria "no ritmo dos últimos 0 meses".
- Na página Segurança, os endereços sem lugar contavam as reproduções de todos de todo o tempo, qualquer que fosse a pessoa e o período a que a página estava restrita.
- Alguém com *Gerenciar o FinStats* mas sem *Ver a atividade de todos* podia mandar para a lixeira a reprodução de qualquer pessoa.
- Um cartão da Retrospectiva ou de um perfil público podia vir do cache do navegador no idioma escolhido antes.
- Os downloads enviados pelo canal ao vivo estavam em inglês britânico, qualquer que fosse o idioma da página.
- Um login que mudava o que alguém pode ver só chegava às outras abas abertas da pessoa na próxima verificação de meio minuto.
- Uma aba que cedia sua conexão ao vivo a uma mais nova pegava uma de volta na página seguinte, então as abas de uma pessoa viviam fechando as conexões umas das outras.
- As tabelas ordenavam como texto os tamanhos de 1.000 a 1.023 unidades nos idiomas que agrupam dígitos com espaço, e também as porcentagens em turco.
- Um título com um marcador no nome, como "Big {device} Bunny", era preenchido pelas mensagens do servidor.
- As tabelas ordenavam como texto durações e contagens sobre um total (5 de 26) em todos os idiomas exceto o inglês e os nórdicos.
- Uma aba ainda aberta em um idioma recebia o servidor em outro assim que outra aba o escolhia.
- Uma palavra ao lado de um número tinha a forma errada em polonês, russo e ucraniano ("5 wieczory", "2 потоков").
- Capacidade contava o primeiro preenchimento de uma biblioteca como crescimento e dividia o que cada biblioteca adicionou por seis meses, qualquer que fosse o período medido; um arquivo que tomou o lugar de outro (uma melhoria do Radarr ou do Sonarr, ou um arquivo a que o Jellyfin deu um novo id) contava como todo novo, quando só a diferença era.
- Quem assistia junto com o SyncPlay não formava um grupo quando um entrava minutos depois do outro; reproduções que terminam juntas no mesmo ponto ficam juntas agora, e o histórico é reagrupado uma vez.
- Alguém sem *Ver a atividade de todos* podia saber pelo *Nunca assistidos* de uma biblioteca que outra pessoa tinha reproduzido um título; agora conta só o que a própria pessoa assistiu.

## [2.2.0] - 2026-10-01 to 2026-10-07

Saúde da biblioteca e FinUI

### Added
- **Um menu só seu no computador**: Configurações → Aparência oferece oito: a barra lateral como era, uma agrupada pelo propósito das páginas, um trilho de ícones que se abre na barra lateral sob o ponteiro, uma barra lateral que começa pela busca, um dock, uma barra de comandos, páginas que você fixa no topo, e blocos coloridos. As barras laterais podem ficar à direita. O dock se esconde enquanto você rola para baixo e volta quando você rola para cima ou aponta para a parte de baixo da janela; com ele, a barra de status vai para o topo.
- **Um menu só seu no celular**: uma barra de abas com Mais (o novo padrão), uma folha que abre até a metade, a tela inteira, as páginas em leque em volta do polegar, ou uma barra de endereço que cresce até virar o menu. Cada um busca nas suas páginas e passa quaisquer outras palavras para a busca.
- Os dois ficam guardados no navegador em que foram escolhidos, então o celular e o computador mantêm cada um o seu; os dois são componentes do FinUI, `desktop-nav` e `mobile-nav`.
- **Um menu de contexto**: clique com o botão direito num título, numa pessoa ou numa biblioteca em qualquer lugar do FinStats (um pôster no painel, uma linha de lista, um nome numa tabela, um filme em Em breve que ainda não está na biblioteca), ou segure o dedo sobre ele, ou aperte a tecla de menu, e um menu cresce a partir do ponto pressionado: abrir, abrir numa nova aba, pôr um filme ou uma série na Minha lista, abrir no Jellyfin ou no TMDB e no TVDB, copiar o link. Shift com o botão direito continua sendo do próprio navegador. O `context-menu` do FinUI.
- **Busca sem diálogo**: a busca cresce a partir do seu menu (uma barra lateral se alarga até ela, o dock sobe até ela, o nome da barra de comandos vira o campo, o menu do celular a abriga) ou, se escolhido em Configurações → Aparência, ocupa o lugar da página enquanto o menu fica. Ctrl+Space a abre como antes; a janela por cima de tudo acabou. O `search` do FinUI.
- **Tudo numa biblioteca**: o card Adicionados recentemente de uma biblioteca oferece *Tudo o que tem nela*, cada filme, série ou álbum que ela contém em pôsteres, por nome, ano, data de adição ou tamanho, filtrado enquanto você digita. Mais itens chegam conforme você rola.
- **Servidor → Saúde da biblioteca**: o FinStats compara cada arquivo com os vizinhos e lista o que destoa, cada item com os números por trás. Ele só informa; nada é alterado, excluído ou escaneado de novo no Jellyfin.
- **Lacunas**: episódios faltando entre dois arquivos de uma temporada (“Temporada 2: os episódios 4 e 5 estão faltando entre o 3 e o 6”). Um arquivo que contém dois episódios conta como os dois; especiais e o fim de uma temporada ficam de fora.
- **Temporadas diferentes** do resto da série em resolução, HDR ou codec, e **episódios destoantes**: um arquivo diferente da sua temporada, como um episódio SD numa temporada 1080p. A resolução vale pela largura ou pela altura, então um filme recortado em 1920×800 é 1080p.
- **Cópias e versões**: o mesmo filme ou episódio duas vezes, pelos ids de provedor que a Minha lista e o Pipeline já usam. Duas cópias na mesma resolução mostram o espaço guardado em dobro; uma cópia 4K ao lado de uma 1080p é listada como duas versões e nunca contada como desperdício.
- **Arquivos magros**: uma taxa de bits muito abaixo do que a resolução e o codec costumam exigir (“4K HEVC a 1,1 Mbps”); os episódios magros de uma temporada são um só achado.
- **Dublagens que param**: um idioma de áudio que algumas temporadas têm em todos os episódios e outras em nenhum.
- **Nunca identificados**: filmes e séries que o Jellyfin associou sem nenhum id do TMDB, TVDB ou IMDb.
- **Dispensar** um achado que está assim de propósito, com uma nota. Ele volta sozinho quando os arquivos dele mudam. Quem pode gerenciar o FinStats e ver o servidor pode dispensar; as dispensas ficam no log de auditoria e fazem parte do backup.
- A página de uma biblioteca diz quantas coisas há para olhar ali, e leva até elas.
- **FinUI**, os componentes de que o FinStats é feito, é um projeto próprio (`github.com/finstats/finui`): cada botão, card e controle em cada estado e nos dois temas em `finui.finstats.no`, e o **FinUI create** para mudar as cores, o raio, a densidade e mais enquanto você vê, e levar tudo para casa com um só comando. **Configurações → Sistema** leva até lá.
- **Configurações → Aparência**: cada um escolhe como o FinStats aparece para si, pelo código de uma predefinição do FinUI: cor de base, destaque, cores dos gráficos, contraste, raio, densidade, bordas, cards, botões, campos, tabelas, realce, movimento, ícones, o menu, a página por trás de tudo, o anel de foco, como os títulos são compostos, e as fontes do texto, dos títulos e dos números. A sua escolha é só sua; a página mostra o que o código significa e leva ao FinUI create para mudá-lo.
- **Estilos**: onze visuais completos para começar, numa só escolha no FinUI create, cujo código Configurações → Aparência aceita. São eles Washi (o FinStats como vem), Studio, Noir, Matinee, Console, Pastille, Ledger, Gazette, Arcade, Canopy e Harbour. O próprio FinStats vem com um único visual e não oferece lista de estilos.
- **Ícones que se mexem**: todo ícone num link, num botão ou numa aba faz o seu próprio movimento uma vez quando apontado (atualizar gira, a seta do download cai na bandeja, a lixeira levanta a tampa), e um atualizar que foi pressionado continua girando até a resposta chegar. Ficam parados com movimento reduzido e com Movimento: Desligado.
- **Movimento, com o FinMotion**: os componentes do FinUI agora se mexem: o botão de um interruptor é lançado de um lado ao outro, o sublinhado de uma aba desliza até a próxima, um campo recusado balança a cabeça, uma cópia é marcada num só traço, os avisos se empilham como cartas na mão e se abrem em leque quando apontados, a folha do celular cede como um elástico quando puxada, as linhas de uma tabela ordenada vão para os seus lugares, um número que muda gira como um odômetro e os gráficos se desenham na primeira vez em que são vistos. Tudo isso segue a escolha de Movimento em Configurações → Aparência e fica parado com movimento reduzido. O FinMotion é um projeto próprio (`github.com/finstats/finmotion`), posto por cima do FinUI.
- **Mais ícones no FinUI**: 239 agora, contra 71 antes, cada um com um movimento próprio (volume para cima e para baixo, um fone de ouvido e um silenciado, um lápis que escreve, um sino que toca, um celular que vibra), e todos podem ser baixados da galeria do FinUI como SVG, parado ou animado.
- **Fontes**: trinta e quatro famílias de licença aberta para escolher (Geist, Roboto, Manrope, IBM Plex Sans, Montserrat, Lora, Playfair Display, Fira Code e outras), incluídas como a Inter, nunca buscadas de outro lugar, e carregadas só quando escolhidas.
- **Mais bases e destaques**: Nord, Solarized, Gruvbox, Catppuccin, Rosé Pine, Tokyo Night, Dracula, Everforest e Ocean; mais dezesseis destaques, dez deles com uma cor de dia e outra à noite. Toda base agora tem o seu próprio papel de dia: os cinzas ficavam parecidos no tema claro.
- **Excluir pode ser desfeito por 30 dias**: uma reprodução ou um backup excluído espera na lixeira, e **Desfazer** no aviso que aparece o traz de volta. A Atividade e Configurações → Backups listam cada uma o que foi excluído recentemente, com a data em que some de vez, sempre que houver algo. Uma reprodução na lixeira fica fora de todo total, gráfico, retrospectiva e perfil, e uma importação ou restauração não a traz de volta como nova; quem a assistiu com alguém conta como sozinho até ela voltar.

### Changed
- O Dark Reader e extensões parecidas deixam as páginas do FinStats em paz (`darkreader-lock`): o FinStats desenha os dois temas por conta própria, e uma página repintada brigava com eles.
- No celular o menu não desliza mais da esquerda: a barra de abas fica embaixo, a menos que outro seja escolhido.
- O FinStats lê a sua biblioteca uma vez depois desta atualização, para saber onde terminam os arquivos que contêm vários episódios.
- Na página de um título, **Adicionar à Minha lista** tem a mesma altura de **Abrir no Jellyfin** ao lado; era um tamanho menor.
- Os filtros de período e de pessoas ficam no topo da tela enquanto a página rola, então dá para trocar 7d por Tudo lá do fim de uma página longa; o mesmo vale para os filtros de pedidos e de lançamentos do Pipeline.
- O botão de voltar ao topo se afasta do menu que ocupa a parte de baixo da janela: bem no canto ao lado do dock, logo acima da barra de abas, do botão do polegar ou da barra de endereço do celular, e acima da barra de status nos outros casos. É o `to-top` do FinUI.
- Excluir uma reprodução ou um backup o move para a lixeira em vez de removê-lo na hora (`PUT` com `{"deleted": true}`; `DELETE` saiu da API). Backups além do número mantido continuam sendo removidos de vez.
- As **Notas da versão** são uma folha só em vez de uma pilha de cards: cada nota é uma linha própria com o seu tipo (Adicionado, Alterado, Corrigido e os demais) e o ícone desse tipo numa coluna à frente, sob um título para cada série. A lista de versões continua à direita.

### Performance
- Abrir o FinStats baixa só o que mudou: uma folha de estilo, um script ou uma fonte que o navegador já tem é confirmado em poucos bytes em vez de ser enviado de novo. Toda visita buscava todas as folhas de estilo e todos os módulos do FinStats por inteiro, embora cada um trouxesse uma tag para distingui-los.

### Stability
- Restaurar um backup lê as configurações só da linha de configurações: um arquivo de um FinStats mais novo que guarda mais coisas ali tinha as configurações descartadas sem aviso.
- Uma aba aberta não cresce mais a cada página visitada: toda página deixada para trás ficava na memória por trás da que estava na tela (cerca de 2.600 elementos por visita), presa pela lista do que o FinMotion move e pelo listener de rolagem da barra de filtros, até a aba ser fechada.
- Um comando digitado errado (`finstats bakcup`), ou `restore` sem o arquivo, não cria nem atualiza mais um banco de dados antes de dizer que não sabe o que fazer.

### Fixed
- O pequeno gráfico de um bloco não passa mais da borda quando o número ao lado é largo (“56d 23h”), ou quando quatro blocos dividem uma linha mais estreita: o gráfico cede e o número mantém a largura.
- No tema claro um botão principal ficava violeta sob o ponteiro; agora mantém a cor do selo.
- Nas Configurações um interruptor fica alinhado com o nome e a ajuda da configuração; antes ficava pendurado no topo da linha.
- Numa lista ranqueada sem nada sob o nome, como Usuários mais ativos no painel, o nome ficava no topo da linha ao lado de uma imagem centralizada; agora fica centralizado.
- Uma linha de blocos de estatística numa largura entre celular e desktop (quatro blocos, três por linha) deixava o último sozinho numa linha própria; agora quatro blocos ficam quatro, dois ou um por linha, e três ficam três ou um.
- A leitura do servidor contava um dispositivo em que ninguém tinha entrado, que ela não guarda: a tarefa dizia três dispositivos onde a página Servidor listava dois.
- Um segundo, um minuto, uma hora: os alertas diziam “1 segundos de diferença” e “em 1 minutos”, e uma tarefa do Jellyfin com intervalo de um minuto rodava “a cada 1 minutos”.
- `finstats --help` cita todas as variáveis que o FinStats lê; faltavam `FINSTATS_PUBLIC_IP_URL`, `FINSTATS_ALLOW_LIBRARY_SHRINK` e `FINSTATS_SKIP_PREUPDATE_BACKUP`.
- O assistente de configuração dizia “não foi possível conectar” sobre algo que nem era um endereço, numa frase que emendava na seguinte; agora diz que aquilo não é um endereço.
- Em Configurações → Backups, uma pergunta feita enquanto um backup era gravado sumia quando o backup terminava.
- Com o menu como trilho que abre, fechar a busca mostrava a busca aberta e o trilho fechado ao mesmo tempo e depois deixava o trilho aberto até o ponteiro entrar e sair. Agora o Esc devolve o foco do teclado para onde estava quando a busca abriu, e solta o foco deixado por um clique.
- No celular, fechar a busca mantinha a página escurecida até a busca sumir e depois passava para claro num só quadro; agora o escurecimento some junto.
- Os passos da importação do Tautulli diziam que o backup é baixado depois de **Backup Database**. O Tautulli o salva na própria máquina: os passos agora dizem onde (a pasta `backups` dele, `/config` com Docker), qual arquivo pegar e que é preciso copiá-lo.
- Uma busca fechada antes de terminar de abrir deixava o trilho que abre sem animação: dali em diante ele abria e fechava de uma vez.
- O trilho que abre continuava aberto depois de escolher uma página nele até um clique em outro lugar, e em alguns navegadores, ou com zoom, desenhava uma barra de rolagem sob os ícones. Agora ele abre só para o ponteiro e para o teclado, e não desenha barra de rolagem enquanto está fechado.
- Na página Segurança, um endereço que o banco de dados situa num país mas não no mapa conta para esse país, em vez de entrar entre os endereços sem lugar conhecido.

## [2.1.2] - 2026-10-01

### Added
- **Importar do Tautulli**: traga o seu histórico do Plex. Envie o backup do banco de dados do Tautulli (o `.db`, ou o `.zip` que o contém) em **Configurações → Importar**; as reproduções dele têm um lugar próprio no filtro *Registrado por* da Atividade.
- **Um painel de fios para dizer quem é quem**: usuários do Plex de um lado, usuários do Jellyfin do outro. Arraste um fio de cada usuário do Plex até quem ele é agora, ou clique num e depois no outro, ou use o teclado. Quem ficar sem fio não é importado, e duas contas do Plex podem ir para uma pessoa só.
- Filmes e episódios são associados à sua biblioteca pelo nome (um filme pelo título e ano, um episódio pela série e pelo número), e o que ainda não está no seu servidor é guardado e vinculado quando chegar. Uma sessão retomada é uma só reprodução; música não é importada.
- O backup contém os tokens de acesso do Plex: o FinStats nunca os lê, e remove o arquivo assim que você importa ou recomeça.
- **Mídia sem vínculo** em **Configurações → Dados**: todo título do Tautulli, Jellystat ou Streamystats que não corresponde a nada na sua biblioteca, com a origem e o lugar onde provavelmente já está. **Localizar** o vincula a partir dos melhores palpites ou de uma busca (um filme pode ser localizado como especial de uma série), e o FinStats lembra a escolha, então o mesmo histórico importado de novo se vincula sozinho. As escolhas fazem parte do backup.

### Performance
- **A retrospectiva abre de três a cinco vezes mais rápido**: os atores e diretores mais assistidos do ano eram calculados varrendo o ano inteiro de novo para cada pessoa que aparecia nele. Medido num histórico real, um ano passou de 0,29 s para 0,09 s e os últimos 12 meses, de 0,53 s para 0,12 s.

### Fixed
- **O histórico importado encontra títulos que têm outro nome**: uma reprodução também corresponde ao nome do título no idioma original, então uma série que o Plex ou o Tautulli conheciam como 오징어 게임 se vincula a Round 6, e *Im Westen nichts Neues* a *Nada de Novo no Front*. Os títulos originais da sua biblioteca são lidos na próxima varredura.
- Os nomes são comparados como a busca os compara (maiúsculas, acentos, pontuação, um ano escrito no nome, um “The” no início), e o ano de lançamento pode diferir em um, então “Star Wars: Episode I - The Phantom Menace” encontra “… – The Phantom Menace” e um Kingsman de 2014 encontra o de 2015. Um título conhecido por um nome totalmente diferente continua nunca sendo adivinhado.
- Uma importação ou restauração do Streamystats com falha era anunciada nas notificações pelo nome interno.
- O botão *Compartilhar o ano em cards* da retrospectiva e a posição de espectador ao lado dele não tinham espaço entre si.

## [2.1.1] - 2026-10-01

### Changed
- **A Minha lista aparece onde houver um título**: nos cards Em breve do painel, como em Pipeline → Em breve, e na página de um episódio ou de uma temporada, que põe a série na lista.
- Num pôster do painel, o botão da Minha lista aparece quando você aponta para o card ou chega até ele pelo teclado, em vez de cobrir o canto de todos os pôsteres; numa tela sensível ao toque ele está sempre lá.

### Stability
- Salvar uma conexão do Sonarr, Radarr ou Seerr enquanto alguém estava com **Configurações → Conexões de saída** aberto podia deixar os dois esperando um pelo outro para sempre.
- **Configurações que não podem ser lidas impedem o FinStats de iniciar**, citando a configuração com problema, em vez de redefinir em silêncio todas as configurações para o padrão.
- **Duas mudanças nas configurações ao mesmo tempo** (a página de configurações e o agendamento de uma tarefa, por exemplo) não desfazem mais uma à outra.
- Uma pausa, um salto ou uma troca de faixa vistos enquanto o banco de dados estava ocupado são gravados na passagem seguinte em vez de se perderem.
- Uma leitura da biblioteca não falha mais por causa de uma Minha lista com várias entradas que acabam sendo o mesmo título.

### Fixed
- **Um arquivo substituído não está “agora no servidor”**: trocar por uma versão melhor o arquivo de um filme que está na Minha lista de alguém não avisa mais que ele acabou de chegar.
- Uma leitura com falha dos plugins, das tarefas agendadas ou do armazenamento do Jellyfin não esvazia mais essa lista na página Servidor até a leitura seguinte.
- **Uma reprodução do Streamystats para onde parou em todas as páginas**: a curva de abandono de um título e o “concluído” de um episódio se guiavam pela duração dela, e a contavam como estimada.

## [2.1.0] - 2026-09-29 to 2026-09-30

Minha lista

### Added
- **Uma lista só sua**: ponha nela um filme ou uma série pela página do título, por Em breve, por Adicionados recentemente ou pela busca (Ctrl+Enter), e encontre tudo em **Minha lista** no seu próprio perfil.
- **Cada entrada diz em que pé está**: no servidor, “5 de 26 episódios”, pedido, “Chega sexta-feira”, fora do servidor, saiu da biblioteca, ou assistido. Isso é calculado toda vez que você olha, com a mesma noção de “visto” do progresso das suas séries. Títulos assistidos ficam, num grupo próprio, até você tirá-los.
- **Um título que ainda não está no seu servidor** pode ir para a lista direto do que o Sonarr ou o Radarr estão esperando. Ele é vinculado ao título sozinho quando chega, e um arquivo renomeado leva a entrada junto.
- **“Na Minha lista: já disponível”**: uma notificação para os seus próprios destinos quando chega algo que você estava esperando. Desligada até você marcá-la.
- **Ninguém mais vê a sua Minha lista**, nem um administrador, e a notificação dela vai só para os seus destinos. Nada é gravado no Jellyfin, no Seerr, no Sonarr ou no Radarr.
- As listas fazem parte do backup, e uma restauração as mescla sem duplicar nada.

### Changed
- As entradas de `/api/upcoming` trazem os ids do TVDB e do TMDB do título, e `/api/auth/me` diz se o servidor mantém a Minha lista.

## [2.0.4] - 2026-09-29

### Added
- **Agendamento de tarefas, do jeito do Jellyfin**: toda tarefa que o FinStats faz sozinho tem gatilhos (diariamente num horário, semanalmente num dia, num intervalo, na inicialização ou após a varredura de biblioteca do Jellyfin), cada um com um limite de tempo opcional. Clique numa tarefa para ver e mudar o agendamento; *Adicionar gatilho* abre o mesmo diálogo que o Jellyfin tem.
- **Tarefas é uma seção própria** nas Configurações, com a última execução de cada tarefa em palavras (“Rodou há 38 minutos, levou 6 minutos”) e um botão Rodar; antes ficava no topo de Sistema.
- **Alterações de metadados**: a cada hora o FinStats lê o que o Jellyfin salvou desde a última olhada (nomes, sinopses, gêneros, classificações, elenco e equipe, detalhes de arquivo e imagens), então uma edição no Jellyfin aparece sem esperar uma varredura de biblioteca.
- O backup e o banco de dados de geolocalização também podem ser rodados em Tarefas; Esc volta do agendamento de uma tarefa para a lista.

### Changed
- **O tema é um interruptor com três posições**, Claro · Dispositivo · Escuro, onde ficava o botão que alternava entre eles: clique num terço dele, arraste o cursor ou use as setas. O cursor traz o ícone da escolha, que entra animado quando ela muda.
- **Sair fica em Configurações → Conta**, que diz quem está conectado aqui; a barra lateral fica com o interruptor de tema.
- **As antigas configurações de agendamento agora são gatilhos**: *Seguir a varredura de biblioteca do Jellyfin* com o seu intervalo reserva, *Fazer backup a cada N dias* e o interruptor *Manter o banco de dados atualizado* do GeoIP viraram os gatilhos correspondentes nas suas tarefas, com os valores que você tinha. Coleta, Backups e Segurança agora levam ao agendamento.
- Uma inicialização não roda mais todas as tarefas de novo: o momento em que cada uma terminou pela última vez fica guardado.
- Uma tarefa do Jellyfin ainda sem estimativa mostra “faltam cerca de …”, com os pontos girando onde o tempo vai aparecer, em vez de “ETA…”.

### Performance
- **O mapa da Segurança não trava mais**: arrastar e dar zoom movem o mapa já desenhado e o redesenham uma vez quando o gesto termina, e o pulso do que está tocando não faz mais o mapa inteiro se repintar 120 vezes por segundo enquanto ninguém mexe nele. Um arrasto passou de 34 para 60 quadros por segundo num computador lento.

### Fixed
- **Um pôster ou um retrato substituído no Jellyfin aparece no FinStats**: as imagens ficavam em cache por uma semana no servidor e mais uma no navegador, independentemente do que o Jellyfin tivesse desde então. Agora ficam em cache sob a tag de imagem do Jellyfin, então uma imagem nova é um arquivo novo, e o navegador confere se a cópia dele está atual toda vez que mostra uma. Substituir um retrato salva de novo só a pessoa, por isso Alterações de metadados também lê as pessoas.
- **Servidor → Tarefas mostra uma estimativa assim que a página abre**: o tempo restante só era medido a partir das leituras que a própria página fazia enquanto estava aberta, então toda visita começava em “ETA…” até ter visto a porcentagem andar. Agora o FinStats cronometra sozinho uma tarefa do Jellyfin em andamento, a cada 10 segundos enquanto ela roda, e a página recebe a estimativa pronta. Uma tarefa que o Jellyfin informa em 0% o tempo todo (a primeira fase de uma varredura de biblioteca) mantém os pontos até andar: nada é adivinhado.
- **As dicas do mapa aparecem sobre o ponto**: no Firefox a dica do ponto do que está tocando podia ir parar longe, e onde os pontos se sobrepõem (alguém assistindo em casa) a dica exibida dependia de um pixel. Agora uma só dica cita tudo o que está sob o ponteiro.
- **A barra de status tem uma só forma de destaque**: todo link (os streams, Repo, a versão) acende como a mesma caixinha, e os pontos entre os itens ficam centralizados nos espaços. Passar o ponteiro sobre a versão acendia também o ponto e o espaço antes dela.
- **Um caminho de arquivo ocupa a largura toda do card**: no card Arquivo de um título, e nas pastas do Jellyfin na página Servidor, um caminho era espremido numa célula estreita e descia por nove linhas ao lado de espaço vazio.
- Um diálogo sumindo não engole mais um clique na página por baixo dele.

## [2.0.3] - 2026-09-28

### Security
- A página Servidor entregava o id de todo dispositivo a qualquer um com *Ver o servidor*; os ids exigem também *Ver detalhes de rede*, como em todo o resto.
- Dois interruptores de permissão acionados antes de o primeiro ser salvo enviavam a mesma lista inicial duas vezes, e o segundo desfazia o primeiro: uma permissão retirada continuava concedida e uma seção de perfil público desligada continuava publicada, enquanto os dois interruptores mostravam o que tinha sido escolhido.

### Stability
- **Só uma importação ou restauração por vez**: dois envios postados juntos passavam ambos pela verificação, e uma restauração nunca olhava as importações; o segundo esperava o banco de dados até desistir e falhava depois de minutos de envio.
- **Um endereço com um escape `%` quebrado abre a página de não encontrado**, em vez de deixar um indicador de carregamento girando para sempre.
- **Um número de página grande numa lista é uma página vazia**: ele estourava e voltava a ser a primeira página.

### Fixed
- **As tabelas ordenam números corretamente num navegador em alemão ou norueguês**: o “1,204” das tabelas (mil duzentos e quatro) era lido como número decimal, e qualquer coisa acima de um milhão, como texto.
- **A visualização em tabela de um gráfico ordena datas, dias da semana e posições pelo tempo**, e não pelas letras.
- **O histórico de downloads diz de onde vieram as importações**: o Sonarr e o Radarr citam o indexador e o protocolo só na captura, então esses detalhamentos ficavam sempre vazios. Um pacote de temporada com falha é uma falha só, e não uma por episódio.
- **Excluir uma reprodução assistida junto** recalcula os grupos daquele título, em vez de deixar um “grupo” de uma pessoa.
- **O tempo juntos de uma pessoa nunca passa do tempo que ela ficou**: a página Juntos lhe creditava a noite inteira.
- **Uma noite acompanhada que atravessa a meia-noite do Réveillon** (ou o limite de “últimos N dias”) pertence inteira à janela em que começou, em vez de virar duas noites sozinho.
- **Reproduções importadas de episódios recebem o nome do episódio** onde o Jellystat guardou só o da série.
- **A geolocalização encontra o banco de dados de cidades ao lado dos de país e de ASN** que o `geoipupdate` grava, em vez de se desligar.
- **O ponto mais retrocedido segue os filtros da página**, como as contagens.
- **“Seu ano está pronto”** vai só para pessoas cuja retrospectiva tem alguma coisa.
- **Resolver de novo um alerta silenciado mantém o par silenciado**; só reabrir desfaz um silenciamento.
- **Uma chave de API expirada não conta mais para o limite de vinte.**
- **O limite de sessões libera a sessão mais antiga** mesmo quando muitos logins chegam no mesmo segundo.
- **Um endereço de casa encontrado pela consulta continua lá** quando o mesmo endereço é retirado da lista digitada.
- **Fotos no log de auditoria e na lista de chaves**, onde antes havia só iniciais.
- **A lista de notificações de um administrador diz de quem é cada destino pessoal**, em vez de “Seu” em todos; **Enviadas recentemente** mostra todos os destinos do último evento.
- **Esc num título ou numa pessoa nunca sai do FinStats**, e fecha só o diálogo de cima.
- **Uma página é desenhada mesmo quando um salvamento cancela a pré-busca que ela esperava.**
- **Reproduzindo agora mostra um espectador pausado que arrasta a barra** onde ele está agora, e não onde estava quando apertou o play.
- **Remover uma conexão atualiza quais páginas existem**, como salvar uma já fazia.
- **A data de compilação do banco de dados de geolocalização mantém o ano** em inglês americano.

## [2.0.2] - 2026-09-28

### Security
- Atrás de um proxy, o limite de login contava o primeiro endereço de `X-Forwarded-For`, que é o que o cliente envia: um endereço inventado novo a cada tentativa, e o limite nunca se aplicava. Agora conta o último, o que o proxy escreveu.
- *Incluir endereços IP e lugares* enviava endereços a um destino pessoal cujo dono não tem *Ver detalhes de rede*, endereços que nenhuma página lhe mostraria. Os logins com falha de outra pessoa agora exigem também *Ver o servidor*, como na página Segurança.

### Stability
- **Uma parada nunca se perde**: quando uma leitura da biblioteca era recusada nos primeiros instantes após a inicialização, antes de o servidor estar escutando por isso, a recusa era anotada e o FinStats seguia em frente em vez de parar.
- **Um Jellyfin que responde num formato que o FinStats não consegue ler é recusado**, e não levado a sério: todos os usuários, ou todos os títulos de uma biblioteca, chegando sob chaves que ele não conhece (`id` em vez de `Id`) passavam pela proteção com a contagem completa e depois eram marcados como removidos.
- **O fim de uma reprodução nunca é descartado**: quando o banco de dados ficava ocupado além da espera (com uma importação segurando-o), uma reprodução que terminava nesse momento continuava marcada como tocando até a próxima reinicialização. Agora ela é gravada na passagem seguinte.
- **Configurações que uma restauração não consegue ler são deixadas como estão**: um backup cujas configurações não cabiam mais nesta versão redefinia todas as configurações na restauração: janela de mesclagem, permissões, endereços de casa, agendamento.
- **Um caractere de controle num nome ou num título não quebra mais os cards**: o card de perfil, todo card do ano que mostrava aquele título e o ZIP deles respondiam com erro.

### Fixed
- **As notificações param para quem foi desativado ou excluído no Jellyfin**: os destinos pessoais dessas pessoas continuavam recebendo tudo o que as permissões antigas permitiam.
- **Uma notificação na fila é verificada de novo antes de ser enviada**: um destino desligado, ou um dono que tinha perdido uma permissão, ainda recebia o que estava esperando nova tentativa.
- **Um destino que sempre responde `Retry-After` é abandonado** depois de cinco tentativas como qualquer outro, em vez de receber novas tentativas por trinta dias.
- **Ids de dispositivo na página de uma pessoa** só aparecem com *Ver detalhes de rede*, como em todo o resto.
- **Um ano publicado espera um dia para tudo**: as séries concluídas e abandonadas acompanhavam cada reprodução, então dar play numa série abandonada mudava a página pública em segundos.
- **Uma série pedida conta como assistida** quando um dos episódios conta: toda série pedida aparecia como nunca assistida, no capítulo Pedidos e no card dele.
- **O capítulo Juntos da retrospectiva deixa a TV ao vivo de fora**, como o resto do ano: um canal deixado ligado em companhia podia virar o título principal, e a porcentagem podia passar de 100%.
- **O card Dias de “os últimos 12 meses” termina no dia mais recente**: a grade deixava de fora as três ou quatro semanas mais recentes.
- **Os novos episódios de uma série são anunciados uma vez por noite**: episódios dos dois lados da meia-noite UTC eram anunciados duas vezes num servidor a leste ou a oeste dela.
- **Conectar o Seerr não anuncia pedidos antigos como prontos para assistir**: um pedido que já estava disponível na primeira leitura, sem outro registro de quando, era lido como se tivesse chegado naquele momento.
- **Os títulos de uma biblioteca excluída no Jellyfin vão junto com ela**: eles ficavam na busca, na prateleira Adicionados recentemente e nos totais.
- **Um remux continua sendo transmissão direta** quando uma leitura perde os detalhes de transcodificação por um instante (um salto), em vez de virar transcodificação pelo resto da reprodução.
- **“O Jellyfin está fora do ar” significa leituras que falharam em sequência**: timeouts avulsos com dias de diferença, num servidor ocioso no socket, somavam até chegar a isso.
- **Uma importação do Streamystats mostra o progresso no próprio card**, e não por cima do último resultado da importação do Jellystat.
- **Um endereço cujo host começa com “web”** (`http://web:8096`) é lido como host, e não cortado como se fosse o caminho do cliente web do Jellyfin.
- **Um erro da fila oculta o login inteiro** de um endereço mesmo quando a senha tem uma barra.

## [2.0.1] - 2026-09-28

### Changed
- **Os idiomas de um título não atrapalham mais**: o cabeçalho cita os três primeiros idiomas de áudio e de legenda, uma linha para cada, e um novo card **Idiomas** lista todos: os completos juntos, os parciais com quantos episódios os têm. As duas dúzias de idiomas de legenda de uma série longa empurravam a arte uma tela para baixo.

### Fixed
- **O pôster de um título fica ao lado do nome**: quando o texto ao lado era mais comprido que o pôster (uma sinopse longa, muitos estúdios), o pôster afundava até o pé do cabeçalho, embaixo de uma arte vazia.
- **Uma versão que o FinStats não consegue ler é chamada assim**: um `app_version` editado à mão para algo ilegível fazia o FinStats se recusar a iniciar como “o FinStats 2.0.1, mais antigo”, o que ele não era. Ele continua se recusando, e agora diz qual valor não conseguiu ler e como escrevê-lo.

## [2.0.0] - 2026-09-25 to 2026-09-28

Retrospectiva 2026, perfis públicos e assistir junto

### Added
- **Com quem você assistiu**, na retrospectiva: noites em companhia, o tempo passado nelas, o título que mais reuniu as pessoas e até três companhias, com nome no app e link para os perfis delas.
- **Séries vistas até o fim**, na retrospectiva: o último episódio de cada uma vista naquele ano, com a mesma noção de “visto” das barras de progresso do perfil, e também as começadas e deixadas para depois, vistas menos da metade e sem nada reproduzido há dois meses.
- **O que você pediu**, na retrospectiva: os pedidos do Seerr feitos no ano, quantos chegaram e quantos você assistiu depois. Só onde existem pedidos.
- **O ano comparado ao anterior**: horas, reproduções e dias, e quanto a mais ou a menos.
- **O ano como história**: cada capítulo como um card de 1080×1920, o formato que os stories do celular usam (o ano, os números, as séries, os filmes e as músicas favoritas, gêneros, a personalidade, horas e dias, cada dia do ano, recordes, companhia, séries concluídas, pedidos e o ano anterior). Salve um, ou todos num ZIP, em “Compartilhar o ano em cards”.
- **Um card nunca cita outra pessoa**: os cards são desenhados a partir de uma cópia do ano sem nenhum campo para outra pessoa, uma posição no ranking ou um app. No app, um card não traz nome nenhum.
- **O ano do servidor**, para administradores do Jellyfin: todos os títulos e totais e a personalidade da casa, sem ninguém ranqueado nem citado. Um interruptor na página da retrospectiva, `scope=server` na API.
- **“A retrospectiva do ano está pronta”**: uma notificação em dezembro, uma vez para cada pessoa que assistiu algo naquele ano.
- **Perfis públicos**: uma pessoa pode publicar parte do seu perfil num link que qualquer um abre sem conta (totais e títulos favoritos, sequências e quando assiste, o ano inteiro como sua história em cards, as dez últimas reproduções), cada parte desligada até o dono ligá-la em Configurações → Perfil público. Dispositivos, endereços, apps, métodos de reprodução, caminhos de arquivo e outras pessoas nunca são publicados, seja o que for que esteja ligado, e o nome exibido é um que o dono digita, nunca o login dele no Jellyfin.
- **Nada numa página publicada tem menos de um dia**, nem as reproduções recentes nem os totais, então uma página que alguém vigia nunca revela quem está assistindo agora.
- **Cards para compartilhar**: todo perfil publicado vem com uma imagem de 1200×630, do perfil e do ano, que o Discord, o Signal, o Mastodon e os demais mostram quando o link é colado, e que pode ser baixada para postar. Desenhada no servidor a partir do que a página mostra e nada mais.
- **Um link que você pode retirar**: 22 caracteres aleatórios, trocados com um clique; o link antigo passa a responder exatamente como um que nunca existiu.
- **Perfis públicos ficam desligados até um administrador permiti-los**: um interruptor para o servidor inteiro, uma lista de quem publica o quê e um jeito de tirar um perfil do ar. Publicações, alterações, trocas de link e retiradas ficam no log de auditoria, e uma chave não consegue tornar nada público.
- **Juntos**, uma página própria: quem assiste com quem, quanto, e como isso mudou. Tempo juntos, noites e a parcela do tempo assistido passada em companhia, cada um comparado ao período anterior; horas em companhia contra horas sozinho por dia ou semana; cada par de pessoas com as suas noites, o tempo juntos, quando assistiram pela última vez e o que mais assistem; os títulos mais assistidos juntos; as noites recentes; e, para quem pode ver todos, o tempo sozinho de cada pessoa contra o tempo em companhia.
- Todo par conta: uma noite de três conta para cada um dos seus três pares, onde o card antes contava só o conjunto exato de pessoas. O tempo juntos de um par é a menor das duas permanências.
- O card **Assistido junto** no painel e num perfil leva à página, e o card de um perfil diz que parcela do tempo assistido daquela pessoa foi em companhia.
- `GET /api/stats/groups` ganha `pairs`, `series`, `people`, `previous`, `bucket`, e `watch_s` e `share` nos totais; as chaves que já tinha não mudam.
- **Onde as pessoas param**, em toda página de filme e de episódio: a parcela das reproduções que ainda estão assistindo a cada minuto, de todos no começo até quem chegou ao fim, com os retrocessos e as ativações de legenda dessas reproduções marcados no mesmo eixo. O card diz quantas paradas foram medidas (reproduções que o FinStats registrou) e quantas estimadas (reproduções importadas, pela duração delas), e precisa de três reproduções antes de desenhar qualquer coisa.
- **Quem continua assistindo**, na página de uma série: quantas pessoas começaram cada episódio, em ordem, e uma coluna **Concluído** por episódio. “Todo mundo larga no episódio três” são pessoas que nunca dão play no episódio quatro.
- **Arquivos que nunca tocam**, em Reprodução: um filme ou episódio começado três vezes ou mais que nunca passou dos trinta segundos, com quem tentou e em quais apps. O log do Jellyfin não registra erros de reprodução, então as próprias reproduções são a testemunha; a duração mínima de reprodução não se aplica aqui, já que é exatamente o que essas reproduções nunca alcançam.
- **Mais retrocedidos** e **Legendas ativadas**, em Reprodução: títulos ranqueados pelos retrocessos por reprodução, com o minuto em que eles se concentram, e pela parcela de reproduções em que as legendas foram ativadas nos primeiros dez minutos. As três listas exigem *Ver a atividade de todos*. Retrocessos de menos de 20 segundos nunca foram registrados, então repetir rapidamente uma fala é invisível de propósito.
- Um salto guarda de onde veio como um número ao lado do rótulo (`from_s` na linha do tempo de uma reprodução). Todo salto já registrado recebe o número a partir do rótulo, e o mesmo vale para um backup de antes disso que for restaurado.
- `GET /api/stats/files` para as três listas; `GET /api/items/{id}` ganha `insights`, e os episódios de uma série ganham `users` e `finished`.
- **Chaves de API**: qualquer pessoa conectada pode criar chaves para si em Configurações → Chaves de API, para um script, um painel ou um celular. Uma chave é enviada como `Authorization: Bearer fs_…`, é mostrada uma vez, guardada como hash, pode ter validade e é revogada com um clique; um administrador vê e pode revogar as de todos. Ela corresponde à mesma pessoa, com as mesmas permissões, que um login, lidas ao vivo a cada requisição, então perder o direito de entrar desativa todas as chaves na hora. Uma chave não pode criar nem revogar chaves.
- **Um feed de calendário**: `GET /api/calendar.ics?key=…` é o que o Sonarr e o Radarr têm a caminho, como um iCalendar que um celular pode assinar. Uma chave com o escopo *calendar* abre o feed e nada mais; o feed lê a chave do endereço porque um calendário assinado não consegue enviar cabeçalho, nunca do cookie, e não cita ninguém.
- **Um log de auditoria** em Servidor → Auditoria, para administradores do Jellyfin: todo login e toda tentativa com falha com o endereço, toda configuração ou permissão alterada e para quê, toda chave criada, usada ou revogada, toda conexão, destino, backup e importação, toda reprodução excluída e todo alerta resolvido, com quem, de onde, por qual chave e se deu certo. Guardado por um ano, faz parte dos backups. `GET /api/audit` o percorre por páginas.
- **Importar do Streamystats**, uma seção própria nas Configurações ao lado do Jellystat. Pegue o arquivo de **Settings → Backup & Import → Download Backup** dele e solte aqui: anos de histórico chegam em uns dois segundos, e nada que você já tem é contado de novo. `finstats import-streamystats <file>` faz isso sem interface, `POST /api/import/streamystats` pela API; uma importação roda por vez, de qualquer tipo.
- **Usou os dois rastreadores? Importe os dois arquivos.** Uma reprodução é reconhecida como uma que o FinStats já tem pelo id que o próprio rastreador lhe deu e, na falta dele, pela mesma pessoa assistindo o mesmo item com qualquer uma das pontas da reprodução perto de uma do histórico, então uma exportação do Jellystat e uma do Streamystats das mesmas noites podem ser importadas, em qualquer ordem, e uma noite que o próprio FinStats acompanhou também não é importada por cima. “Perto” é a janela de mesclagem em Configurações → Coleta.
- Linhas importadas dizem o que não conseguem dizer. Uma reprodução que o próprio Streamystats acompanhou não guarda nada sobre o arquivo (nem codec, resolução, taxa de bits ou contêiner), então esses campos ficam vazios em vez de preenchidos com um palpite, e os cards que os mostrariam se escondem. Uma reprodução que ele tinha trazido do Jellystat carrega a sessão inteira, e é ela que se lê: datada pelo momento em que terminou, o único que os dois rastreadores guardaram, e descrita como transcodificação só quando a sessão realmente foi uma.
- Linhas que o Streamystats escreveu porque o Jellyfin informou um item como assistido, para uma sessão que ele nunca viu, são contadas e puladas: elas têm a duração do filme inteiro e virariam tempo assistido numa noite em que ninguém assistiu. O resultado diz quantas eram.
- Como os dados do Streamystats são interpretados, campo a campo, aprendido com uma exportação real e não com documentação: `docs/streamystats-import.md`.
- **De qual rastreador veio cada reprodução**, como filtro na Atividade: o que o próprio FinStats registrou, o que veio do Jellystat, o que veio do Streamystats. Só são oferecidos os que o seu histórico realmente tem, e o filtro respeita o escopo como todas as outras listas, então quem só pode ver as próprias reproduções só fica sabendo de onde veio o próprio histórico.
- **Os filtros da Atividade são listas suspensas que você marca**, então dá para pedir filmes *e* episódios sem música, dois métodos de reprodução ou duas pessoas ao mesmo tempo, em vez de um de cada ou todos. Tipo de mídia ganhou **Outros** agora que pode ser combinado.
- Várias pessoas ao mesmo tempo, em todo lugar onde aparece o filtro de usuário (o painel, a reprodução, as bibliotecas, a segurança e os demais), com o mapa e todas as estatísticas acompanhando. Um filtro de usuário continua só podendo restringir o que alguém pode ver: sem *Ver a atividade de todos*, uma requisição fica presa a quem a fez, seja quem for e quantos forem os que o endereço citar.
- **Um tema claro**: papel washi, com texto em tinta e um destaque vermelhão. O FinStats segue a configuração clara ou escura do dispositivo até você escolher um.
- **Um botão de tema ao lado de Sair** alterna entre Dispositivo, Claro e Escuro. A escolha fica guardada neste navegador e é aplicada antes de a página ser desenhada, então o outro tema nunca pisca na tela.
- **Abrir no Jellyfin**: toda página de filme, série e episódio tem um botão que abre a página daquele título no Jellyfin, numa nova aba.
- **Endereço do Jellyfin para as pessoas** em Configurações → Jellyfin: para onde esses botões apontam, para quando o endereço a que o FinStats se conecta não é um que o navegador alcança. Vazio mantém o endereço a que o FinStats se conecta; só um administrador do Jellyfin pode defini-lo.

### Changed
- **O FinStats tem um novo endereço**: `github.com/finstats/finstats`, e a imagem é `ghcr.io/finstats/finstats`. Troque o nome da imagem no seu `docker run` ou arquivo compose para continuar recebendo atualizações; a pasta de dados continua como está.
- **As Configurações mostram uma seção por vez.** A página tinha catorze cards numa coluna e seiscentas palavras de ajuda. Agora cada seção tem o seu próprio endereço (`/settings/collection`), uma lista à esquerda leva de uma a outra, e no celular a lista vira uma fileira de chips. Links para as âncoras antigas continuam funcionando.
- **Uma configuração é uma linha**: o que ela é, uma linha com a regra, o controle à direita. Os números de uma seção são salvos juntos com um botão embaixo; os interruptores continuam se salvando sozinhos. Os cards de status (Tarefas, Conexões de saída, Banco de dados) e o link das licenças ficam em **Sistema**, e os dois importadores têm uma seção própria.
- **Encontrar uma configuração** pela caixa no cabeçalho (`/` põe o foco nela) por qualquer palavra do nome; ela diz em que seção a configuração fica, e Enter leva à linha.
- **As Configurações são para todos que estão conectados**: as seções que administram o servidor continuam com quem pode administrá-lo, e uma pessoa que só tinha permissão para notificações finalmente consegue chegar aos próprios destinos.
- **A página Servidor mostra uma seção por vez**, como as Configurações: Visão geral (sistema e armazenamento), Tarefas, Dispositivos, Plugins, Log e Auditoria, cada uma no seu próprio endereço. Links para `/server#jobs` continuam funcionando.
- **O log do servidor é uma seção da página Servidor** (`/server/log`) em vez de uma página própria no menu; `/events` redireciona para lá com os filtros.
- Uma notificação sobre uma tarefa com falha, um backup com falha ou uma conexão que parou de responder leva à seção das configurações que cuida disso, em vez do topo da página.
- O que a sua biblioteca sabe dizer sobre uma reprodução importada (que tipo de coisa era, qual episódio, a duração, a que biblioteca pertence) é preenchido depois de **toda** leitura da biblioteca, e não só no momento de uma importação. O histórico costuma ser importado antes de o FinStats ter lido a biblioteca alguma vez, então essas reproduções ficavam para sempre adivinhadas ou desconhecidas.
- Restaurar um backup do FinStats não acrescenta mais uma reprodução que chegou de um rastreador com outro id, e continua mantendo duas sessões genuínas da mesma coisa com minutos de diferença.
- Os detalhes de uma reprodução citam o rastreador que a trouxe, em vez de chamar tudo o que foi importado de reprodução do Jellystat.

### Performance
- **Reiniciar é quase instantâneo com qualquer histórico.** Toda inicialização calculava quem assistiu junto em todo o histórico e decidia de novo quais reproduções eram locais, ambos para não mudar nada: 24 s e 5,5 s com dez milhões de reproduções. Agora uma inicialização faz só o que a execução anterior deixou pendente, e tudo de novo só depois de uma atualização ou quando a configuração por trás mudou.
- **Uma multidão chegando de uma vez é registrada em segundos.** Cada nova reprodução, cada progresso salvo e cada dispositivo eram gravados um a um, e cada reprodução encerrada fechava com uma olhada própria no histórico inteiro do título; agora uma passagem grava cada tipo de uma vez. 100.000 reproduções começando juntas levam 3 s, onde 25.000 terminando juntas levavam mais de quatro minutos.
- **Reproduzindo agora continua rápido com milhares assistindo.** Descobrir quem assiste junto comparava cada stream com todos os outros e listava cada companhia pelo nome: quarenta segundos com 10.000 streams, além de gigabytes de memória. Agora os streams são comparados dentro de um título, e cada um mostra alguns nomes e quantos outros há.
- Calcular quem assistiu junto regravava toda reprodução agrupada do histórico cada vez que o FinStats iniciava, importava ou via uma reprodução terminar; agora grava só o que mudou. Descobrir quais reproduções tinham terminado não fica mais lento com o quadrado do número de reproduções em andamento.
- **Reconhecer uma reprodução que o histórico já tem passa por um índice**, então importar 150.000 reproduções leva segundos, e revincular depois de uma leitura da biblioteca percorre os títulos em vez de ler todas as reproduções segurando o banco de dados.
- **Os insights de reprodução e a visão geral ficam mais rápidos em todo o período**: as partes independentes são calculadas lado a lado, o gráfico de simultaneidade consulta o dia local uma vez por quarto de hora em vez de uma vez por reprodução, e os gêneros são contados por título. Com um milhão de reproduções: de 4,1 s para 2,4 s, e de 1,6 s para 1,1 s.

### Stability
- **Uma biblioteca de TV não é mais lida como uma biblioteca esvaziada.** Ler uma biblioteca exige duas passadas (todos os itens, depois uma menor para o elenco e a equipe só de filmes e séries), e a verificação que decide se uma leitura é confiável para remover o que não viu olhava a segunda. Numa biblioteca de TV ou de música as duas contagens ficam muito longe uma da outra, então a verificação recusava toda leitura e parava o FinStats, e a instalação reiniciava e fazia o mesmo de novo. Nada foi removido por engano, mas nenhuma leitura da biblioteca conseguia terminar. Se a sua biblioteca de Séries ou de Música não estava pegando títulos novos, é por isso.
- Uma importação podia falhar com “database is locked” quando outra gravação (uma reprodução sendo atualizada) caía entre a primeira leitura e a primeira gravação dela. Os dois importadores pegam o bloqueio de gravação logo no começo, e o journal do banco de dados não fica mais do tamanho da maior importação já feita.
- **Um arquivo de banco de dados danificado, truncado, de outro programa ou somente leitura é recusado na hora**, citando o arquivo e dizendo que nada foi alterado, em vez de 30 segundos de silêncio; o banco de dados de outro programa nunca recebe gravações.
- Um backup ou snapshot de pré-atualização interrompido no meio não deixa mais um arquivo para trás, que podia tomar o lugar de um snapshot bom entre os três mantidos.
- **Sobrescrever o banco de dados de lugares com o FinStats rodando não o derruba mais**: o FinStats lê de uma cópia privada do arquivo.
- Um app do Jellyfin que informava um id de dispositivo novo a cada vez podia fazer a memória do FinStats crescer enquanto ele rodasse; dispositivos não vistos por cinco minutos agora são esquecidos.
- Uma pessoa mantém no máximo 30 sessões conectadas; a mais antiga é desconectada no login seguinte.

### Fixed
- **A mesma noite não é mais contada duas vezes quando um item foi renomeado.** Revincular o histórico a um título que foi adicionado de novo no Jellyfin podia transformar uma reprodução importada de um rastreador em duplicata de outra que já estava lá, porque a verificação que a pegaria rodava antes de o id mudar. Agora ela roda de novo onde quer que ids sejam revinculados. Só reproduções importadas são removidas, nunca uma que o próprio FinStats registrou, e quantas saíram fica no log. A primeira inicialização da 2.0 varre o histórico inteiro uma vez atrás das que já existem.
- **Uma sessão agora acompanha a pessoa como o Jellyfin a tem agora.** Um administrador rebaixado no Jellyfin mantinha acesso total ao FinStats pelo resto da sessão de 30 dias, e alguém desativado ou excluído no Jellyfin continuava conectado. Agora eles perdem o acesso assim que o FinStats lê os usuários do Jellyfin (a cada 15 minutos, ou na hora pelas Configurações), como sempre aconteceu com as chaves de API.
- Entrar grava o que o Jellyfin diz sobre a pessoa na lista de usuários do FinStats, então um backup importado de um rastreador não consegue mais tornar alguém administrador antes que a primeira leitura dos usuários corrija isso.
- Alguém excluído no Jellyfin é marcado como removido mesmo quando duas leituras dos usuários caem no mesmo segundo.
- **Erros de download nunca mostram a senha de um cliente de download ou a chave de API de um indexador**, que o Sonarr e o Radarr podem citar neles.
- Reproduções revinculadas a um título renomeado durante uma leitura da biblioteca não eram verificadas para assistir junto até a próxima reinicialização; agora são, na hora.

## [1.6.5] - 2026-09-24

### Added
- **Um backup completo é feito automaticamente antes de toda atualização.** Na primeira vez que um FinStats mais novo abre o seu banco de dados, ele grava uma cópia completa (o seu histórico inteiro, configurações e tudo) em `data/pre-update-backups/` *antes* de mudar qualquer coisa. Se uma atualização um dia quebrar algo, os seus dados estão seguros: pare o FinStats, ponha a cópia no lugar de `finstats.db`, inicie a versão que você usava e relate o bug. Nada se perde. As cópias mais recentes são mantidas; defina `FINSTATS_SKIP_PREUPDATE_BACKUP=1` para desligar isso se o espaço em disco estiver apertado.

## [1.6.4] - 2026-09-24

### Added
- **Mais cinco lugares para enviar notificações**: Telegram, Slack, Pushover, Pushbullet e e-mail, ao lado dos destinos webhook, Discord, ntfy e Gotify que já existiam. Cada um pede só o que precisa (um chat id, uma chave de usuário, uma caixa de correio), e os três que ficam num serviço só (Telegram, Pushover, Pushbullet) só podem ser alcançados ali, então um token nunca pode ser postado num host imitador. O e-mail sai por TLS desde o primeiro byte ou por `STARTTLS`, e não há terceira opção.
- **Licenças**, em **Configurações**: a GPL do próprio FinStats e toda licença de terceiros em que ele se apoia (253 crates Rust, as duas fontes incluídas, os contornos do mapa e o banco de dados de geolocalização), cada texto completo, como o projeto dele o escreveu. Escolha um componente e a licença abre na janela ao lado da lista; um crate que traz vários arquivos alterna entre eles ali mesmo. A lista é gerada a partir dos crates de que o FinStats é realmente feito, então nenhuma dependência pode ser adicionada sem que a licença seja registrada.

## [1.6.3] - 2026-09-24

### Fixed
- A proteção agora também entra em ação quando o Jellyfin não lista *nenhuma* biblioteca ou *nenhum* usuário (uma leitura quebrada, nunca um dia normal), o que o ajuste da 1.6.2 tinha deixado passar. Perder algumas bibliotecas continua sendo tratado como normal; perder todas de uma vez, não.

## [1.6.2] - 2026-09-24

### Fixed
- A proteção de integridade dos dados adicionada na 1.6.1 era zelosa demais: uma biblioteca que de fato perdia a maior parte dos itens (um punhado de clipes cujos arquivos sumiram, por exemplo) era confundida com uma leitura quebrada, então o FinStats se recusava a atualizá-la e parava. Agora ela só entra em ação numa leitura *claramente* quebrada: uma biblioteca grande o bastante para importar que volta totalmente vazia, ou uma grande reduzida a quase nada. As mudanças do dia a dia valem como antes, e o caso “o Jellyfin não devolveu nada” continua sendo pego.

## [1.6.1] - 2026-09-24

### Added
- **O FinStats não confia mais numa leitura do Jellyfin que apagaria o seu histórico.** Se uma biblioteca para a qual o FinStats guarda milhares de itens de repente volta vazia ou quase (uma atualização do Jellyfin que mudou a API, um erro disfarçado de resultado vazio), o FinStats mantém os dados que tem, marca a sincronização como falha (e notifica) e para de forma limpa com uma mensagem clara, em vez de marcar tudo como removido. Reinicie quando o Jellyfin voltar ao normal, ou defina `FINSTATS_ALLOW_LIBRARY_SHRINK=1` se você realmente esvaziou uma biblioteca. A mesma proteção cobre a lista de bibliotecas e a lista de usuários.

### Security
- Um cabeçalho `X-Forwarded-Host` enviado por um cliente podia avalizar um `Origin` alheio e fazer uma gravação entre sites passar pela proteção de mesma origem. Agora esse cabeçalho só é aceito atrás de um proxy confiável (`FINSTATS_TRUST_PROXY`), a mesma regra que a consulta do endereço do cliente já usa. O cookie de sessão `SameSite=Lax` era, e continua sendo, a primeira linha de defesa.
- Um destino de notificação pessoal podia apontar para `0.0.0.0` ou `::`, que chegam a esta máquina. A verificação que mantém os destinos pessoais apontados para endereços públicos agora os conta como locais, ao lado do loopback e das faixas privadas.

### Fixed
- A página **Reprodução** não deixa mais de carregar para uma biblioteca que contém uma reprodução sem método registrado (uma linha restaurada de um backup anterior à coluna); essa reprodução simplesmente conta como reprodução direta.
- Uma palavra muito longa numa busca ou num filtro não é mais um erro do servidor: o SQLite recusa um padrão `LIKE` com mais de 50.000 caracteres, então as palavras do filtro são cortadas num tamanho razoável antes da consulta.
- A página **Configurações** não rola mais para os lados no celular quando um card devolve uma frase inteira, como o resultado de uma restauração.

## [1.6.0] - 2026-09-24

Notificações

### Added
- Card **Notificações** nas Configurações: adicione um destino, marque sobre o que ele deve ser avisado, aperte Testar. Sem destino, nenhuma requisição é feita a lugar nenhum.
- Quatro tipos de destino: um **webhook** simples (JSON, documentado em `docs/api.md`, cabeçalho `Authorization` opcional), **Discord** (um embed pela URL de webhook de um canal), **ntfy** (ntfy.sh ou o seu próprio, para um tópico) e **Gotify** (o seu próprio servidor, com um token de aplicativo). Os títulos vão na mensagem, e não num cabeçalho.
- Onze tipos de evento, cada um com a sua marcação: viagem impossível, uma conta vista num país novo, uma rajada de logins com falha, uma tarefa que falhou, um backup que não pôde ser gravado, algo que parou de responder (Jellyfin, Sonarr, Radarr ou Seerr) e quando volta a responder, títulos novos na biblioteca (agrupados por série e por dia, como a prateleira do painel os agrupa), um pedido que ficou disponível, e uma reprodução começando ou terminando. Os dois últimos vêm desligados.
- Um pedido que ficou disponível cita o título, quem pediu e quanto tempo esperou.
- Destinos pessoais: conceda **Receber notificações** e a pessoa pode adicionar um destino próprio, que recebe só o que ela já pode ver: os próprios pedidos e alertas sempre, os de outras pessoas com *Ver a atividade de todos*, endereços com *Ver detalhes de rede*. Um destino pessoal precisa apontar para um endereço público; só um administrador pode apontar um para dentro da rede.
- **Incluir endereços IP e lugares**, por destino e desligado por padrão: sem isso, uma mensagem sobre viagem impossível cita só os lugares (“Oslo, Noruega e Londres, Reino Unido”).
- **Enviadas recentemente**, no mesmo card: o que foi dito, para qual destino, e o que o outro lado respondeu quando não chegou. Uma mensagem com falha é tentada de novo quatro vezes ao longo de mais ou menos uma hora, e um destino que está limitando a taxa é aguardado.
- **O endereço do FinStats**, uma configuração para administradores, é a base do link de uma notificação. Vazio significa que as mensagens não levam link.
- As próprias tarefas agendadas do Jellyfin na página Servidor: o que cada uma faz, em palavras e não pelo nome do código, e se é das pesadas. Uma tarefa em execução mostra uma barra de progresso, uma porcentagem e quanto falta; as outras mostram quando rodam de novo, quando rodaram por último e se deu certo. As tarefas que o Jellyfin esconde do próprio painel estão incluídas, e nada no FinStats pode iniciar ou parar uma tarefa.
- Quanto falta de uma tarefa é medido pelo ritmo em que a porcentagem realmente andou, comparado com a leitura mais recente que seja antiga o bastante para dizer alguma coisa, e não pela média da execução inteira, então uma tarefa que acelera ou desacelera é descrita pelo ritmo que tem agora.
- As listas de tarefas que o FinStats já lê por outros motivos (a verificação da varredura de biblioteca a cada 5 minutos, os detalhes do servidor a cada 15) alimentam a mesma medição, então uma tarefa rodando há uma hora costuma ter uma estimativa no momento em que a página abre. Nada a mais é pedido ao Jellyfin para isso.
- Uma tarefa em execução sem nada medido ainda mostra **ETA** e reticências girando em vez de um número, e uma cuja porcentagem não andou por meio minuto mostra “68% há 4 min” ao lado. Nada toma o lugar de uma estimativa que não foi medida.
- Uma lista de todas as versões ao lado das notas da versão: um clique leva a qualquer versão, ela marca a que está sendo lida, e numa tela estreita vira uma fileira de séries acima das notas.

### Changed
- As notas da versão não se recolhem mais. Toda série fica aberta e a lista de versões é como você navega, em vez de um grupo aberto por vez e um clique para chegar a qualquer outro.
- **Configurações → Conexões de saída** lista todos os destinos de notificação ao lado do Jellyfin e das conexões de serviços: o que é, o host, se está ligado e quando recebeu uma mensagem pela última vez.
- Nada muda na coleta, no histórico ou em qualquer número na tela. O FinStats continua só *lendo* do Jellyfin, Sonarr, Radarr e Seerr.

## [1.5.2] - 2026-09-22

### Changed
- Uma leitura da lista de sessões do Jellyfin e a reinicialização do relógio que a pediu são uma só chamada, então as duas não podem mais se separar. Esse era o formato da falha de requisições em disparada encontrada durante a construção da 1.5.0.
- Provar isso leva uma fração de segundo em vez de sete minutos contra um servidor real, então é verificado toda vez que algo muda. As mesmas requisições, o mesmo histórico, os mesmos números na tela.

## [1.5.1] - 2026-09-22

### Fixed
- Uma transcodificação é registrada uma vez, e não uma vez por segundo. Quando uma reprodução começa transcodificando e depois volta para reprodução direta, o Jellyfin deixa os detalhes da transcodificação na sessão; cada leitura era comparada com um valor que o FinStats tinha acabado de sobrescrever, então toda leitura parecia uma mudança e a página Atividade se enchia de linhas “Transcodificação” com “Reprodução direta: …” embaixo.
- As linhas já gravadas são limpas na inicialização: onde uma sequência delas diz a mesma coisa, fica a primeira. Uma reprodução que precisou de transcodificação continua contando como transcodificação.

## [1.5.0] - 2026-09-22

Acompanhamento de sessões por WebSocket

### Changed
- O FinStats mantém uma conexão aberta com o Jellyfin desde o momento em que inicia, e é por ela que fica sabendo de uma reprodução. A 1.4.0 trouxe isso como configuração, desligada por padrão, para se provar contra servidores reais; agora está sempre ligada. As mesmas reproduções, pausas e avanços, calculados pelo mesmo código.
- Nada tocando: o FinStats não pergunta nada ao Jellyfin. Uma noite em que ninguém assiste não custa nenhuma requisição.
- Algo tocando: a lista de sessões é lida uma vez por segundo. É daí que uma pausa, um salto ou uma troca de faixa tira a sua precisão, e é o que encerra uma reprodução cujo cliente sumiu sem se despedir.
- Tudo o que está carregado pausado por três leituras seguidas: as perguntas param e a conexão assume, onde antes um filme pausado custava a mesma pergunta a cada segundo, cerca de 16 MB por hora. Uma retomada, ou outra pessoa começando algo, é respondida em mais ou menos um segundo, e o tempo pausado continua nunca contando como tempo assistido.
- Toda leitura de saída pede respostas compactadas e as descompacta (Jellyfin, Sonarr, Radarr e Seerr compactam quando pedido), o que tira mais da metade das duas maiores leituras: a lista de sessões durante uma reprodução e a leitura da biblioteca. O banco de dados de geolocalização continua sendo baixado do jeito que é armazenado.
- O card Jellyfin nas Configurações diz qual das duas metades está rodando, e o ponto ao vivo segue se a conexão está trazendo dados, e não em qual metade você está.
- Só uma das duas roda por vez: enquanto o FinStats está perguntando, ele diz ao Jellyfin para parar de enviar, e pede que volte a enviar no momento em que a última reprodução termina. A conexão fica aberta o tempo todo.
- Um Jellyfin que responde à assinatura com nada é levado a sério enquanto responde aos keep-alives: o FinStats avisa uma vez, lê a lista de sessões por conta própria enquanto isso, e continua assinado até o primeiro envio resolver a questão. “Não consegue enviar” e “estava reiniciando quando perguntamos” parecem iguais, então o silêncio nunca é um veredito.
- A alternativa continua intacta: uma conexão que fecha, fica em silêncio ou acaba não falando esse protocolo volta a perguntar num temporizador já na passagem seguinte e continua tentando reconectar, nos intervalos de **Enquanto alguém assiste, verificar a cada** e **Enquanto nada toca, verificar a cada**.

### Added
- O que o coletor está fazendo, em `GET /api/status`: em qual das quatro situações ele está (`idle_socket`, `playing_poll`, `paused_socket`, `fallback`), se a conexão está aberta e assinada, com que frequência ele está perguntando, quantas leituras de sessão saíram no último minuto e quando isso mudou pela última vez. Respondido da memória (sem requisição, sem consulta), e sem nomes, títulos ou contagens.
- Esses campos são conferidos uns contra os outros a cada passagem, e uma divergência que persiste é gravada no log.
- Um limitador sob toda leitura de sessão no processo: no máximo 2 por segundo e 70 por minuto, com o excesso adiado, contado e registrado no log no máximo uma vez por minuto. O uso normal fica bem abaixo disso.

### Fixed
- As atualizações ao vivo agora substituem de verdade as consultas periódicas. Ligada, a 1.4.0 ainda pedia a lista de sessões a cada cinco segundos e reabria a conexão a cada vinte a quarenta, porque julgava a conexão pela última vez que uma lista tinha chegado, e o Jellyfin envia uma quando algo *muda* e nada entre uma mudança e outra. Agora estar vivo significa o Jellyfin responder qualquer coisa, verificado com um keep-alive cerca de duas vezes por minuto.
- Uma lista de sessões que chegava enquanto o FinStats estava entre duas passagens era jogada fora, supondo que outra viria um segundo e meio depois. Agora ela é guardada e usada.
- **Última verificação** nas Configurações, e o card Conexões de saída, não ficam mais desatualizados enquanto a conexão está em silêncio: o que mostram é a resposta do Jellyfin a um keep-alive.

### Removed
- A configuração **Deixar o Jellyfin enviar o que está tocando** em Configurações → Coleta, e `live_socket` da API de configurações.
- `socket_enabled` do status do coletor: não existe mais “desligado”.
- `last_reconcile_at` do status do coletor, substituído por `socket_live`.

## [1.4.1] - 2026-09-22

### Changed
- O Seerr só é lido por inteiro quando algo mudou: a cada cinco minutos o FinStats pede uma linha, o pedido alterado mais recentemente, e a passagem termina ali se ele já o conhece. Isso dá cerca de um kilobyte contra uns sessenta de uma página de cinquenta. Pedidos ainda a caminho são verificados a cada quinze minutos, tudo é listado uma vez por dia, e um pedido novo continua aparecendo em até cinco minutos.
- As filas do Sonarr e do Radarr são lidas a cada cinco segundos enquanto uma página as mostra, a cada minuto enquanto há algo na fila e a cada cinco minutos enquanto ela está vazia, em vez de a cada minuto o dia inteiro. Abrir a página, conectar um serviço ou um pedido novo no Seerr as atualiza na hora.

## [1.4.0] - 2026-09-21

Sessões por WebSocket opcionais, e todo lugar que o FinStats alcança

### Added
- **Deixar o Jellyfin enviar o que está tocando** em Configurações → Coleta, desligado até você ligar: uma conexão traz a lista de sessões, e cerca de 17.000 requisições por dia viram quase nenhuma. As reproduções, pausas, avanços e grupos são calculados exatamente pelo mesmo código de antes. Enquanto está ativo, o FinStats ainda faz uma requisição comum por minuto *enquanto algo está tocando*, e nenhuma quando nada está. Uma conexão que fecha, fica em silêncio por quinze segundos ou acaba não falando esse protocolo volta para o temporizador na passagem seguinte e continua tentando reconectar.
- **Conexões de saída**, um card nas Configurações para administradores do Jellyfin: todo destino que o FinStats pode alcançar (o seu Jellyfin, o serviço “qual é o meu IP”, o banco de dados da DB-IP, e cada Sonarr, Radarr ou Seerr), com para que serve, se está ligado e quando respondeu pela última vez. Montado com o que o FinStats já sabe, e mostra só endereços: nunca uma chave, nunca um caminho base.
- **Consultar agora** em Configurações → Rede doméstica, para o dia em que o seu endereço público mudar.

### Changed
- A consulta “qual é o meu IP” roda uma vez, na primeira vez que o FinStats precisa de um endereço, em vez de a cada quinze minutos.
- O card Jellyfin nas Configurações diz como o coletor está sendo informado e, quando a conexão está ligada mas não traz dados, por quê.

## [1.3.0] - 2026-09-20

Sonarr, Radarr e Seerr, numa página só

### Added
- **Conexões** nas Configurações, para administradores do Jellyfin: **Sonarr**, **Radarr** e **Seerr** (ou Jellyseerr/Overseerr), vários de cada tipo, cada um testado antes de ser salvo. O seu cliente de download não precisa de configuração própria, porque o Sonarr e o Radarr já conversam com ele e o FinStats lê o que eles sabem.
- **Pedidos**: quem pediu o quê, quanto tempo levou para chegar e se a pessoa chegou a assistir. Blocos para os títulos em espera e a espera típica, um gráfico de como isso mudou mês a mês, e listas do que chegou há semanas e nunca foi reproduzido, e de quem mais pede.
- **Em breve**: novos episódios e lançamentos de filmes do Sonarr e do Radarr como uma agenda por dia, marcados com se *você* assiste àquela série e, para quem pode ver todos, quem mais assiste. Uma série conta como assistida quando alguém reproduziu um episódio dela nos últimos quatro meses. Uma fileira “Em breve” no painel, uma por perfil para as séries que aquela pessoa assiste, e o que vem a seguir para um título na própria página dele.
- **Downloads**: a fila ao vivo de todo Sonarr e Radarr (o que é, quanto já foi, a que velocidade, o que deu errado na importação), com a pessoa que pediu ao lado. Um pacote de temporada é uma linha só, por mais episódios que tenha, e o que precisa de atenção fica no topo. Abaixo, o que chegou numa semana, num mês ou num ano por indexador, qualidade e cliente de download, e quais downloads falharam. O painel mostra a mesma lista, resumida.
- **Ver o que está baixando**, uma permissão nova. Sem ela, as pessoas ainda veem até onde o *próprio* pedido chegou e quanto falta, e nada mais: nem nomes de releases, nem velocidades, nem outros downloads. Os pedidos de outras pessoas exigem *Ver a atividade de todos*.

### Changed
- O FinStats envia a esses serviços `GET` e nada mais: não há nele código capaz de aprovar um pedido, iniciar uma busca, ou adicionar, pausar ou remover um download.
- As chaves de API deles ficam guardadas no próprio banco de dados do FinStats, nunca são devolvidas ao navegador, nunca são gravadas num log e nunca fazem parte de um backup. Nenhum redirecionamento é seguido, então uma chave não pode ir para um lugar que você não informou, e os certificados são verificados a menos que você desligue isso para uma conexão.

## [1.2.2] - 2026-09-20

### Added
- Idiomas de áudio e de legenda em todo filme e episódio (“Áudio: japonês · inglês”).
- Até onde vai uma dublagem: uma série ou temporada diz quantos episódios têm cada idioma (“Inglês: 13 de 26 episódios”), e a lista de episódios tem uma coluna **Áudio**.
- **Idiomas de áudio** e **Idiomas de legenda** nas páginas de biblioteca: quantos arquivos podem ser reproduzidos em cada um.
- O FinStats relê a biblioteca uma vez depois desta atualização para pegar os idiomas. Ele nunca diz “dublado”, porque o Jellyfin não informa o idioma original de um título, e uma faixa sem etiqueta de idioma aparece como “Desconhecido”.

## [1.2.1] - 2026-09-20

### Changed
- As imagens do README foram refeitas nesta versão, com uma nova da página **Segurança**. Nada no app muda.

## [1.2.0] - 2026-09-20

De onde as pessoas assistem, num mapa

### Added
- **Página Segurança**, para quem pode ver detalhes de rede e a atividade de todos: um mapa-múndi com um ponto para cada lugar de onde os seus usuários assistem, do tamanho do quanto acontece ali. Casa é verde, lugares fora de casa são roxos, streams ao vivo pulsam e logins com falha de fora são vermelhos. Arraste para mover, dê zoom com os botões, as teclas ou Ctrl + roda do mouse, e pontos sobrepostos se juntam até você aproximar. Abaixo, cada lugar com as suas pessoas, reproduções, tempo assistido e logins, e os países por reproduções.
- **Viagem impossível**: um alerta quando uma conta é vista em dois lugares que nenhum voo liga, ou em dois lugares distantes ao mesmo tempo, com os dois registros, a distância, o tempo entre eles e a velocidade que isso exigiria. **Mostrar no mapa** desenha o trajeto.
- **País novo**: a primeira vez que alguém reproduz algo ou entra a partir de um país em que nunca foi visto.
- Resolva um alerta com uma nota, ou silencie um par de lugares para aquela pessoa, para uma VPN ou um celular cuja operadora fica na capital. Alertas encontrados em histórico antigo, depois de uma importação ou num banco de dados novo, são arquivados como resolvidos em vez de inundar a lista. Os alertas e o que você decidiu sobre eles fazem parte dos backups.
- Os lugares vêm de um banco de dados de cidades lido localmente: nenhum endereço é enviado a lugar nenhum, e o mapa é desenhado a partir de contornos incluídos no FinStats, e não de um serviço de mapas. Baixe o banco de dados gratuito da DB-IP pela página Segurança ou por **Configurações → Segurança**, deixe o FinStats atualizá-lo todo mês (desligado até você ligar), ou ponha o seu próprio `.mmdb` (DB-IP, MaxMind GeoLite2-City) em `data/geoip/`. `FINSTATS_GEOIP_DB` indica um arquivo em outro lugar.
- **Configurações → Segurança**: o banco de dados em uso, a atualização mensal, e quão rápido (900 km/h) e quão longe (500 km) dois registros precisam estar para contar como viagem impossível. Bancos de dados de cidades costumam errar por algumas centenas de quilômetros, e é para isso que serve a distância.

## [1.1.3] - 2026-09-20

### Added
- Ctrl+Space também busca no elenco e na equipe de tudo o que há na sua biblioteca, em **Elenco e equipe**, com foto, se a pessoa atua ou dirige e em quantos dos seus títulos ela aparece; escolher uma abre a página dela. Tão tolerante quanto a busca de títulos (palavras em qualquer ordem, acentos ignorados, um deslize do dedo permitido), e quando várias combinam igualmente bem, vem primeiro a que está em mais títulos seus.

## [1.1.2] - 2026-09-20

### Added
- **Adicionados recentemente** no painel, acima do gráfico de Atividade: as 30 chegadas mais recentes como uma fileira de pôsteres que você rola para os lados, com setas no computador. Cada um diz quando chegou e o que é (um filme, um álbum, ou “Temporada 4 · 3 episódios”), e um único episódio novo diz qual é, com o pôster da própria temporada quando ela tem um.
- **Shift + roda do mouse** e as setas do teclado movem um pôster, Page Up e Page Down uma tela inteira, Home e End vão para cada ponta, e deslizar o dedo funciona. A roda sozinha continua rolando a página.
- Episódios novos da mesma série que chegam no mesmo dia dividem uma entrada, e uma série inteira adicionada de uma vez é uma entrada só (“3 temporadas · 60 episódios”), então uma importação grande não empurra todo o resto para fora da lista.
- A fileira ignora o período e a pessoa do painel, já que é sobre a biblioteca e não sobre reproduções, e se esconde enquanto a biblioteca está vazia.

## [1.1.1] - 2026-09-20

### Added
- As páginas abrem na hora: um instante depois de o FinStats carregar, ele busca discretamente o que você provavelmente vai abrir em seguida (o painel, o seu perfil e a sua linha do tempo, Atividade, Usuários, Bibliotecas e cada biblioteca, Reprodução, as páginas do servidor, a retrospectiva e as pessoas mais ativas), então abrir uma delas a mostra imediatamente e ela ainda se atualiza nos bastidores.
- Filmes, séries e membros do elenco só são buscados por intenção, já que são milhares: deixar o ponteiro sobre um link, tocá-lo ou chegar até ele pelo teclado busca aquela página.
- Ele espera a página que você abriu terminar de carregar, trabalha uma página por vez enquanto o navegador está ocioso, pausa numa aba em segundo plano, não faz nada numa conexão com economia de dados ou muito lenta, e é esquecido quando você sai.
- Um pequeno link **Repo** com a marca do GitHub na barra de status, ao lado da versão.

### Fixed
- A **Linha do tempo** esquecia quais bibliotecas estavam desligadas quando a página era recarregada ou aberta de um favorito, embora a escolha estivesse no endereço.

## [1.1.0] - 2026-09-20

Uma linha do tempo de tudo o que você assistiu

### Added
- **Linha do tempo**, ao lado de **Visão geral** em todo perfil: o que você assistiu como uma trilha só, de hoje até a primeira reprodução que o FinStats conhece, com uma noite de episódios reunida numa única parada. Cada parada é um pôster com o que foi assistido e quando (“Temporada 2 · Episódios 3-6”, um filme e se levou mais de uma sessão, ou um álbum e quantas faixas), e abre o título.
- A trilha acompanha a largura da janela: três paradas por linha numa tela larga, duas numa mais estreita, uma linha sim, outra não correndo ao contrário, com uma curva ligando-a à seguinte, até virar uma única linha reta no celular. A primeira parada de cada mês traz o mês.
- As paradas mais antigas carregam sozinhas conforme você rola, até onde o histórico começa.
- Os interruptores de **Bibliotecas** acima da trilha deixam de fora o que você não quer ver, como música, e a escolha faz parte do endereço.
- Algo que ainda está tocando diz **Reproduzindo agora**.
- Você vê a sua própria linha do tempo; a de outras pessoas exige a permissão “ver todos”.

## [1.0.5] - 2026-09-20

### Added
- Proteção contra downgrade: o banco de dados lembra a versão mais nova do FinStats que o abriu, e uma versão mais antiga se recusa a iniciar com ele em vez de trabalhar silenciosamente com dados que não entende por completo. A mensagem diz o que fazer. As versões até a 1.0.4 foram lançadas antes de essa verificação existir.

## [1.0.4] - 2026-09-20

### Fixed
- Nos detalhes de uma reprodução, o botão de copiar ao lado de um **ID do dispositivo** longo tinha caído para uma linha própria.
- Na página Reprodução, a última coluna de **Quais clientes transcodificam** ficava cortada (“Transc…”); cabeçalhos de duas palavras agora quebram a linha.

### Changed
- As capturas de tela do README estão atualizadas de novo; ainda mostravam a versão 0.5.0.

## [1.0.3] - 2026-09-20

### Added
- A página de um episódio mostra a imagem do próprio episódio, a que o Jellyfin mostra na lista de episódios, em vez do pôster da série. Um episódio para o qual o Jellyfin não tem imagem fica com o pôster.

## [1.0.2] - 2026-09-20

### Fixed
- Nas **Notas da versão**, os grupos v0.9 e v0.10 não tinham manchete ao lado da versão. A manchete de um grupo agora é sempre a primeira frase do que tratava a primeira versão dele, então uma introdução longa não é cortada no meio da frase.

## [1.0.1] - 2026-09-20

### Fixed
- A imagem publicada não iniciava numa máquina nova: “unable to open database file: /data/finstats.db”, depois de ficar travada por meio minuto. O Docker cria uma pasta `data` que não existe como root, e o FinStats roda como usuário comum. O contêiner agora toma a pasta para si e depois passa a esse usuário, então `docker run` funciona de primeira. Para arquivos de outro dono que não o usuário 1000, defina `PUID` e `PGID`; `--user` funciona como antes.
- Uma pasta de dados em que realmente não dá para gravar é informada na hora, com o comando que resolve, em vez de 30 segundos depois como erro de banco de dados.

## [1.0.0] - 2026-09-20

Pronto para todos

### Added
- Uma imagem pronta, agora em `ghcr.io/finstats/finstats`, para máquinas Intel/AMD e ARM de 64 bits (um Raspberry Pi 4 ou 5 funciona): `:latest` é a versão mais recente, uma tag de versão fica onde está, e `:edge` é a versão de desenvolvimento.
- Toda versão aparece na página de releases do projeto no GitHub com estas notas da versão.

### Changed
- Os números de versão agora dizem o que significam: atualizações 1.x não vão quebrar os seus dados, os seus backups nem as suas configurações.
- O arquivo do Docker Compose e todas as instruções usam a imagem publicada. Um `finstats:latest` compilado por você só precisa trocar o nome da imagem; a pasta `data` continua intacta.

## [0.10.2] - 2026-09-20

### Fixed
- No celular, as Configurações e a página de uma série podiam rolar para os lados: um rótulo feito só para leitores de tela escapava da tabela, e as listas de episódios e a fileira do elenco faziam o mesmo. Agora elas rolam por dentro.
- A retrospectiva não rola mais para os lados quando a palavra de fundo de um capítulo é mais larga que a página.
- Uma linha “assistindo em…” comprida sob **Reproduzindo agora** termina em reticências em vez de ser cortada no meio de uma letra.
- O pôster na frente de cada série num perfil era um segundo link, sem nome, para leitores de tela; agora ele fica oculto para eles e o título ao lado é o link.

## [0.10.1] - 2026-09-20

### Fixed
- Baixar um backup falhava no navegador (“the source file could not be read” no Firefox) sem nada na página dizendo por quê: o FinStats compactava pela segunda vez um backup que já é compactado, e a transferência duplamente empacotada se interrompia antes do fim. Os backups são enviados como estão, com o tamanho informado de antemão, para que o navegador mostre o progresso real.

## [0.10.0] - 2026-09-20

Backups, e a mudança para uma nova instalação

### Added
- Um backup por semana, guardando os cinco mais recentes: toda reprodução com a sua linha do tempo de pausas e avanços, as marcas de visto, as permissões, os endereços de casa, o log do servidor e as suas configurações, num arquivo pequeno, algumas centenas de KB para milhares de reproduções. **Configurações → Backups** os lista com um botão **Baixar** para cada um, faz um na hora quando pedido, e define com que frequência e quantos. Eles ficam na pasta `backups` do seu diretório de dados. Um backup não contém chave de API do Jellyfin nem sessão de login, mas é um histórico de exibição completo com endereços IP, então guarde-o num lugar privado.
- Restaurar mescla em vez de sobrescrever: reproduções que já estão lá são ignoradas, então é seguro fazer duas vezes ou numa instância que já está rodando há algum tempo. Desmarque “Restaurar também as configurações e permissões” para levar só o histórico. Os backups funcionam entre versões nos dois sentidos. Pelo terminal: `finstats backup` e `finstats restore <file>`.

### Changed
- A coleta verifica a cada segundo enquanto alguém assiste e a cada 5 segundos enquanto ninguém assiste, onde antes eram 5 segundos o tempo todo, então pausas, avanços e trocas de faixa são registrados com precisão de segundo. As duas ficam em **Configurações → Coleta**; um intervalo antigo alterado precisa ser definido de novo.

### Fixed
- Um endereço IP nos detalhes de uma reprodução quebrava em duas linhas no meio do endereço. A marca Local/Remoto passa para baixo quando não há espaço.

## [0.9.1] - 2026-09-20

### Fixed
- Pessoas em casa apareciam como remotas. Um dispositivo na sua própria rede que chega ao Jellyfin pelo nome público dele, como por um proxy reverso ou um domínio, chega com o IP público da casa, e só endereços privados contavam como locais. O FinStats agora aprende o endereço público e conta essas reproduções como locais em todo o histórico, e lembra os endereços anteriores.

### Added
- **Configurações → Rede doméstica**: os endereços que o FinStats trata como casa, para adicionar os seus (um endereço anterior, uma segunda casa, uma saída de VPN) ou desligar a consulta. A consulta pergunta a um serviço simples de “qual é o meu IP” a cada 15 minutos, tentando vários (Amazon, Cloudflare e outros) porque DNS com bloqueio de anúncios como o Pi-hole costuma bloqueá-los, e um deles nem precisa de DNS. A requisição não contém nada sobre você ou o seu servidor, e com o interruptor desligado o FinStats não fala com nada além do Jellyfin. `FINSTATS_PUBLIC_IP_URL` define um serviço seu.

## [0.9.0] - 2026-09-20

Todas as tabelas, na ordem que você quiser

### Added
- Toda tabela ordena: clique num cabeçalho para ordenar por ele, de novo para inverter, uma terceira vez para a ordem original. O significado da célula é entendido, então “3d 2h” é ordenado como tempo e “1,4 GB” como tamanho, e células vazias ficam por último. Isso vale para a lista de usuários, quem assistiu, episódios, dispositivos, plugins, tarefas agendadas, a visualização em tabela de todo gráfico, a matriz de permissões e as listas de barras (codecs, resoluções, clientes e o resto), que ganham um cabeçalho fininho próprio.
- A Atividade e o Log do servidor ordenam em todas as páginas, e não só nas linhas da tela, e a ordem fica guardada no endereço, então sobrevive a uma recarga ou a um link compartilhado.
- Tabelas de dez linhas ou mais ganham um pequeno campo de filtro acima delas.

## [0.8.2] - 2026-09-20

### Changed
- O seletor de ano da retrospectiva é o mesmo interruptor de todos os outros seletores de período do FinStats (7d · 30d · 90d …).

## [0.8.1] - 2026-09-19

### Added
- Páginas para atores e diretores, a partir de “Pessoas mais assistidas” da retrospectiva ou da nova fileira **Elenco e equipe** em qualquer filme, série ou episódio: tudo em que a pessoa aparece no seu servidor, quanto foi assistido e por quem, e o que ainda espera. A página segue o período que você escolhe, quem só pode ver as próprias estatísticas vê ali só o que assistiu, e Esc leva você de volta.

## [0.8.0] - 2026-09-19

A retrospectiva tem visual novo, e mais a dizer

### Added
- **Pessoas mais assistidas**: os atores com quem você passou mais tempo e os diretores por trás do que você assistiu, como uma fileira de retratos. O elenco e a equipe são lidos junto com a biblioteca e aparecem depois da primeira leitura da biblioteca após esta atualização.
- **Seu ano em dias**: cada dia do ano como um quadradinho, mais claro quanto mais você assistiu, com a sua sequência mais longa e o seu maior dia acima. Passe o ponteiro sobre um dia, ou percorra o calendário com as setas, para ver o que ele teve.
- **Em números**: reproduções, tempo assistido, títulos diferentes, dias com algo assistido, sequência mais longa e quanto foi **revisto**, além de como o ano se divide entre episódios, filmes e música.
- O capítulo de gêneros diz quantos gêneros você experimentou e que fatia o principal levou.

### Changed
- Um design novo: ela abre no seu ano “revivido”, a manchete ao lado dos pôsteres que o preencheram, sobre uma forma de onda do ano com uma barra por semana e a mais alta acesa, e você escolhe o ano logo acima. Cada capítulo tem uma manchete com a palavra-chave em violeta, uma frase que traz os números, e a palavra do capítulo grande e apagada ao fundo. Ela termina como os filmes terminam, com os créditos: estrelando, dirigido por, exibido em, duração.
- Horas, dias da semana e meses dividem um só capítulo **Padrões de atividade** com um interruptor entre eles, e a manchete acompanha o que você está vendo (“A sexta-feira levou a coroa”).

### Fixed
- **Reproduzindo agora** no painel ficava em branco quando ninguém estava assistindo, então o título parecia pertencer aos filtros abaixo dele. A caixa “Nada tocando agora” voltou.

## [0.7.10] - 2026-09-19

### Added
- Arte das bibliotecas: cada card de biblioteca, e a própria página da biblioteca, mostra a imagem que o Jellyfin usa para ela, pelo proxy de imagens do FinStats como os pôsteres. Uma biblioteca sem imagem fica com o ícone.

## [0.7.9] - 2026-09-19

### Changed
- O card Gêneros volta a abrir como **Lista**, com o radar como segunda opção no interruptor. Uma escolha já feita é mantida.

## [0.7.8] - 2026-09-19

### Added
- Uma visualização em radar para o card Gêneros, no painel e nos perfis: um raio por gênero, alternado entre **Radar** e **Lista** no canto do card, e a escolha é lembrada. Passar o ponteiro sobre um ponto, ou usar as setas, mostra o tempo assistido e a parcela daquele gênero.

## [0.7.7] - 2026-09-19

### Fixed
- Filmes que pertencem a uma coleção apareciam como “Não está mais na biblioteca”. Com *group movies into collections* ligado, o Jellyfin entrega a coleção em vez dos filmes dentro dela, então o FinStats nunca os via; num servidor isso escondia 556 dos 1.238 itens da biblioteca de Filmes. Agora o FinStats pede os próprios filmes, e a leitura que esta atualização dispara os traz de volta e revincula as reproduções deles.

## [0.7.6] - 2026-09-19

### Changed
- Nos perfis, o card **Séries** desceu para logo acima de Gêneros, então uma lista longa de séries não empurra mais o tempo assistido, o gráfico de atividade e os títulos favoritos para fora da vista. As sequências de dias ficam no topo, e uma série que você abriu continua aberta quando você muda o período.

## [0.7.5] - 2026-09-19

### Changed
- A busca combina palavra por palavra, em qualquer ordem, com qualquer coisa no meio, em vez de exigir a frase exata, então “alya hides” encontra *Alya Sometimes Hides Her Feelings in Russian*. Maiúsculas, acentos e pontuação não importam, um “The” no início é ignorado, e um deslize do dedo é perdoado (“mentalsit” encontra *The Mentalist*); palavras de três letras ou menos ainda precisam ser digitadas certo. As melhores correspondências vêm primeiro, e músicas também são encontradas pelo artista.
- As caixas de busca da Atividade e do Log do servidor combinam palavra por palavra em todas as colunas: “alya opera” encontra reproduções daquela série no Opera.

## [0.7.4] - 2026-09-19

### Changed
- As notas da versão são agrupadas por série: cada série `0.x` é um bloco recolhível com as suas versões, com o assunto da série, quantas versões há e quando saíram. Só a mais recente começa aberta, e um bloco fechado continua mostrando qual versão você está rodando.

## [0.7.3] - 2026-09-19

### Fixed
- O relógio de Reproduzindo agora hesitava a cada poucos segundos: a atualização de 5 segundos também o redesenhava, então sempre que o servidor estava uma fração adiantado o número andava antes da hora e depois ficava parado por um segundo. Agora só o tique de uma vez por segundo o move, exatamente um segundo de cada vez (medido entre 999 e 1001 ms), e a atualização ainda corrige depois de uma pausa, de um salto ou de um desvio real, caindo no tique.

## [0.7.2] - 2026-09-19

### Added
- Assistir junto, ao vivo: cada card de Reproduzindo agora diz “Com maria” enquanto as pessoas assistem à mesma coisa juntas, onde antes um grupo só era reconhecido depois que as reproduções terminavam. Ao vivo, as pessoas contam como juntas quando estão no mesmo título e começaram dentro da janela de grupo ou estão quase na mesma posição, o que funciona mesmo se o FinStats tiver sido reiniciado no meio do stream.

## [0.7.1] - 2026-09-19

### Changed
- Reproduzindo agora conta cada segundo: o relógio avança e a barra desliza entre uma consulta e outra, em vez de pular a cada vez que o servidor é consultado. O servidor ainda é consultado a cada 5 segundos e continua sendo a fonte da verdade: a tela se ressincroniza numa pausa, num salto ou quando o servidor está adiantado, mas não por uma posição apenas alguns segundos defasada, já que os clientes só informam mais ou menos a cada dez segundos. Streams pausados ficam parados.

## [0.7.0] - 2026-09-19

Quem assiste junto

### Added
- Assistir junto: quando pessoas diferentes começam o mesmo título ao mesmo tempo, o FinStats conta isso como assistir junto. Um card **Assistido junto** no painel mostra os grupos, os títulos que eles dividem e as horas juntos, cada perfil mostra com quem aquela pessoa mais assiste, as reproduções compartilhadas levam uma pequena marca de pessoas na Atividade, e os detalhes da reprodução dizem com quem ela foi assistida. Funciona em todo o seu histórico, inclusive nas reproduções importadas.
- Uma configuração para o quão próximos os inícios precisam estar, 60 segundos por padrão: cerca de um terço das sessões em grupo genuínas começa com 6 a 60 segundos de diferença. As pessoas também precisam continuar assistindo lado a lado por alguns minutos, então duas pessoas abrindo o mesmo episódio por coincidência não formam um grupo.

### Changed
- A busca abre com **Ctrl + Space** em vez de Ctrl/⌘ + K.

## [0.6.1] - 2026-09-19

### Added
- Administradores podem abrir a retrospectiva de outra pessoa, no topo da página Retrospectiva ou com **Abrir retrospectiva** no perfil dela. Ela então fala da pessoa pelo nome em vez de “você”.

### Changed
- Todos os outros continuam vendo só a própria retrospectiva, sejam quais forem as permissões, e continua não havendo retrospectiva do servidor inteiro.

## [0.6.0] - 2026-09-19

Seu perfil, e onde você está em cada série

### Added
- Progresso das séries nos perfis: cada série que você começou como uma barra com um segmento por episódio (visto, começado ou ainda não), que abre temporada por temporada, ordenada pelo quanto falta para você terminar, com visualizações separadas para séries concluídas e para tudo.
- **Marcar como visto** no seu próprio perfil, por episódio, temporada ou série, para o que você assistiu enquanto nada estava registrando. As marcas de reproduzido do próprio Jellyfin também são usadas. As marcas ficam no FinStats e nunca mudam nada no Jellyfin; uma reprodução registrada não pode ser desmarcada, as suas próprias marcas podem.
- Sequências de dias nos perfis: a sua maior série de dias seguidos com uma reprodução, a sequência atual e em quantos dias você assistiu alguma coisa.
- **Meu perfil** na barra lateral.
- A Atividade mostra onde a reprodução parou (por exemplo `44:15 / 45:00`) sob a barra de progresso, e a data e a hora sob “há 3 h”. Reproduções importadas do Jellystat não têm posição de parada, porque o Jellystat nunca registrou uma.

### Changed
- Episódios que estão faltando ou ainda não foram ao ar não são mais tratados como parte da sua biblioteca. O Jellyfin os lista como marcadores de posição sem arquivo, e eles contavam nos totais de episódios.

## [0.5.1] - 2026-09-19

### Added
- Um logo: um botão de play fatiado em três barras de gráfico, na barra lateral e nas telas de login, como SVG nítido com um `favicon.ico` de vários tamanhos para navegadores e barras de favoritos que queiram um, além de um ícone de tela inicial para celulares e tablets.

## [0.5.0] - 2026-09-19

Você decide quem vê o quê

### Added
- Permissões em Configurações → Acesso, para todos de uma vez ou pessoa por pessoa: **Ver a atividade de todos** (as estatísticas e o histórico de outras pessoas, a página Usuários, todo stream ao vivo), **Ver detalhes de rede** (endereços IP, ids de dispositivo, local ou remoto), **Ver o servidor** (a página Servidor, o log do servidor, logins com falha, caminhos de arquivo) e **Gerenciar o FinStats** (configurações, tarefas, importação, exclusão de reproduções).
- Login por pessoa, para deixar algumas pessoas entrarem sem abrir o FinStats a todos os usuários do Jellyfin.
- As mudanças valem na hora, inclusive retirar o acesso, e tudo é garantido pelo servidor, e não escondendo botões.

### Changed
- Os administradores do Jellyfin continuam tendo tudo e são os únicos que podem mudar permissões; quem pode gerenciar o FinStats não pode conceder nada, nem a si mesmo nem a ninguém.
- A retrospectiva do ano continua pessoal, seja o que for concedido.
- Nada muda nas instalações existentes até você conceder alguma coisa.

## [0.4.5] - 2026-09-19

### Changed
- O FinStats é licenciado sob a **GNU General Public License v3.0**. Nenhuma versão anterior foi publicada, então nada foi lançado sob outros termos. As licenças das fontes incluídas vão junto com elas.

## [0.4.4] - 2026-09-19

### Added
- Esc volta um passo: numa biblioteca, num usuário ou num título aberto, ele leva você de volta para onde estava, com a lista exatamente como você a deixou. O que estiver aberto fecha primeiro (detalhes da reprodução, a busca, uma lista suspensa, o menu do celular), e o Esc é deixado de lado enquanto você digita ou usa um gráfico pelo teclado.

## [0.4.3] - 2026-09-19

### Changed
- O FinStats lembra o que cada página mostrou por último: voltar a um perfil, uma biblioteca ou um período que você já viu o exibe na hora, com foto de perfil e tudo, enquanto os números novos carregam por trás, e a página só é redesenhada se algo mudou. O que fica lembrado vive na aba do navegador e é descartado quando você sai ou muda qualquer coisa.
- Os indicadores de carregamento só aparecem quando o carregamento é realmente lento. Antes, toda página os mostrava por pelo menos um terço de segundo.

### Fixed
- Bibliotecas que foram excluídas no Jellyfin e nunca tiveram uma reprodução não são mais listadas. Uma biblioteca excluída com histórico continua aparecendo, marcada como removida.

## [0.4.2] - 2026-09-19

### Fixed
- Na página Usuários, a etiqueta **Admin** ficava numa linha própria e deixava aquela linha mais alta que as outras. Agora as etiquetas ficam ao lado do nome.

## [0.4.1] - 2026-09-19

### Fixed
- A palavra “null” não aparece mais sob o título do painel fora de dezembro e janeiro, quando não há faixa da retrospectiva para mostrar.

## [0.4.0] - 2026-09-19

Veja o que mudou sem sair do app

### Added
- Aba **Notas da versão** mostrando todas as versões, com a que você está rodando marcada.
- A versão na barra de status leva até ela, e a aba mostra um ponto depois de uma atualização até você dar uma olhada.

### Changed
- Revincular o histórico a itens renomeados ficou muito mais rápido: de 18 segundos para um terço de segundo com 666 títulos órfãos, então a inicialização continua instantânea conforme o histórico cresce.

## [0.3.3] - 2026-09-19

### Changed
- A biblioteca é relida logo depois que a tarefa **Scan Media Library** do Jellyfin termina, nunca num temporizador separado e nunca durante uma varredura. O FinStats sempre foi somente leitura em relação ao Jellyfin, e nunca inicia uma varredura lá.
- Uma leitura semanal de segurança cobre servidores que dependem do monitoramento em tempo real, nos quais a tarefa de varredura pode nunca rodar.
- O intervalo antigo agora é só uma alternativa, para quando o acompanhamento está desligado ou o servidor não informa nenhuma tarefa de varredura.

### Added
- Interruptor **Seguir a varredura de biblioteca do Jellyfin** em Configurações → Coleta, ligado por padrão.

## [0.3.2] - 2026-09-19

### Fixed
- O histórico sobrevive a renomeações. O Jellyfin dá um id novo a um arquivo renomeado, o que deixava as reproduções anteriores dele largadas sob o nome cru da pasta (`Title (2010) [tmdbid-…] [imdbid-…]`) e sem pôster. Essas reproduções são revinculadas à entrada atual da biblioteca: pelo id do TMDB/IMDb/TVDB quando o nome antigo traz um, senão pelo título limpo e pelo ano, e episódios pela série mais o número da temporada e do episódio. Uma correspondência precisa ser inequívoca, então títulos que foram realmente excluídos continuam como estão.
- Canais de TV ao vivo importados do Jellystat eram contados como filmes, porque o Jellystat não registra tipo de item. Agora são reconhecidos como TV ao vivo, e novas reproduções de TV ao vivo são registradas com o próprio tipo do Jellyfin.

### Changed
- A TV ao vivo fica totalmente de fora da retrospectiva: listas de favoritos, totais, personalidade, recordes e posição no ranking.

### Added
- `finstats relink` para revincular o histórico sob demanda. Ele também roda na inicialização e depois de toda leitura da biblioteca.

## [0.3.1] - 2026-09-19

### Changed
- A retrospectiva é estritamente pessoal: todos, administradores incluídos, veem só a própria, e a visualização “Todos”, o seletor de usuário e o capítulo do servidor saíram.
- A retrospectiva abre no ano que está “pronto”: o ano atual durante dezembro, senão o anterior. Os outros anos e “Últimos 12 meses” continuam a um clique.

## [0.3.0] - 2026-09-19

Seu ano em retrospectiva

### Added
- Aba **Retrospectiva**, uma história de um ano para rolar: horas assistidas, séries, filmes, faixas e gêneros favoritos, e uma personalidade de espectador (coruja, madrugador, guerreiro de fim de semana, maratonista, cinéfilo, amante da música, criatura de hábitos) com gráficos por hora do dia e por dia da semana.
- O ano mês a mês, com o título que marcou cada mês.
- Recordes: o maior dia, a maior maratona, a sequência diária mais longa, a reprodução única mais longa, o título mais revisto, a primeira reprodução do ano e o título mais antigo assistido.
- Descobertas: séries novas começadas, filmes e episódios concluídos, e as séries que tiveram exatamente um episódio.
- Uma faixa de uma linha no painel em dezembro e janeiro quando uma retrospectiva está pronta.

## [0.2.0] - 2026-09-19

Mais do que o Jellystat jamais registrou

### Added
- Uma linha do tempo para cada reprodução: início, pausa, retomada, salto, trocas de áudio e de legenda, reprodução direta virando transcodificação, e parada. Além disso, contagens de pausas e saltos, de onde a reprodução foi retomada, e se o espectador estava na rede local.
- Aba **Servidor**: versão do Jellyfin, atualizações e reinícios pendentes, uso de disco por biblioteca (Jellyfin 10.11+), plugins, resultados das tarefas agendadas e todos os dispositivos registrados.
- Insights de **Reprodução**: streams simultâneos ao longo do tempo, quais clientes forçam transcodificações, até onde as pessoas chegam antes de parar, reproduções locais contra remotas e comportamento de exibição.
- Do que a sua biblioteca é feita: resoluções, codecs, faixa dinâmica, contêineres, títulos por década, adições por mês, os maiores títulos, e os títulos que ninguém nunca assistiu, conferidos tanto no histórico do FinStats quanto nas marcas de reproduzido do próprio Jellyfin.
- Painel: pico de streams simultâneos, estimativa de dados transmitidos, parcela de reproduções remotas, gêneros por tempo assistido e logins com falha.
- Páginas de item: links do IMDb e do TMDB, estúdios, profundidade de bits e taxa de quadros, e quem tem o título marcado como reproduzido no Jellyfin.
- Páginas de usuário: gêneros, e os filmes, episódios e favoritos que o Jellyfin tem registrados.

### Fixed
- O rótulo de tempo num card de Reproduzindo agora não transborda mais do card.
- Rótulos longos na página Reprodução eram cortados cedo demais.
- Uma série que saiu da biblioteca continua mostrando a lista de temporadas e episódios.

## [0.1.0] - 2026-09-19

Primeira versão

### Added
- Coleta ao vivo das sessões do Jellyfin: usuário, título, cliente, dispositivo, endereço IP, método de reprodução, motivos da transcodificação e aceleração de hardware, detalhes de vídeo, áudio e legenda, e tempo assistido contra tempo pausado.
- Login com a sua conta do Jellyfin. Uma configuração em duas etapas cria a própria chave de API do FinStats, e as senhas nunca são guardadas. Usuários que não são administradores podem, opcionalmente, entrar para ver só as próprias estatísticas.
- Páginas Painel, Atividade, Usuários, Bibliotecas, Reprodução e Log do servidor, uma paleta de comandos (Ctrl/⌘ K) e uma barra de status ao vivo.
- Importação do Jellystat: backups grandes são gravados em disco aos poucos e importados em segundos, e importar o mesmo arquivo duas vezes é seguro.
- Um único binário pequeno com a interface web embutida e um arquivo SQLite, além de uma imagem Docker.
