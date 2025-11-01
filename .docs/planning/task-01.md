Preciso que vc use como BASE os arquivos do tools\\codex\\codex-rs\\cli para termos o msm CLI com o msm design da lista interativa em tempo real do códex, a lista interativa é iniciada quando digitamos "/" no primeiro caractere.

A imagem tools\\nettoolskit-cli\\.docs\\codex-list.png é de como o códex é, e vc deve remover a lista de comando do codex e usara minha lista de comandos.

Não precisa implementar os comandos deixo como TODO que depois fazemos, deixar pronto somente o quit.



REFERENCIA:

\- tools\\nettoolskit-cli\\.docs\\ntk-cli.png

\- tools\\nettoolskit-cli\\.docs\\codex-initial.png

\- tools\\nettoolskit-cli\\.docs\\codex-list.png

\- tools\\nettoolskit-cli\\.docs\\codex-list-filter.png

\- tools\\nettoolskit-cli\\.docs\\INTERACTIVE-MENU-DESIGN.md

\- tools\\nettoolskit-cli\\.docs\\paleta-codex.md



REGRAS:

\- Usar Lista de comandos que estão em tools\\nettoolskit-cli\\.docs\\paleta-codex.md.

\- Implementar apenas o comando quit.

\- Implementar em tools\\nettoolskit-cli, com o MSM padrão do CODEX, com módulos (cli, ...) e dentro de cada modulo deve ter src e tests.

\- manter as pastas do códex (otel, olhama, file-search, async-utils) essas coisas quero manter e aproveitar pois os devs dele são excelentes.



TAREFAS:

1- quando iniciar sempre deve limpar o terminal.

2- deve seguir esse UI com a logo tools\\nettoolskit-cli\\.docs\\ntk-cli.png.

3- a tecla "/" deve funcionar, E quando digitar ela, é que deve abrir o menu interativo Abaixo da área de digitação como no tools\\nettoolskit-cli\\.docs\\codex-list.png.
4- a cor do meu sistema deve ser a msm do ícone assets\\nuget-icon.png.

5- a lista interativa deve pular uma linha abaixo da aread de digitação, e deve ser navegada pelas teclas de navegação do teclado.
6- Ao ir digitando após o "/" deve ir filtrando a lista abaixo conforme tools\\nettoolskit-cli\\.docs\\codex-list-filter.png



╭─────────────────────────────────────────────────────────────────────────────────────╮

│ >\_ NetToolsKit CLI  (Version)                                                       │

│    A powerful toolkit for .NET development                                          │

│                                                                                     |

|    directory:  ~\\Documents\\Trabalho\\Pessoal\\Desenvolvimento\\…\\tools\\nettoolskit-cli |

╰─────────────────────────────────────────────────────────────────────────────────────╯

