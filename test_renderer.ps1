$urls = @(
  'https://ely.by/services/skins-renderer?url=http://skinsystem.ely.by/skins/Inn4teCube.png&scale=10&renderFace=1',
  'https://ely.by/services/skins-renderer?url=http://skinsystem.ely.by/skins/Inn4teCube.png',
  'http://skinsystem.ely.by/skins/Inn4teCube.png'
)
foreach ($u in $urls) {
  try {
    $res = Invoke-WebRequest -Uri $u -UseBasicParsing -TimeoutSec 5
    Write-Host "$u -> $($res.StatusCode) (Size: $($res.RawContentLength))"
  } catch {
    Write-Host "$u -> Error: $($_.Exception.Message)"
  }
}
