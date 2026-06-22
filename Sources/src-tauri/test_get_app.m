#import <Cocoa/Cocoa.h>

int main(int argc, const char * argv[]) {
    @autoreleasepool {
        NSString *path = [[NSWorkspace sharedWorkspace] fullPathForApplication:@"Terminal"];
        NSLog(@"Path: %@", path);
    }
    return 0;
}
