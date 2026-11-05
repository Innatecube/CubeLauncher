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
   3. Interactive 3D Minecraft Skin Model Playground
   Renders a 3D blocky Minecraft character with yaw, pitch, drag controls & animation.
   ========================================================================== */
function initSkinPlayground() {
  const canvas = document.getElementById('skinCanvas');
  if (!canvas) return;
  const ctx = canvas.getContext('2d');

  const yawSlider = document.getElementById('yawSlider');
  const pitchSlider = document.getElementById('pitchSlider');
  const yawVal = document.getElementById('yawVal');
  const pitchVal = document.getElementById('pitchVal');
  const resetBtn = document.getElementById('resetSkinBtn');
  const animBtn = document.getElementById('toggleAnimBtn');
  const modelBtns = document.querySelectorAll('.model-btn');

  let yaw = 25; // degrees
  let pitch = 12; // degrees
  let isDragging = false;
  let startX, startY;
  let isAnimated = true;
  let animTime = 0;
  let isSlim = false;

  // Set high DPI canvas resolution
  function resizeCanvas() {
    const rect = canvas.parentElement.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    ctx.scale(dpr, dpr);
    render();
  }
  window.addEventListener('resize', resizeCanvas);
  setTimeout(resizeCanvas, 50);

  // Mouse Drag handlers
  canvas.parentElement.addEventListener('mousedown', (e) => {
    isDragging = true;
    startX = e.clientX;
    startY = e.clientY;
  });

  window.addEventListener('mousemove', (e) => {
    if (!isDragging) return;
    const dx = e.clientX - startX;
    const dy = e.clientY - startY;
    startX = e.clientX;
    startY = e.clientY;

    yaw = (yaw + dx * 0.7) % 360;
    pitch = Math.max(-45, Math.min(45, pitch - dy * 0.5));

    updateSliderUI();
    render();
  });

  window.addEventListener('mouseup', () => {
    isDragging = false;
  });

  // Slider inputs
  if (yawSlider) {
    yawSlider.addEventListener('input', (e) => {
      yaw = parseFloat(e.target.value);
      if (yawVal) yawVal.textContent = Math.round(yaw) + '°';
      render();
    });
  }

  if (pitchSlider) {
    pitchSlider.addEventListener('input', (e) => {
      pitch = parseFloat(e.target.value);
      if (pitchVal) pitchVal.textContent = Math.round(pitch) + '°';
      render();
    });
  }

  function updateSliderUI() {
    if (yawSlider) yawSlider.value = ((yaw % 360) + 360) % 360;
    if (pitchSlider) pitchSlider.value = pitch;
    if (yawVal) yawVal.textContent = Math.round(((yaw % 360) + 360) % 360) + '°';
    if (pitchVal) pitchVal.textContent = Math.round(pitch) + '°';
  }

  if (resetBtn) {
    resetBtn.addEventListener('click', () => {
      yaw = 25;
      pitch = 12;
      updateSliderUI();
      render();
    });
  }

  if (animBtn) {
    animBtn.addEventListener('click', () => {
      isAnimated = !isAnimated;
      animBtn.classList.toggle('active', isAnimated);
    });
  }

  modelBtns.forEach(btn => {
    btn.addEventListener('click', () => {
      modelBtns.forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      isSlim = btn.getAttribute('data-model') === 'slim';
      render();
    });
  });

  // 3D Matrix & Box projection helper
  function render() {
    const rect = canvas.parentElement.getBoundingClientRect();
    const width = rect.width;
    const height = rect.height;

    ctx.clearRect(0, 0, width, height);

    // Subtle background grid
    ctx.strokeStyle = 'rgba(0, 229, 255, 0.05)';
    ctx.lineWidth = 1;
    const gridSize = 24;
    for (let x = 0; x < width; x += gridSize) {
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, height);
      ctx.stroke();
    }
    for (let y = 0; y < height; y += gridSize) {
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(width, y);
      ctx.stroke();
    }

    // Shadow circle on ground
    const groundY = height * 0.82;
    ctx.save();
    ctx.beginPath();
    ctx.ellipse(width / 2, groundY, 48, 14, 0, 0, Math.PI * 2);
    ctx.fillStyle = 'rgba(0, 0, 0, 0.55)';
    ctx.fill();
    ctx.restore();

    // 3D Rendering of Minecraft Character Cubes
    // Angles in radians
    const radYaw = (yaw * Math.PI) / 180;
    const radPitch = (pitch * Math.PI) / 180;
    const cosY = Math.cos(radYaw);
    const sinY = Math.sin(radYaw);
    const cosP = Math.cos(radPitch);
    const sinP = Math.sin(radPitch);

    const scale = Math.min(width, height) / 85;
    const centerX = width / 2;
    const centerY = height * 0.46;

    function project(x, y, z) {
      // Rotate Y (yaw)
      const x1 = x * cosY - z * sinY;
      const z1 = z * cosY + x * sinY;

      // Rotate X (pitch)
      const y2 = y * cosP - z1 * sinP;
      const z2 = z1 * cosP + y * sinP;

      return {
        x: centerX + x1 * scale,
        y: centerY - y2 * scale,
        z: z2,
      };
    }

    // List of character parts (boxes)
    // Minecraft scale: Head 8x8x8, Body 8x12x4, Arms 4x12x4 (or 3x12x4 for slim), Legs 4x12x4
    const armW = isSlim ? 3 : 4;
    const armOffset = isSlim ? 5.5 : 6;

    // Optional subtle idle breathing / limb sway
    const sway = isAnimated ? Math.sin(animTime * 2.5) * 0.12 : 0;
    const legSway = isAnimated ? Math.sin(animTime * 2.5) * 0.18 : 0;

    const parts = [
      // Head: center at y = 14
      {
        cx: 0, cy: 14, cz: 0,
        w: 8, h: 8, d: 8,
        colors: { top: '#453229', bottom: '#e0a982', front: '#e8b894', back: '#3d2b23', left: '#453229', right: '#453229' },
        overlayColor: '#1d222b', // Hair / mask overlay
      },
      // Torso: center at y = 4
      {
        cx: 0, cy: 4, cz: 0,
        w: 8, h: 12, d: 4,
        colors: { top: '#1c3144', bottom: '#1c3144', front: '#1b3a5b', back: '#152b44', left: '#1c3144', right: '#1c3144' },
      },
      // Left Arm (player's right): center at cx = -armOffset
      {
        cx: -armOffset, cy: 4, cz: sway * 6,
        w: armW, h: 12, d: 4,
        colors: { top: '#1c3144', bottom: '#e0a982', front: '#1e3852', back: '#152b44', left: '#1c3144', right: '#1c3144' },
      },
      // Right Arm (player's left): center at cx = armOffset
      {
        cx: armOffset, cy: 4, cz: -sway * 6,
        w: armW, h: 12, d: 4,
        colors: { top: '#1c3144', bottom: '#e0a982', front: '#1e3852', back: '#152b44', left: '#1c3144', right: '#1c3144' },
      },
      // Left Leg: cx = -2, cy = -8
      {
        cx: -2, cy: -8, cz: -legSway * 8,
        w: 4, h: 12, d: 4,
        colors: { top: '#1e242d', bottom: '#332924', front: '#19202a', back: '#141820', left: '#19202a', right: '#19202a' },
      },
      // Right Leg: cx = 2, cy = -8
      {
        cx: 2, cy: -8, cz: legSway * 8,
        w: 4, h: 12, d: 4,
        colors: { top: '#1e242d', bottom: '#332924', front: '#19202a', back: '#141820', left: '#19202a', right: '#19202a' },
      },
    ];

    // Collect all faces of all parts for painter's algorithm z-sorting
    const faces = [];

    parts.forEach(part => {
      const hw = part.w / 2;
      const hh = part.h / 2;
      const hd = part.d / 2;

      // 8 vertices for the box
      const v = [
        project(part.cx - hw, part.cy - hh, part.cz - hd), // 0: -x, -y, -z
        project(part.cx + hw, part.cy - hh, part.cz - hd), // 1: +x, -y, -z
        project(part.cx + hw, part.cy + hh, part.cz - hd), // 2: +x, +y, -z
        project(part.cx - hw, part.cy + hh, part.cz - hd), // 3: -x, +y, -z
        project(part.cx - hw, part.cy - hh, part.cz + hd), // 4: -x, -y, +z (front)
        project(part.cx + hw, part.cy - hh, part.cz + hd), // 5: +x, -y, +z
        project(part.cx + hw, part.cy + hh, part.cz + hd), // 6: +x, +y, +z
        project(part.cx - hw, part.cy + hh, part.cz + hd), // 7: -x, +y, +z
      ];

      // 6 faces: front (+z), back (-z), top (+y), bottom (-y), left (-x), right (+x)
      const boxFaces = [
        { pts: [v[4], v[5], v[6], v[7]], color: part.colors.front, shade: 1.0 },
        { pts: [v[1], v[0], v[3], v[2]], color: part.colors.back, shade: 0.65 },
        { pts: [v[7], v[6], v[2], v[3]], color: part.colors.top, shade: 1.25 },
        { pts: [v[0], v[1], v[5], v[4]], color: part.colors.bottom, shade: 0.55 },
        { pts: [v[0], v[4], v[7], v[3]], color: part.colors.left, shade: 0.8 },
        { pts: [v[5], v[1], v[2], v[6]], color: part.colors.right, shade: 0.9 },
      ];

      boxFaces.forEach(f => {
        // Compute face average Z depth
        const avgZ = (f.pts[0].z + f.pts[1].z + f.pts[2].z + f.pts[3].z) / 4;
        faces.push({ ...f, z: avgZ });
      });
    });

    // Sort back-to-front
    faces.sort((a, b) => a.z - b.z);

    // Draw sorted faces
    faces.forEach(face => {
      // Backface culling via cross product in screen space
      const v1x = face.pts[1].x - face.pts[0].x;
      const v1y = face.pts[1].y - face.pts[0].y;
      const v2x = face.pts[2].x - face.pts[0].x;
      const v2y = face.pts[2].y - face.pts[0].y;
      const cross = v1x * v2y - v1y * v2x;

      if (cross > 0) {
        ctx.beginPath();
        ctx.moveTo(face.pts[0].x, face.pts[0].y);
        ctx.lineTo(face.pts[1].x, face.pts[1].y);
        ctx.lineTo(face.pts[2].x, face.pts[2].y);
        ctx.lineTo(face.pts[3].x, face.pts[3].y);
        ctx.closePath();

        ctx.fillStyle = face.color;
        ctx.fill();

        // Edge stroke
        ctx.strokeStyle = 'rgba(0, 0, 0, 0.35)';
        ctx.lineWidth = 1;
        ctx.stroke();
      }
    });
  }

  // Animation Loop
  function animate() {
    if (isAnimated) {
      animTime += 0.025;
      render();
    }
    requestAnimationFrame(animate);
  }
  animate();
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
