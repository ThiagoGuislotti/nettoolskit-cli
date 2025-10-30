# Script para testar o NetToolsKit CLI de forma interativa
Write-Host "Testando NetToolsKit CLI - pressione '/' para ver os comandos"
Write-Host "Use '/quit' para sair ou Ctrl+C"
Write-Host ""

& "$PSScriptRoot\target\debug\ntk.exe"