import { createApp } from 'vue'
import './style.css'
import App from './App.vue'

createApp(App).mount('#app')

import { invoke } from '@tauri-apps/api'

// now we can call our Command!
// Right-click the application background and open the developer tools.
// You will see "Hello, World!" printed in the console!
invoke('startPdf', { path: 'World', folder: false })
  // `invoke` returns a Promise
  .then((response) => console.log(response))