// data sources:
// - type
//   - f: /sys/class/net/$IFACE/wireless/ --> present if wireless
// - status:
//   - f: /proc/net/wireless
//        "The cfg80211 wext compat layer assumes a maximum quality of 70"
//        -- https://git.openwrt.org/?p=project/iwinfo.git;a=blob;f=iwinfo_nl80211.c
//   - f: /sys/class/net/$IFACE/operstate --> up | down
//   - f: /proc/net/fib_trie
//   - f: /proc/net/route
//   - c: ip route list
//   - l: libnetlink
//   - l: https://github.com/achanda/netlink
// - traffic:
//   - f: /proc/net/dev
// - SSID:
//   - c: iwconfig
//   - c: iwgetid
// - TCP buffers
//  - f: /proc/net/tcp
//      > The "tx_queue" and "rx_queue" are the
//      > outgoing and incoming data queue
//      > in terms of kernel memory usage.

pub mod if_operstate;
pub mod wifi_link_qual;
