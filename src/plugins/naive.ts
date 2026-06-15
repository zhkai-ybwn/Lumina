import {
  NButton,
  NMessageProvider,
  NDialogProvider,
  NNotificationProvider,
  NDrawer,
  NDrawerContent,
  NInput,
  NModal,
  NRadio,
  NRadioGroup,
} from 'naive-ui'
import type { App } from 'vue'

export default {
  install(app: App) {
    app.component('NButton', NButton)
    app.component('NMessageProvider', NMessageProvider)
    app.component('NDialogProvider', NDialogProvider)
    app.component('NNotificationProvider', NNotificationProvider)
    app.component('NDrawer', NDrawer)
    app.component('NDrawerContent', NDrawerContent)
    app.component('NInput', NInput)
    app.component('NModal', NModal)
    app.component('NRadio', NRadio)
    app.component('NRadioGroup', NRadioGroup)
  }
}
