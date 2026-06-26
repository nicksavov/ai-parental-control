package org.aiparentalcontrol.child.usage

import android.app.usage.UsageStatsManager
import android.content.Context

/**
 * Screen-time reporting from UsageStatsManager. This is the policy-clean basis
 * for screen time on Android; it is what Digital Wellbeing itself uses. Requires
 * the PACKAGE_USAGE_STATS special access, granted by the parent in Settings.
 */
class UsageReporter(private val context: Context) {

    data class AppUsage(val packageName: String, val foregroundMillis: Long)

    /** Foreground time per app for the window [since, now]. */
    fun usageSince(since: Long): List<AppUsage> {
        val usm = context.getSystemService(Context.USAGE_STATS_SERVICE) as UsageStatsManager
        val now = System.currentTimeMillis()
        return usm.queryUsageStats(UsageStatsManager.INTERVAL_DAILY, since, now)
            .filter { it.totalTimeInForeground > 0 }
            .map { AppUsage(it.packageName, it.totalTimeInForeground) }
            .sortedByDescending { it.foregroundMillis }
    }

    /**
     * The current foreground package, sampled from the event stream. This is the
     * detection used to enforce per-app limits and bedtime windows by drawing the
     * overlay lock when an over-limit or blocked app comes to the front.
     */
    fun currentForegroundPackage(windowMillis: Long = 10_000): String? {
        val usm = context.getSystemService(Context.USAGE_STATS_SERVICE) as UsageStatsManager
        val now = System.currentTimeMillis()
        val events = usm.queryEvents(now - windowMillis, now)
        val event = android.app.usage.UsageEvents.Event()
        var last: String? = null
        while (events.hasNextEvent()) {
            events.getNextEvent(event)
            if (event.eventType == android.app.usage.UsageEvents.Event.MOVE_TO_FOREGROUND) {
                last = event.packageName
            }
        }
        return last
    }
}
