#include <stdio.h>
#include <unistd.h>

int main() {
    printf("🚀 Hello from the Android-like Sandbox init process!\n");
    printf("PID: %d, UID: %d, GID: %d\n", getpid(), getuid(), getgid());
    
    char cwd[1024];
    if (getcwd(cwd, sizeof(cwd)) != NULL) {
        printf("CWD: %s\n", cwd);
    }
    
    printf("Sandbox environment is functional.\n");
    return 0;
}
