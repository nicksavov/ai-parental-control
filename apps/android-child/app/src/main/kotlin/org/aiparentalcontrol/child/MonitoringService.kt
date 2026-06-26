package org.aiparentalcontrol.child

import android.app.Notification
import android.app.Service
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import androidx.core.app.NotificationCompat

/**
 * The always-on, overt monitoring service.
 *
 * This is the overt-only invariant in code. While monitoring or enforcement is
 * active, the child device shows a persistent, non-dismissable notification and
 * a visible app icon. There is no flag, debug mode, or setting that hides it.
 * Removing or weakening this notification is a release blocker (see
 * compliance/store-submission-checklist.md) and must fail CI.
 */
class MonitoringService : Service() {

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        startForegroundOvert()
        // TODO(v0): start UsageReporter sampling and ensure the DNS filter VPN is
        // running. TODO(v1+): wire the on-device AI pipeline and alert relay.
        return START_STICKY
    }

    private fun startForegroundOvert() {
        val notification: Notification = NotificationCompat.Builder(this, ChildApp.MONITORING_CHANNEL_ID)
            .setSmallIcon(R.drawable.ic_monitoring)
            .setContentTitle(getString(R.string.monitoring_active_title))
            .setContentText(getString(R.string.monitoring_active_text))
            .setOngoing(true) // cannot be swiped away
            .setShowWhen(false)
            .setCategory(NotificationCompat.CATEGORY_SERVICE)
            .setPriority(NotificationCompat.PRIORITY_LOW)
            .build()

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
            startForeground(NOTIFICATION_ID, notification, ServiceInfo.FOREGROUND_SERVICE_TYPE_SPECIAL_USE)
        } else {
            startForeground(NOTIFICATION_ID, notification)
        }
    }

    override fun onBind(intent: Intent?): IBinder? = null

    companion object {
        private const val NOTIFICATION_ID = 1001
    }
}
