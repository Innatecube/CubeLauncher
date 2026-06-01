$urls = @(
  'http://skinsystem.ely.by/avatar/Inn4teCube.png',
  'http://skinsystem.ely.by/avatar/Inn4teCube/128',
  'http://skinsystem.ely.by/helm/Inn4teCube/128',
  'http://skinsystem.ely.by/cube/Inn4teCube/128',
  'http://skins.ely.by/api/skins/Inn4teCube/face'
)
foreach ($u in $urls) {
  try {
    $res = Invoke-WebRequest -Uri $u -UseBasicParsing -TimeoutSec 5
    Write-Host "$u -> $($res.StatusCode)"
  } catch {
    Write-Host "$u -> Error: $($_.Exception.Message)"
  }
}
