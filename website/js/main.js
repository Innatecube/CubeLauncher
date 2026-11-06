/**
 * CubeLauncher Website Main JavaScript
 * Interactive components, 3D skin canvas playground, FAQ accordion, tabs, and OS detection.
 */

document.addEventListener('DOMContentLoaded', () => {
  initTabs();
  initFaqAccordion();
  initSkinPlayground();
  initOsDetection();
  initSmoothScroll();
});

/* ==========================================================================
   1. Interactive Feature Showcase Tabs
   ========================================================================== */
function initTabs() {
  const tabBtns = document.querySelectorAll('.tab-btn');
  const tabContents = document.querySelectorAll('.tab-content');

  tabBtns.forEach(btn => {
    btn.addEventListener('click', () => {
      const targetId = btn.getAttribute('data-tab');

      // Update button active state
      tabBtns.forEach(b => b.classList.remove('active'));
      btn.classList.add('active');

      // Update tab contents
      tabContents.forEach(content => {
        if (content.id === targetId) {
          content.classList.add('active');
        } else {
          content.classList.remove('active');
        }
      });
    });
  });
}

/* ==========================================================================
   2. FAQ Accordion (NoRisk Style with + toggle)
   ========================================================================== */
function initFaqAccordion() {
  const faqItems = document.querySelectorAll('.faq-item');

  faqItems.forEach(item => {
    const question = item.querySelector('.faq-question');
    const answer = item.querySelector('.faq-answer');

    question.addEventListener('click', () => {
      const isOpen = item.classList.contains('open');

      // Optional: Close all others for single-open behavior
      faqItems.forEach(otherItem => {
        if (otherItem !== item && otherItem.classList.contains('open')) {
          otherItem.classList.remove('open');
          const otherAnswer = otherItem.querySelector('.faq-answer');
          otherAnswer.style.maxHeight = null;
        }
      });

      if (isOpen) {
        item.classList.remove('open');
        answer.style.maxHeight = null;
      } else {
        item.classList.add('open');
        answer.style.maxHeight = answer.scrollHeight + 'px';
      }
    });
  });
}

/* ==========================================================================
   3. Interactive 3D Minecraft Skin Model Playground (skinview3d WebGL)
   Hardware-accelerated real 3D depth-tested Minecraft player skin renderer.
   ========================================================================== */
function initSkinPlayground() {
  const canvas = document.getElementById('skinCanvas');
  if (!canvas) return;

  const container = canvas.parentElement;
  const yawSlider = document.getElementById('yawSlider');
  const pitchSlider = document.getElementById('pitchSlider');
  const yawVal = document.getElementById('yawVal');
  const pitchVal = document.getElementById('pitchVal');
  const resetBtn = document.getElementById('resetSkinBtn');
  const animBtn = document.getElementById('toggleAnimBtn');
  const modelBtns = document.querySelectorAll('.model-btn');

  if (typeof skinview3d === 'undefined') {
    console.warn('skinview3d not loaded.');
    return;
  }

  const rect = container.getBoundingClientRect();
  const initW = Math.max(300, rect.width || 360);
  const initH = Math.max(340, rect.height || 380);

  // Initialize hardware-accelerated 3D WebGL skin viewer
  const viewer = new skinview3d.SkinViewer({
    canvas: canvas,
    width: initW,
    height: initH,
    skin: 'images/skin.png',
    model: 'default',
    alpha: true,
  });

  // Lighting & Camera configuration
  viewer.camera.position.set(0, 0, 68);
  viewer.playerObject.rotation.y = 0.45; // ~25 degrees
  viewer.playerObject.rotation.x = 0.12; // ~7 degrees
  viewer.globalLight.intensity = 1.7;
  viewer.cameraLight.intensity = 1.0;

  // Orbit drag controls with natural friction
  viewer.controls.enableRotate = true;
  viewer.controls.enableZoom = false;
  viewer.controls.enablePan = false;

  // Gentle, realistic idle breathing / limb sway
  let idleAnim = new skinview3d.IdleAnimation();
  idleAnim.speed = 0.6;
  viewer.animation = idleAnim;
  let isAnimated = true;

  // Handle responsive resize
  function resizeViewer() {
    if (!container) return;
    const w = container.clientWidth;
    const h = container.clientHeight;
    if (w > 50 && h > 50) {
      viewer.setSize(w, h);
    }
  }
  window.addEventListener('resize', resizeViewer);
  setTimeout(resizeViewer, 100);

  // Sync sliders when dragging 3D model
  viewer.controls.addEventListener('change', () => {
    const rotY = (viewer.playerObject.rotation.y * 180 / Math.PI) % 360;
    const rotX = (viewer.playerObject.rotation.x * 180 / Math.PI);
    const cleanY = Math.round(((rotY % 360) + 360) % 360);
    const cleanX = Math.round(Math.max(-45, Math.min(45, rotX)));
    if (yawSlider) yawSlider.value = cleanY;
    if (yawVal) yawVal.textContent = cleanY + '°';
    if (pitchSlider) pitchSlider.value = cleanX;
    if (pitchVal) pitchVal.textContent = cleanX + '°';
  });

  // Manual Slider inputs
  if (yawSlider) {
    yawSlider.addEventListener('input', (e) => {
      const deg = parseFloat(e.target.value);
      viewer.playerObject.rotation.y = (deg * Math.PI) / 180;
      if (yawVal) yawVal.textContent = Math.round(deg) + '°';
    });
  }

  if (pitchSlider) {
    pitchSlider.addEventListener('input', (e) => {
      const deg = parseFloat(e.target.value);
      viewer.playerObject.rotation.x = (deg * Math.PI) / 180;
      if (pitchVal) pitchVal.textContent = Math.round(deg) + '°';
    });
  }

  // Model geometry toggle (Classic 4px vs Slim 3px)
  modelBtns.forEach(btn => {
    btn.addEventListener('click', () => {
      modelBtns.forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      const isSlim = btn.getAttribute('data-model') === 'slim';
      viewer.loadSkin('images/skin.png', { model: isSlim ? 'slim' : 'default' });
    });
  });

  // Animation Toggle
  if (animBtn) {
    animBtn.addEventListener('click', () => {
      isAnimated = !isAnimated;
      animBtn.classList.toggle('active', isAnimated);
      animBtn.textContent = isAnimated ? 'Animation: ON' : 'Animation: OFF';
      viewer.animation = isAnimated ? idleAnim : null;
    });
  }

  // Reset Camera
  if (resetBtn) {
    resetBtn.addEventListener('click', () => {
      viewer.playerObject.rotation.y = 0.45;
      viewer.playerObject.rotation.x = 0.12;
      viewer.playerObject.rotation.z = 0;
      viewer.camera.position.set(0, 0, 68);
      viewer.camera.lookAt(0, 0, 0);
      if (yawSlider) yawSlider.value = 25;
      if (yawVal) yawVal.textContent = '25°';
      if (pitchSlider) pitchSlider.value = 7;
      if (pitchVal) pitchVal.textContent = '7°';
    });
  }
}

/* ==========================================================================
   4. Operating System Auto-Detection
   ========================================================================== */
function initOsDetection() {
  const downloadBtn = document.getElementById('primaryDownloadBtn');
  const osBadge = document.getElementById('detectedOsBadge');
  if (!downloadBtn) return;

  const userAgent = window.navigator.userAgent.toLowerCase();

  if (userAgent.includes('win')) {
    if (osBadge) osBadge.textContent = 'Detected Windows (64-bit)';
    downloadBtn.setAttribute('href', 'downloads/CubeLauncher-Setup.exe');
  } else if (userAgent.includes('mac')) {
    if (osBadge) osBadge.textContent = 'Detected macOS (Coming Soon)';
  } else if (userAgent.includes('linux')) {
    if (osBadge) osBadge.textContent = 'Detected Linux (Coming Soon)';
  }
}

/* ==========================================================================
   5. Smooth Scroll for Anchor Links
   ========================================================================== */
function initSmoothScroll() {
  document.querySelectorAll('a[href^="#"]').forEach(anchor => {
    anchor.addEventListener('click', function (e) {
      const targetId = this.getAttribute('href');
      if (targetId === '#') return;
      const targetElement = document.querySelector(targetId);
      if (targetElement) {
        e.preventDefault();
        targetElement.scrollIntoView({
          behavior: 'smooth',
          block: 'start'
        });
      }
    });
  });
}
