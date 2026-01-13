Write-Host "ADVERTENCIA: Este script reescribirá el historial de Git para eliminar permanentemente:" -ForegroundColor Red
Write-Host " - diagnostic.log" -ForegroundColor Yellow
Write-Host " - proposition.md" -ForegroundColor Yellow
Write-Host ""
Write-Host "Esto es destructivo y no se puede deshacer fácilmente."
Write-Host "Asegúrate de tener un backup si no estás seguro."
Write-Host ""
$confirmation = Read-Host "¿Deseas continuar? (Escribe 'SI' para confirmar)"

if ($confirmation -eq 'SI') {
    Write-Host "Iniciando limpieza del historial..." -ForegroundColor Cyan
    git filter-branch --force --index-filter "git rm --cached --ignore-unmatch diagnostic.log proposition.md" --prune-empty --tag-name-filter cat -- --all
    
    Write-Host ""
    Write-Host "Limpieza completada localmente." -ForegroundColor Green
    Write-Host "Para aplicar los cambios en GitHub, debes ejecutar manualmente:" -ForegroundColor Yellow
    Write-Host "git push origin --force --all" -ForegroundColor White
} else {
    Write-Host "Operación cancelada." -ForegroundColor Yellow
}
