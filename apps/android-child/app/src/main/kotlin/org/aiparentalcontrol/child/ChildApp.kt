package org.aiparentalcontrol.child

import android.app.Application
import android.app.NotificationChannel
import android.app.NotificationManager

class ChildApp : Application() {

    override fun onCreate() {
        super.onCreate()
        createMonitoringChannel()
    }

    /**
     * A low-importance but always-visible channel for the persistent monitoring
     * notification. The channel must not be deletable by the user; the overt
     * notification is part of how this app stays a parental control tool and not
     * stalkerware.
     */
    private fun createMonitoringChannel() {
        val channel = NotificationChannel(
            MONITORING_CHANNEL_ID,
            getString(R.string.monitoring_channel_name),
            NotificationManager.IMPORTANCE_LOW,
        ).apply {
            description = getString(R.string.monitoring_channel_desc)
            setShowBadge(false)
        }
        getSystemService(NotificationManager::class.java).createNotificationChannel(channel)
    }

    companion object {
        const val MONITORING_CHANNEL_ID = "apc_monitoring"
    }
}
