## 📋 Executive Summary

This document consolidates the complete command reference for NetToolsKit CLI and the manifest specification for .NET code generation. The CLI provides a set of commands for discovery, validation, rendering, and application of templates/manifests in .NET solutions.

---

## 🎯 Available commands

### `/list` - List Manifests

**Status**: ✅ **Implemented**
**File**: `crates/commands/management/src/handlers/list.rs`

#### Description
Lists all manifest files discovered in the current workspace.

#### Usage
```bash
> /list
```

#### Expected Output
```
📦 Discovered Manifests:
├─ ntk-manifest-artifact.yml
├─ ntk-manifest-feature.yml
└─ ntk-manifest-layer.yml

3 manifest(s) found.
```

#### Implementation
- Recursive search for `*.yml` and `*.yaml` files in workspace
- Filters files matching the pattern `ntk-manifest-*.yml`
- Displays relative path and manifest type (artifact/feature/layer)

#### Telemetry
- `command.list.success` - contador de execuções bem-sucedidas
- `command.list.manifests_found` - histograma de manifestos encontrados
- `command.list.duration` - duração da execução

---

### `/check` - Validate Manifest/Template

**Status**: ⏳ **In Development**
**File**: `crates/commands/management/src/handlers/check.rs`

#### Description
Validates the structure and semantics of a manifest or template.

#### Usage
```bash
> /check ntk-manifest-artifact.yml
> /check --template templates/dotnet/Entity.cs.hbs
```

#### Arguments
- `<FILE>` - Path to the file to validate (manifest or template)
- `--template` - Flag to indicate template validation (optional)

#### Validations
**Manifest**:
- Valid YAML schema
- Supported `apiVersion` (`nettoolskit.io/v1`)
- Valid `kind` (`ArtifactManifest` | `FeatureManifest` | `LayerManifest`)
- Template references exist
- Solution/project guards are consistent
- JSON Path selectors are valid

**Template**:
- Valid Handlebars syntax
- Referenced variables are declared
- Helpers used are available
- Referenced partials exist

#### Expected Output (Success)
```
✅ ntk-manifest-artifact.yml is valid

Manifest Summary:
  Kind: ArtifactManifest
  Contexts: 1
  Templates: 3
  Target Projects: 1

No issues found.
```

#### Expected Output (Error)
```
❌ ntk-manifest-artifact.yml has errors

Validation Errors:
  [Line 15] Unknown template: templates/dotnet/Missing.cs.hbs
  [Line 23] Invalid JSON Path: contexts[*].invalid_selector
  [Line 35] Guard violation: solution root not found

3 error(s), 0 warning(s)
```

#### Telemetry
- `command.check.success` / `command.check.error`
- `command.check.validation_errors` - histograma de erros
- `command.check.duration`

---

### `/render` - Template Preview

**Status**: 📋 **Planned**
**File**: `crates/commands/management/src/handlers/render.rs` (pendente)

#### Description
Renders a template with provided variables without writing to disk (preview only).

#### Usage
```bash
> /render templates/dotnet/Entity.cs.hbs --var Name=Product --var Namespace=Rent.Service.Domain
> /render templates/dotnet/Entity.cs.hbs --vars-file render-vars.json
```

#### Arguments
- `<TEMPLATE>` - Handlebars template path
- `--var KEY=VALUE` - Define inline variable (repeatable)
- `--vars-file <FILE>` - Load variables from JSON/YAML file
- `--output <FILE>` - Save preview to file instead of stdout

#### Expected Output
```
📝 Preview: templates/dotnet/Entity.cs.hbs

Variables:
  Name: Product
  Namespace: Rent.Service.Domain
  TargetFramework: net8.0
  Author: NetToolsKit

---
Namespace Rent.Service.Domain.Entities;

/// <summary>
/// Represents a Product entity
/// </summary>
public sealed class Product
{
    // TODO: Add properties
}
---

Preview complete (25 lines).
```

#### Features
- Syntax highlighting of rendered code
- Automatic language detection (C#, TypeScript, etc.)
- Required variable validation
- ANSI color preview in terminal

#### Telemetry
- `command.render.success` / `command.render.error`
- `command.render.variables_used`
- `command.render.template_size`

---

### `/new` - Create New Project

**Status**: 📋 **Planned**
**File**: `crates/commands/management/src/handlers/new.rs` (pending)

#### Description
Creates a new project from a predefined template.

#### Usage
```bash
> /new dotnet-microservice --name RentService --output ./services
> /new dotnet-domain-entity --name Product
```

#### Arguments
- `<TEMPLATE_ID>` - Template identifier (e.g.: `dotnet-microservice`)
- `--name <NAME>` - Project/component name
- `--output <DIR>` - Target directory (default: current workspace)
- `--set KEY=VALUE` - Template variable override

#### Available Templates
1. **dotnet-microservice** - Complete microservice with Clean Architecture
2. **dotnet-domain-entity** - DDD domain entity
3. **dotnet-use-case** - Use case with CQRS + MediatR
4. **dotnet-api-controller** - ASP.NET Core controller

#### Expected Output
```
🚀 Creating project from template: dotnet-microservice

Project: RentService
Output: ./services/RentService

Creating structure...
  [create] RentService.sln
  [create] src/RentService.Domain/RentService.Domain.csproj
  [create] src/RentService.Application/RentService.Application.csproj
  [create] src/RentService.Infrastructure/RentService.Infrastructure.csproj
  [create] src/RentService.API/RentService.API.csproj
  [create] tests/RentService.Tests.Unit/RentService.Tests.Unit.csproj

✅ Project created successfully (12 files, 2.3 MB)

Next steps:
  cd services/RentService
  dotnet restore
  dotnet build
```

#### Telemetry
- `command.new.success` / `command.new.error`
- `command.new.template_used`
- `command.new.files_created`

---

### `/apply` - Apply Manifest

**Status**: 🔄 **In Development** (Phase 2.4)
**File**: `crates/commands/manifest/src/apply.rs`
**Document**: `task-phase-2.4-manifest-apply-dotnet.md`

#### Description
Applies a solution manifest, creating/updating files in the existing .NET solution according to defined rules.

#### Usage
```bash
> /apply ntk-manifest-artifact.yml
> /apply ntk-manifest-feature.yml --dry-run
> /apply ntk-manifest-layer.yml --output ./target/solution
```

#### Arguments
- `<MANIFEST>` - Manifest file path (Pattern: `ntk-manifest.yml`)
- `--output <DIR>` - Solution root directory (default: `target/ntk-output`)
- `--dry-run` / `-n` - Preview changes without writing files
- `--with-post` - Execute post-steps after application (future)
- `--strict-post` - Fail if post-step returns error (future)

#### Execution Flow

1. **Load & Validate**
   ```
   📂 Loading manifest: ntk-manifest-artifact.yml
   ✅ Manifest validation passed

   Manifest Info:
     Kind: ArtifactManifest
     API Version: nettoolskit.io/v1
     Contexts: 1 (Rent)
     Aggregates: 2
     Entities: 3
     Templates: 4
   ```

2. **Guard Checks**
   ```
   🔒 Checking guards...
   ✅ Solution root exists: samples/src
   ✅ Solution file found: Rent.Service.sln
   ✅ Project exists: Rent.Service.Domain
   ```

3. **Template Resolution**
   ```
   🔍 Resolving templates...
   ✅ Entity.cs.hbs -> 3 target(s)
   ✅ Repository.cs.hbs -> 2 target(s)
   ✅ IRepository.cs.hbs -> 2 target(s)

   Total: 7 file(s) to process
   ```

4. **Change Plan**
   ```
   📝 Change Plan:

   CREATE  samples/src/Rent.Service.Domain/Entities/Rental.cs
   CREATE  samples/src/Rent.Service.Domain/Entities/Vehicle.cs
   CREATE  samples/src/Rent.Service.Domain/Entities/Customer.cs
   CREATE  samples/src/Rent.Service.Domain/Repositories/IRentalRepository.cs
   CREATE  samples/src/Rent.Service.Infrastructure/Repositories/RentalRepository.cs
   UPDATE  samples/src/Rent.Service.sln (add projects)

   Summary: 5 create(s), 1 update(s), 0 skip(s)
   ```

5. **Execution** (se não for dry-run)
   ```
   🚀 Applying changes...
   [create] Rental.cs ✅
   [create] Vehicle.cs ✅
   [create] Customer.cs ✅
   [create] IRentalRepository.cs ✅
   [create] RentalRepository.cs ✅
   [update] Rent.Service.sln ✅

   ✅ Apply complete!

   Files created: 5
   Files updated: 1
   TODO markers: 3
   Duration: 1.2s
   ```

#### Collision Policies

```yaml
conventions:
  policy:
    collision: fail  # ou: safe, force, skip
    insertTodoWhenMissing: true
```

- **fail**: Abort execution if file already exists
- **safe**: Create backup before overwriting (future)
- **force**: Overwrite without asking (future)
- **skip**: Ignore existing files

#### TODO Markers

When `insertTodoWhenMissing: true` and template detects empty sections:

```csharp
Namespace Rent.Service.Domain.Entities;

public sealed class Rental
{
    // TODO: Add properties for Rental entity
    // Generated by: ntk-manifest-artifact.yml (artifact: Entity)
    // Context: Rent, Aggregate: Rental
}
```

#### Telemetry
- `command.apply.success` / `command.apply.error`
- `command.apply.files_created`
- `command.apply.files_updated`
- `command.apply.collisions_detected`
- `command.apply.todos_inserted`
- `command.apply.duration`

---

### `/quit` - Exit CLI

**Status**: ✅ **Implemented**
**File**: `crates/commands/management/src/handlers/quit.rs`

#### Description
Exits the NetToolsKit CLI gracefully.

#### Usage
```bash
> /quit
```

#### Output
```
👋 Goodbye!
```

#### Behavior
- Saves session state (command history, preferences)
- Finalizes async processes in progress
- Closes telemetry and flushes logs
- Returns `ExitStatus::Success` (code 0)

---

## 📄 Manifest Structure

### Manifest Types

#### 1. **ArtifactManifest** - Individual Artifact Generation

Used to generate isolated components (entities, repositories, controllers).

```yaml
apiVersion: nettoolskit.io/v1
kind: ArtifactManifest

meta:
  name: domain-entities
  description: Generate domain entities for Rent context
  author: NetToolsKit Team
  version: 1.0.0

solution:
  root: samples/src
  slnFile: Rent.Service.sln
  guard: requireExisting

projects:
  - name: Rent.Service.Domain
    path: Rent.Service.Domain
    targetFramework: net8.0

contexts:
  - name: Rent
    description: Rent and vehicle management
    aggregates:
      - name: Rental
        entities:
          - name: Rental
          - name: Vehicle
          - name: Customer

conventions:
  NamespaceRoot: Rent.Service
  policy:
    collision: fail
    insertTodoWhenMissing: true

templates:
  mapping:
    - artifact: Entity
      template: templates/dotnet/Entity.cs.hbs
      destination: "{Project}/Entities/{Name}.cs"

render:
  rules:
    - artifact: Entity
      selector: "contexts[*].aggregates[*].entities[*]"

apply:
  mode:
    artifact:
      kind: Entity
      context: Rent
```

#### 2. **FeatureManifest** - Complete Feature Generation

Used to generate multi-layer features (use cases, commands, queries).

```yaml
apiVersion: nettoolskit.io/v1
kind: FeatureManifest

meta:
  name: rental-use-cases
  description: Generate CQRS use cases for rental management

solution:
  root: samples/src
  slnFile: Rent.Service.sln

projects:
  - name: Rent.Service.Application
    path: Rent.Service.Application

contexts:
  - name: Rent
    features:
      - name: CreateRental
        type: command
        aggregate: Rental
      - name: GetRentalById
        type: Query
        aggregate: Rental

templates:
  mapping:
    - artifact: command
      template: templates/dotnet/command.cs.hbs
      destination: "{Project}/UseCases/{Context}/{Name}command.cs"
    - artifact: commandHandler
      template: templates/dotnet/commandHandler.cs.hbs
      destination: "{Project}/UseCases/{Context}/{Name}commandHandler.cs"

render:
  rules:
    - artifact: command
      selector: "contexts[*].features[?(@.type == 'command')]"
    - artifact: commandHandler
      selector: "contexts[*].features[?(@.type == 'command')]"

apply:
  mode:
    feature:
      context: Rent
      include: ["CreateRental", "GetRentalById"]
```

#### 3. **LayerManifest** - Complete Layer Generation

Used to generate entire layers (Application, Infrastructure).

```yaml
apiVersion: nettoolskit.io/v1
kind: LayerManifest

meta:
  name: infrastructure-layer
  description: Generate complete infrastructure layer

solution:
  root: samples/src
  slnFile: Rent.Service.sln

projects:
  - name: Rent.Service.Infrastructure
    path: Rent.Service.Infrastructure

contexts:
  - name: Rent
    aggregates:
      - name: Rental

templates:
  mapping:
    - artifact: Repository
      template: templates/dotnet/Repository.cs.hbs
      destination: "{Project}/Repositories/{Name}Repository.cs"
    - artifact: RepositoryInterface
      template: templates/dotnet/IRepository.cs.hbs
      destination: "{Domain}/Repositories/I{Name}Repository.cs"
    - artifact: DbContext
      template: templates/dotnet/DbContext.cs.hbs
      destination: "{Project}/Date/{Solution}DbContext.cs"

render:
  rules:
    - artifact: Repository
      selector: "contexts[*].aggregates[*]"
    - artifact: RepositoryInterface
      selector: "contexts[*].aggregates[*]"
    - artifact: DbContext
      selector: "contexts[0]"

apply:
  mode:
    layer:
      include: ["Infrastructure"]
```

---

## 🔧 Conventions and Variables

### Available Variables in Templates

All Handlebars templates have access to the following variables:

#### Context Variables
```handlebars
{{Name}}              {{!-- Entity/component name --}}
{{Context}}           {{!-- Bounded context name --}}
{{Aggregate}}         {{!-- Aggregate name --}}
{{NamespacePluralForm}}   {{!-- Namespace in plural --}}
{{Project}}           {{!-- .csproj project name --}}
{{Solution}}          {{!-- .sln solution name --}}
```

#### Convention Variables
```handlebars
{{NamespaceRoot}}     {{!-- Rent.Service --}}
{{TargetFramework}}   {{!-- net8.0 --}}
{{Author}}            {{!-- NetToolsKit Team --}}
{{Version}}           {{!-- 1.0.0 --}}
```

#### Helpers Handlebars
```handlebars
{{lower Name}}        {{!-- rental --}}
{{upper Name}}        {{!-- RENTAL --}}
{{pascal Name}}       {{!-- Rental --}}
{{camel Name}}        {{!-- rental --}}
{{plural Name}}       {{!-- Rentals --}}
{{snake Name}}        {{!-- rental_entity --}}
```

### Placeholders in Paths

Destination paths support variable substitution:

```yaml
destination: "{Project}/Entities/{Name}.cs"
# Result: Rent.Service.Domain/Entities/Rental.cs

destination: "{Project}/UseCases/{Context}/{Name}command.cs"
# Result: Rent.Service.Application/UseCases/Rent/CreateRentalcommand.cs
```

---

## 🧪 JSON Path Selectors

Manifests use JSON Path to select elements from context:

### Selector Examples

```yaml
# All entities from all aggregates
selector: "contexts[*].aggregates[*].entities[*]"

# Only command-type features
selector: "contexts[*].features[?(@.type == 'command')]"

# Aggregates from context "Rent"
selector: "contexts[?(@.name == 'Rent')].aggregates[*]"

# First entity of each aggregate
selector: "contexts[*].aggregates[*].entities[0]"

# Features with specific name
selector: "contexts[*].features[?(@.name == 'CreateRental')]"
```

### Supported Operators

- `[*]` - All elements
- `[0]`, `[1]` - Element by index
- `[?(@.prop == 'value')]` - Conditional filter
- `.` - Property navigation

---

## 📊 Implementation Status

### Commands

| Command | Status | Progress | Note |
|---------|--------|----------|------|
| `/list` | ✅ Implemented | 100% | Fully functional |
| `/check` | ⏳ In Development | 60% | Basic validation implemented |
| `/render` | 📋 Planned | 0% | Awaiting Phase 2.5 |
| `/new` | 📋 Planned | 0% | Awaiting Phase 2.5 |
| `/apply` | 🔄 In Development | 75% | Phase 2.4 in progress |
| `/quit` | ✅ Implemented | 100% | Fully functional |

### Features

| Feature | Status | Progress |
|---------|--------|----------|
| Manifest Parsing | ✅ | 100% |
| Template Registry | ✅ | 100% |
| Handlebars Engine | ✅ | 100% |
| JSON Path Selectors | ⏳ | 70% |
| Change Plan | ⏳ | 80% |
| Dry-run Preview | ⏳ | 60% |
| File Writing | ✅ | 100% |
| Collision Detection | ⏳ | 50% |
| TODO Injection | ✅ | 100% |
| Solution scaffolding | ⏳ | 40% |
| Post-steps | 📋 | 0% |

---

## 🚀 Next Steps

### Phase 2.4 (in progress)
- ✅ Manifest parsing and validation
- ✅ Template resolution
- ⏳ Change plan generation
- ⏳ Collision policies (fail Implemented, safe/force pending)
- ⏳ Dry-run with unified diff
- ⏳ Solution/project scaffolding
- ⏳ Complete integration tests

### Phase 2.5 (Planned)
- 📋 Async executor integration for `/apply`
- 📋 `/render` command implementation
- 📋 `/new` command implementation
- 📋 Post-steps execution framework
- 📋 Template hot-reload during development

### Phase 3.0 (Future)
- 📋 Plugin system for custom helpers
- 📋 Remote template registry
- 📋 Manifest authoring tooling (wizard)
- 📋 Cross-platform template families (Node.js, Python)

---

## 📚 References

- **Main Document**: `nettoolskit-cli.md`
- **Phase 2.4 Plan**: `task-phase-2.4-manifest-apply-dotnet.md`
- **Templates**: `tools/nettoolskit-cli/templates/dotnet/`
- **Example Manifests**: `tools/nettoolskit-cli/.docs/ntk-manifest-*.yml`
- **Source Code**: `tools/nettoolskit-cli/crates/commands/`

---