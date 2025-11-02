Vamos ajustar o layout do terminal SEM mudar formatação nem estilos nem funcionalidades:



Como no exemplo abaixo, preciso que tenha um rodapé e cabeçalho fixo, com uma are de digitação e resultado no meio com scroll, ou seja,

vamos ir digitando os comandos e tarefas e o rodapé e cabeço deve ser mantido estáticos mantendo sempre fixos, o meio que (área dinâmica) que deve ser atualizado.

Projeto esta em nettoolskit-cli.



EXEMPLO:

-> cabeçalho

╭─────────────────────────────────────────────────────────────────────────────────────────╮

│ >\_ NetToolsKit CLI (1.0.0)                                                                                       │

│    A comprehensive toolkit for backend development                                                               │

│                                                                                                                  │

│    directory: ~\\Documents\\Trabalho\\...\\NetToolsKit\\tools\\nettoolskit-cli                                         │

╰─────────────────────────────────────────────────────────────────────────────────────────╯





 ███╗   ██╗███████╗████████╗████████╗ ██████╗  ██████╗ ██╗     ███████╗██╗  ██╗██╗████████╗

 ████╗  ██║██╔════╝╚══██╔══╝╚══██╔══╝██╔═══██╗██╔═══██╗██║     ██╔════╝██║ ██╔╝██║╚══██╔══╝

 ██╔██╗ ██║█████╗     ██║      ██║   ██║   ██║██║   ██║██║     ███████╗█████╔╝ ██║   ██║

 ██║╚██╗██║██╔══╝     ██║      ██║   ██║   ██║██║   ██║██║     ╚════██║██╔═██╗ ██║   ██║

 ██║ ╚████║███████╗   ██║      ██║   ╚██████╔╝╚██████╔╝███████╗███████║██║  ██╗██║   ██║

 ╚═╝  ╚═══╝╚══════╝   ╚═╝      ╚═╝    ╚═════╝  ╚═════╝ ╚══════╝╚══════╝╚═╝  ╚═╝╚═╝   ╚═╝





💡 Tip: Type / to see available commands

   Use ↑↓ to navigate, Enter to select, /quit to exit

-> cabeçalho



-> área dinâmica



>



-> área dinâmica



-> rodapé

---

2025-10-30T19:08:38.309653Z  INFO 76: Starting NetToolsKit CLI interactive mode

2025-10-30T19:08:38.309707Z  INFO 28: Initializing metrics collector

2025-10-30T19:08:38.373509Z  INFO 96: Displaying application logo and UI

2025-10-30T19:08:48.444836Z  INFO 28: Initializing metrics collector

2025-10-30T19:08:48.444916Z  INFO 33: Processing CLI command command=/check command\_type=check

2025-10-30T19:08:48.445130Z  INFO 153: Operation completed operation=command\_execution duration\_ms=0

2025-10-30T19:08:48.445218Z  WARN 167: Timer dropped without explicit stop - auto-recording operation=command\_execution duration\_ms=0

2025-10-30T19:08:48.445272Z  INFO 90: Command execution completed command=/check duration\_ms=0 status="error"

2025-10-30T19:08:48.445333Z  INFO 113: Metrics summary logged counter\_count=2 gauge\_count=0



---

-> rodapé