package org.aiparentalcontrol.child.ui

import android.app.Activity
import android.content.Intent
import android.net.VpnService
import android.os.Bundle
import androidx.core.content.ContextCompat
import org.aiparentalcontrol.child.MonitoringService

/**
 * Overt setup. Before anything runs, this screen tells the child and the
 * installing parent exactly what is monitored, asks for consent, then walks
 * through the grants (notifications, usage access, VPN consent) and pairing.
 *
 * This UI is the "transparency" half of being a parental control tool rather
 * than spyware. It must never be skippable into a hidden state.
 */
class SetupActivity : Activity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // TODO(v0): show the disclosure + consent screen, then:
        //   1. request POST_NOTIFICATIONS
        //   2. send the user to Settings to grant Usage Access
        //   3. request VPN consent (below)
        //   4. scan the parent's pairing QR and call PairingManager.pair(...)
        //   5. start overt monitoring
    }

    private fun requestVpnConsent() {
        val intent = VpnService.prepare(this)
        if (intent != null) {
            startActivityForResult(intent, REQ_VPN)
        } else {
            onVpnReady()
        }
    }

    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        if (requestCode == REQ_VPN && resultCode == RESULT_OK) onVpnReady()
    }

    private fun onVpnReady() {
        ContextCompat.startForegroundService(this, Intent(this, MonitoringService::class.java))
    }

    companion object {
        private const val REQ_VPN = 1
    }
}
