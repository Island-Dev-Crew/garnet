/* Garnet seccomp-apply harness (UTM OS-sandbox-apply slice).
 *
 * Takes the syscall allowlist that `garnet sandbox --format json` GENERATES
 * (S46) and APPLIES it as a real seccomp filter (default action ERRNO(EPERM),
 * mirroring Garnet's SCMP_ACT_ERRNO), then demonstrates the trap on the running
 * kernel: an allowed syscall (getpid) succeeds; a DENIED syscall (socket — not in
 * the @caps(fs) policy) is deterministically refused with EPERM.
 *
 * This turns S46 from "policy generation only" into "applied + trapped on a real
 * Linux kernel". Honest scope: Linux seccomp only; it proves the GENERATED policy
 * is enforceable, not that the program is "safe". Build: cc -O2 -o seccomp_apply
 * seccomp_apply.c -lseccomp. Usage: seccomp_apply <allowlist-file> (one syscall
 * name per line).
 *
 * Post-load output uses raw write(2) (which the policy allows) and snprintf (pure
 * formatting, no syscalls), avoiding printf's stat-family calls whose names
 * (fstat/stat/lstat) are x86-centric and may not resolve on aarch64.
 */
#define _GNU_SOURCE
#include <seccomp.h>
#include <sys/socket.h>
#include <unistd.h>
#include <errno.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

static void out(const char *s) { (void)write(1, s, strlen(s)); }

int main(int argc, char **argv) {
    if (argc < 2) {
        out("usage: seccomp_apply <allowlist-file> [expect=denied|allowed]\n");
        return 2;
    }
    /* Expected outcome for the socket() probe: `denied` (the @caps(fs) trap, the
     * default) or `allowed` (the @caps(fs,net) policy-driven contrast). */
    int expect_denied = !(argc >= 3 && strcmp(argv[2], "allowed") == 0);
    FILE *f = fopen(argv[1], "r");
    if (!f) { perror("open allowlist"); return 2; }

    scmp_filter_ctx ctx = seccomp_init(SCMP_ACT_ERRNO(EPERM));
    if (!ctx) { out("seccomp_init failed\n"); return 2; }

    char line[128];
    int loaded = 0, unresolved = 0;
    char unresolved_names[512] = {0};
    while (fgets(line, sizeof line, f)) {
        line[strcspn(line, "\r\n")] = 0;
        if (line[0] == 0) continue;
        int nr = seccomp_syscall_resolve_name(line);
        if (nr == __NR_SCMP_ERROR) {
            unresolved++;
            if (strlen(unresolved_names) + strlen(line) + 2 < sizeof unresolved_names) {
                if (unresolved_names[0]) strcat(unresolved_names, ",");
                strcat(unresolved_names, line);
            }
            continue;
        }
        /* Always permit write/getpid/exit_group so the harness can report + exit
         * even if a minimal policy omitted them. */
        if (seccomp_rule_add(ctx, SCMP_ACT_ALLOW, nr, 0) == 0) loaded++;
    }
    fclose(f);
    for (const char *need[] = {"write", "getpid", "exit_group", "exit", 0}, **p = need; *p; p++) {
        int nr = seccomp_syscall_resolve_name(*p);
        if (nr != __NR_SCMP_ERROR) seccomp_rule_add(ctx, SCMP_ACT_ALLOW, nr, 0);
    }

    char pre[256];
    snprintf(pre, sizeof pre,
             "seccomp-apply: applied %d allowed syscalls (default ERRNO); "
             "%d names unresolved on this arch%s%s\n",
             loaded, unresolved, unresolved ? ": " : "", unresolved_names);
    out(pre);

    if (seccomp_load(ctx) != 0) { out("seccomp_load failed\n"); return 2; }
    seccomp_release(ctx);

    /* --- under the applied filter --- */
    long pid = (long)getpid();           /* allowed */
    errno = 0;
    int s = socket(AF_INET, SOCK_STREAM, 0); /* denied for @caps(fs) */
    int se = errno;
    int blocked = (s < 0 && se == EPERM);
    if (s >= 0) close(s);

    char buf[512];
    snprintf(buf, sizeof buf,
             "seccomp-apply: allowed getpid() -> %s (pid=%ld)\n"
             "seccomp-apply: socket(AF_INET) -> %s (ret=%d errno=%d/%s); expected %s\n",
             pid > 0 ? "OK" : "FAIL", pid,
             blocked ? "BLOCKED (trap)" : "ALLOWED", s, se, strerror(se),
             expect_denied ? "DENIED" : "ALLOWED");
    out(buf);

    int as_expected = (expect_denied == blocked);
    if (pid > 0 && as_expected) {
        out(expect_denied
                ? "seccomp-apply: PROVEN -- the generated policy is APPLIED and TRAPS a "
                  "denied syscall on this kernel.\n"
                : "seccomp-apply: PROVEN -- the generated policy is APPLIED and ALLOWS the "
                  "declared syscall (policy-driven, not blanket-deny).\n");
        return 0;
    }
    out("seccomp-apply: FAILED -- the applied policy did not match the expected outcome.\n");
    return 1;
}
