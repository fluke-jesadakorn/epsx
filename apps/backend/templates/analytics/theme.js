(function () {
  const d = document;
  const prefersReduced = window.matchMedia(
    '(prefers-reduced-motion: reduce)'
  ).matches;

  const qs = (sel, root = d) => root.querySelector(sel);
  const qsa = (sel, root = d) => Array.from(root.querySelectorAll(sel));

  function focusFirst() {
    const el =
      d.querySelector('[data-autofocus]') ||
      d.querySelector('.metro-input') ||
      d.querySelector('input,select,textarea,button');
    if (el) {
      try {
        el.focus();
      } catch (_) {}
    }
  }

  function initAutofocus() {
    const explicit = qs('[data-autofocus]');
    if (explicit) {
      try {
        explicit.focus();
      } catch (_) {}
      return;
    }
    const firstInput = qs('input,select,textarea,button');
    if (firstInput) {
      try {
        firstInput.focus();
      } catch (_) {}
    }
  }

  function attachPasswordToggles() {
    qsa('[data-toggle="password"]').forEach(btn => {
      const targetSel = btn.getAttribute('data-target');
      const input = targetSel ? qs(targetSel) : null;
      if (!input) return;
      btn.addEventListener('click', () => {
        input.type = input.type === 'password' ? 'text' : 'password';
        // Toggle emoji for visibility
        const isHidden = input.type === 'password';
        btn.textContent = isHidden ? '👁️' : '🙈';
      });
    });
  }

  function handleFormLoading() {
    qsa('form').forEach(form => {
      form.addEventListener('submit', () => {
        const loading = qs('#loading');
        if (loading) loading.style.display = 'block';
        const submit = form.querySelector(
          'button[type="submit"], .metro-button'
        );
        if (submit) {
          submit.disabled = true;
          if (!submit.dataset.originalText)
            submit.dataset.originalText = submit.innerHTML;
          submit.innerHTML = '⏳';
          submit.style.opacity = '0.7';
        }
      });
    });
  }

  // Register form client validation to preserve prior UX
  function enhanceRegisterForm() {
    const form = qs('#register-form');
    if (!form) return;
    form.addEventListener('submit', e => {
      const pw = qs('#password');
      const confirm = qs('#confirm-password');
      if (pw && confirm && pw.value !== confirm.value) {
        e.preventDefault();
        // Preserve minimal alert emoji UX without wording changes
        // Original used: alert('🔐 ❌');
        try {
          alert('🔐 ❌');
        } catch (_) {}
        return;
      }
    });
  }

  function animateStatusTiles() {
    if (prefersReduced) return;
    const tiles = qsa('.metro-tile.small');
    tiles.forEach((tile, index) => {
      setTimeout(() => {
        tile.style.transform = 'scale(1.05) rotate(2deg)';
        setTimeout(() => {
          tile.style.transform = '';
        }, 500);
      }, index * 150);
    });
  }

  function cycleSystemStatusBar() {
    const fill = qs('.status-bar .fill') || qs('[data-status-fill]');
    if (!fill || prefersReduced) return;
    const widths = ['87%', '89%', '85%', '91%', '88%'];
    let i = 0;
    setInterval(() => {
      fill.style.width = widths[i % widths.length];
      i++;
    }, 3000);
  }

  function emailValidation() {
    const emailInputs = qsa('#email, #recovery_email');
    emailInputs.forEach(inp => {
      inp.addEventListener('input', () => {
        const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
        if (emailRegex.test(inp.value)) {
          inp.classList.add('success');
          inp.classList.remove('error');
        } else {
          inp.classList.remove('success');
        }
      });
    });
  }

  function passwordStrengthAndMatch() {
    const pw = qs('#newPassword');
    const confirm = qs('#confirmPassword');
    const bar = qs('#strength-bar');
    const txt = qs('#strength-text');
    const matchDiv = qs('#password-match');
    const submit = qs('#confirm-button');

    function updateStrength() {
      if (!pw || !bar || !txt) return;
      const value = pw.value || '';
      let s = 0;
      const hasLength = value.length >= 8;
      const hasCase = /[a-z]/.test(value) && /[A-Z]/.test(value);
      const hasNumber = /\d/.test(value);
      const hasSpecial = /[^A-Za-z0-9]/.test(value);
      if (hasLength) s++;
      if (hasCase) s++;
      if (hasNumber) s++;
      if (hasSpecial) s++;
      let color = '#D83B01',
        width = '25%',
        feedback = '⚠️';
      if (s === 2) {
        color = '#FF8C42';
        width = '50%';
        feedback = '📊';
      }
      if (s === 3) {
        color = '#DAA520';
        width = '75%';
        feedback = '✨';
      }
      if (s === 4) {
        color = '#107C10';
        width = '100%';
        feedback = '🛡️';
      }
      bar.style.background = color;
      bar.style.width = width;
      txt.textContent = feedback;
    }

    function updateMatch() {
      if (!pw || !confirm || !matchDiv || !submit) return;
      if (confirm.value.length === 0) {
        matchDiv.textContent = '';
        submit.disabled = true;
        submit.style.opacity = '0.5';
        return;
      }
      if (pw.value === confirm.value && pw.value.length >= 8) {
        matchDiv.textContent = '✅';
        matchDiv.style.color = '#107C10';
        submit.disabled = false;
        submit.style.opacity = '1';
      } else {
        matchDiv.textContent = '❌';
        matchDiv.style.color = '#D83B01';
        submit.disabled = true;
        submit.style.opacity = '0.5';
      }
    }

    if (pw)
      pw.addEventListener('input', () => {
        updateStrength();
        updateMatch();
      });
    if (confirm) confirm.addEventListener('input', updateMatch);
    updateStrength();
    updateMatch();
  }

  function celebrateSuccess() {
    if (prefersReduced) return;
    const tiles = qsa('#celebration-tiles .metro-tile');
    if (tiles.length === 0) return;
    tiles.forEach((tile, index) => {
      setTimeout(() => {
        tile.style.transform = 'scale(1.1)';
        setTimeout(() => {
          tile.style.transform = '';
        }, 300);
      }, index * 150);
    });
  }

  d.addEventListener('DOMContentLoaded', () => {
    focusFirst();
    initAutofocus();
    attachPasswordToggles();
    handleFormLoading();
    enhanceRegisterForm();
    cycleSystemStatusBar();
    emailValidation();
    passwordStrengthAndMatch();
    celebrateSuccess();

    if (!prefersReduced) {
      setTimeout(animateStatusTiles, 600);
    }
  });
})();
