import {
  NButton,
  NCheckbox,
  NMessageProvider,
  NDialogProvider,
  NNotificationProvider,
  NDrawer,
  NDrawerContent,
  NDropdown,
  NInput,
  NModal,
  NRadio,
  NRadioGroup,
  NSelect,
} from 'naive-ui'
import type { App } from 'vue'

export default {
  install(app: App) {
    app.component('NButton', NButton)
    app.component('NCheckbox', NCheckbox)
    app.component('NMessageProvider', NMessageProvider)
    app.component('NDialogProvider', NDialogProvider)
    app.component('NNotificationProvider', NNotificationProvider)
    app.component('NDrawer', NDrawer)
    app.component('NDrawerContent', NDrawerContent)
    app.component('NDropdown', NDropdown)
    app.component('NInput', NInput)
    app.component('NModal', NModal)
    app.component('NRadio', NRadio)
    app.component('NRadioGroup', NRadioGroup)
    app.component('NSelect', NSelect)
  }
}
