/**
 * CubeLauncher Website Main JavaScript
 * Interactive components, 3D skin canvas playground, FAQ accordion, tabs, and OS detection.
 */

document.addEventListener('DOMContentLoaded', () => {
  initTabs();
  initFaqAccordion();
  initOsDetection();
  initSmoothScroll();
  initAdSidebar();
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
   3. Operating System Auto-Detection
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

/* ==========================================================================
   6. Advertisement Sidebar Controller
   ========================================================================== */
function initAdSidebar() {
  const adSidebar = document.getElementById('adSidebar');
  const adCloseBtn = document.getElementById('adCloseBtn');
  const adExpandBtn = document.getElementById('adExpandBtn');
  const adFallback = document.getElementById('adFallback');

  if (!adSidebar) return;

  // Determine initial state: user preference or auto-collapse on smaller screens
  const savedState = localStorage.getItem('cubelauncher_ad_collapsed');
  const isNarrowScreen = window.innerWidth < 1200;

  if (savedState === 'true' || (savedState === null && isNarrowScreen)) {
    adSidebar.classList.add('collapsed');
  }

  // Close / Collapse handler
  if (adCloseBtn) {
    adCloseBtn.addEventListener('click', () => {
      adSidebar.classList.add('collapsed');
      localStorage.setItem('cubelauncher_ad_collapsed', 'true');
    });
  }

  // Expand handler
  if (adExpandBtn) {
    adExpandBtn.addEventListener('click', () => {
      adSidebar.classList.remove('collapsed');
      localStorage.setItem('cubelauncher_ad_collapsed', 'false');
    });
  }

  // Observer to hide placeholder once AdSense renders an ad iframe
  if (adFallback) {
    const adIns = adSidebar.querySelector('ins.adsbygoogle');
    if (adIns) {
      const observer = new MutationObserver(() => {
        if (adIns.hasChildNodes() || adIns.getAttribute('data-ad-status') === 'filled') {
          adFallback.style.display = 'none';
        }
      });
      observer.observe(adIns, { childList: true, attributes: true });
    }
  }
}
