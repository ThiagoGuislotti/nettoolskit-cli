# NetToolsKit CLI

Um CLI interativo para desenvolvimento .NET com templates, manifests e ferramentas de automação, inspirado no design do Codex CLI.

## ✨ Características

- **Interface Interativa**: CLI modo terminal com paleta de comandos ativada por `/`
- **Paleta de Comandos**: Similar ao Codex, com filtro em tempo real e navegação por setas
- **Comandos Slash**: Sistema de comandos padronizado começando com `/`
- **Modularidade**: Arquitetura baseada em módulos reutilizáveis
- **Performance**: Implementado em Rust para máxima velocidade

## 🚀 Instalação

```bash
# Clone o repositório
git clone https://github.com/your-org/NetToolsKit.git

# Navegue para o diretório do CLI
cd NetToolsKit/tools/NetToolsKit.CLI

# Compile e instale
cargo install --path cli
```

## 📋 Comandos Disponíveis

| Comando | Descrição |
|---------|-----------|
| `/list` | Lista templates disponíveis |
| `/new` | Cria projeto a partir de template |
| `/check` | Valida manifest ou template |
| `/render` | Renderiza preview de template |
| `/apply` | Aplica manifest à solução existente |
| `/help` | Mostra ajuda detalhada |
| `/quit` | Sai do NetToolsKit CLI |

## 💡 Como Usar

### Modo Interativo

Execute o CLI sem argumentos para entrar no modo interativo:

```bash
ntk
```

### Paleta de Comandos

1. Digite `/` para abrir a paleta de comandos
2. Continue digitando para filtrar os comandos
3. Use `↑` e `↓` para navegar
4. Pressione `Enter` ou `Tab` para selecionar
5. Pressione `Esc` para cancelar

### Comandos Diretos

Você também pode executar comandos diretamente:

```bash
# Listar templates
ntk list --filter "dotnet"

# Criar novo projeto
ntk new dotnet-api --name "MyAPI" --output "./my-api"

# Validar manifest
ntk check manifest.yml --strict

# Renderizar template
ntk render dotnet-api --vars variables.json

# Aplicar manifest
ntk apply manifest.yml --target ./my-solution
```

## 🏗️ Arquitetura

O projeto segue a estrutura modular do Codex:

```
NetToolsKit.CLI/
├── cli/                    # CLI principal
│   ├── src/
│   │   ├── main.rs        # Ponto de entrada
│   │   ├── lib.rs         # Modo interativo
│   │   ├── commands.rs        # Definição dos comandos CLI
│   │   ├── command_palette.rs # Paleta interativa
│   │   └── commands/      # Implementação dos comandos
│   └── tests/             # Testes
├── async-utils/           # Utilitários assíncronos
├── file-search/          # Busca e filtros de arquivo
├── otel/                 # OpenTelemetry/observabilidade
├── ollama/               # Integração com Ollama
└── Cargo.toml            # Workspace configuration
```

## 🧪 Testes

Execute os testes com:

```bash
cargo test
```

## 🎨 Design Inspirado no Codex

Este CLI foi desenvolvido seguindo o excelente design de UX do Codex CLI:

- **Paleta de Comandos**: Ativação por `/`, filtro em tempo real, navegação intuitiva
- **Feedback Visual**: Uso de cores e ícones para melhor experiência
- **Modularidade**: Separação clara de responsabilidades em módulos
- **Performance**: Otimizado para resposta instantânea do usuário

## 📝 Licença

Este projeto está licenciado sob a Licença MIT - veja o arquivo [LICENSE](../../LICENSE) para detalhes.

## 🤝 Contribuindo

Contribuições são bem-vindas! Por favor, leia nossas diretrizes de contribuição antes de submeter pull requests.

## 📞 Suporte

Para suporte e dúvidas:
- Abra uma [issue](https://github.com/your-org/NetToolsKit/issues)
- Consulte a [documentação](../../docs/)

---

**NetToolsKit CLI** - Ferramentas poderosas para desenvolvimento .NET 🚀