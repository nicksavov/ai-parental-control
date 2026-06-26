package org.aiparentalcontrol.child.accessibility

import android.accessibilityservice.AccessibilityService
import android.view.accessibility.AccessibilityEvent

/**
 * Sideload ("deep") flavor only.
 *
 * Reads on-screen text so the on-device text AI can analyze conversations across
 * apps (the Bark/Adora-style coverage). Google Play classifies monitoring use of
 * AccessibilityService as ineligible, so this exists only in the sideload build,
 * and only after the parent enables it with a clear disclosure at setup.
 *
 * The text is analyzed in memory by the on-device pipeline (packages/ai); only a
 * structured alert leaves the device. Raw text is never stored or transmitted.
 */
class TextMonitorAccessibilityService : AccessibilityService() {

    override fun onAccessibilityEvent(event: AccessibilityEvent?) {
        event ?: return
        // TODO(v2): extract visible text from the event/window, hand it to the
        // on-device text pipeline (Stage-0 gate -> Stage-1 LLM), and emit an
        // alert via PairingManager.submitAlert if something fires. Never persist
        // or transmit the raw text.
    }

    override fun onInterrupt() {}
}
