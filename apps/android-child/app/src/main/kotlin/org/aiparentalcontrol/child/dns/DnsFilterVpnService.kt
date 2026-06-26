package org.aiparentalcontrol.child.dns

import android.net.VpnService
import android.content.Intent
import android.os.ParcelFileDescriptor

/**
 * On-device DNS filtering via a local VPN.
 *
 * VpnService is the Play-blessed mechanism for parental-control filtering. We
 * stand up a local TUN interface and route DNS so lookups can be checked against
 * the policy blocklists and forced to a family resolver with SafeSearch. No
 * traffic leaves the device for a remote server; this is local filtering only.
 *
 * Limits we accept (see docs/architecture.md): filtering is DNS/SNI level, not
 * deep HTTPS inspection (no TLS interception), and encrypted DNS (DoH/DoT) can
 * bypass plain DNS unless known endpoints are blocked.
 */
class DnsFilterVpnService : VpnService() {

    private var tunnel: ParcelFileDescriptor? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (tunnel == null) {
            tunnel = establishTunnel()
            // TODO(v0): start the DNS query loop: read packets from the tunnel,
            // parse DNS questions, block or rewrite per the policy, otherwise
            // forward to the family resolver. Detect and handle DoH/DoT.
        }
        return START_STICKY
    }

    private fun establishTunnel(): ParcelFileDescriptor? {
        val builder = Builder()
            .setSession("APC DNS filter")
            .addAddress("10.111.222.1", 32)
            // Capture DNS by pointing the system at our in-tunnel resolver.
            .addDnsServer("10.111.222.53")
            .addRoute("10.111.222.53", 32)
            .setBlocking(true)
        // Per-app routing is possible via addAllowedApplication / addDisallowedApplication.
        return builder.establish()
    }

    override fun onDestroy() {
        tunnel?.close()
        tunnel = null
        super.onDestroy()
    }
}
