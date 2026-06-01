$html = Invoke-RestMethod -Uri 'https://ely.by/u/Inn4teCube' -UseBasicParsing
if ($html -match 'og:image"\s+content="([^"]+)"') {
    Write-Host "Found URL: $($matches[1])"
} else {
    Write-Host "No og:image found."
}
