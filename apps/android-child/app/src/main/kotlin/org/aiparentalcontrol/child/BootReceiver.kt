package org.aiparentalcontrol.child

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import androidx.core.content.ContextCompat

/** Restart overt monitoring after a reboot, but only if the device is paired. */
class BootReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action != Intent.ACTION_BOOT_COMPLETED) return
        // TODO(v0): check paired state before starting.
        ContextCompat.startForegroundService(context, Intent(context, MonitoringService::class.java))
    }
}
