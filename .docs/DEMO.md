# Demonstração do CLI NetToolsKit

## Como testar o CLI

### 1. Build do projeto
```bash
cd tools/NetToolsKit.CLI
cargo build
```

### 2. Teste de comandos diretos
```bash
# Listar templates
cargo run -- list

# Ajuda
cargo run -- --help

# Criar novo projeto
cargo run -- new dotnet-api --name "MyAPI"

# Validar arquivo
cargo run -- check manifest.yml

# Renderizar template
cargo run -- render dotnet-api

# Aplicar manifest
cargo run -- apply manifest.yml
```

### 3. Modo Interativo
```bash
# Executar CLI interativo
cargo run

# No prompt interativo:
# - Digite "/" para abrir a paleta de comandos
# - Use setas ↑↓ para navegar
# - Digite para filtrar comandos (ex: "/li" filtra para /list)
# - Pressione Enter ou Tab para selecionar
# - Pressione Esc para cancelar
# - Digite "/quit" para sair
```

## Funcionalidades Implementadas

✅ **Estrutura Modular do Codex**
- Módulos async-utils, file-search, otel, ollama
- Arquitetura workspace com múltiplos crates

✅ **Comandos Slash**
- /list - Lista templates disponíveis
- /new - Cria projeto de template
- /check - Valida manifesto ou template
- /render - Renderiza preview de template
- /apply - Aplica manifesto a solução
- /help - Mostra ajuda detalhada
- /quit - Sai do CLI

✅ **Paleta Interativa**
- Ativação com "/"
- Filtro em tempo real
- Navegação por setas (↑↓)
- Seleção com Enter/Tab
- Cancelamento com Esc
- Visual similar ao Codex

✅ **Interface Moderna**
- Cores e ícones com owo-colors
- Feedback visual claro
- Tratamento de Ctrl+C
- Modo raw terminal

✅ **Testes Implementados**
- Testes de comandos slash
- Testes de filtros de arquivo
- Validação de funcionalidade

## Próximos Passos

1. **Implementação Real dos Comandos**
   - Integrar com sistema de templates real
   - Implementar parsing de manifests YAML
   - Adicionar geração de código com Handlebars

2. **Melhorias na UX**
   - Histórico de comandos
   - Autocomplete avançado
   - Progress bars para operações longas

3. **Integração com NetToolsKit**
   - Conectar com módulos do NetToolsKit existentes
   - Usar templates e manifestos reais do projeto
   - Integração com pipeline de CI/CD

4. **Extensibilidade**
   - Plugin system para comandos personalizados
   - Configuração via arquivo de config
   - Suporte a múltiplas linguagens de template