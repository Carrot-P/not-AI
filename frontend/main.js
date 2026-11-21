import './stub.js';
import { createDockview, themeLight } from 'dockview';
import 'dockview/dist/styles/dockview.css';
import './style.css';
import { composePanel, outputPanel, initCompose } from './compose.js';

const workspace = document.getElementById('workspace');
const dock = createDockview(workspace, {
  theme: themeLight,
  dndStrategy: 'pointer',
  disableFloatingGroups: true,
  defaultTabComponent: 'label',
  createTabComponent() {
    const element = document.createElement('span');
    return {
      element,
      init({ title }) { element.textContent = title; },
    };
  },
  createComponent({ name }) {
    if (name === 'compose') return composePanel();
    if (name === 'output') return outputPanel();
    if (name === 'editor') {
      const element = document.createElement('textarea');
      element.setAttribute('aria-label', 'Text editor');
      element.autofocus = true;
      element.spellcheck = false;
      return { element, init() {}, focus: () => element.focus() };
    }

    return { element: document.createElement('div'), init() {} };
  },
});

// These panels have no reopen command, so also suppress Dockview's close keys.
workspace.addEventListener('keydown', (event) => {
  if (event.target.matches('[role="tab"]') && ['Delete', 'Backspace'].includes(event.key)) {
    event.preventDefault();
    event.stopPropagation();
  }
}, { capture: true });

dock.layout(workspace.clientWidth, workspace.clientHeight);
dock.addPanel({ id: 'editor', component: 'editor', title: 'Editor', minimumWidth: 160 });
dock.addPanel({
  id: 'explorer',
  component: 'explorer',
  title: 'Explorer',
  position: { referencePanel: 'editor', direction: 'left' },
  initialWidth: 180,
  minimumWidth: 100,
  inactive: true,
});
dock.addPanel({
  id: 'compose', component: 'compose', title: 'Compose',
  position: { referencePanel: 'editor', direction: 'left' }, initialWidth: 340, minimumWidth: 260,
});
dock.addPanel({
  id: 'output', component: 'output', title: 'Output',
  position: { referencePanel: 'editor', direction: 'above' }, initialHeight: 420, minimumHeight: 160,
});
initCompose();
