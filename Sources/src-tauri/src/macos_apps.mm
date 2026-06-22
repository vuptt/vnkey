#import <Cocoa/Cocoa.h>

extern "C" {
    char* macos_get_frontmost_app_bundle_id() {
        @autoreleasepool {
            NSRunningApplication* app = [[NSWorkspace sharedWorkspace] frontmostApplication];
            if (!app) return NULL;
            NSString* bundleId = [app bundleIdentifier];
            if (!bundleId) return NULL;
            return strdup([bundleId UTF8String]);
        }
    }

    static NSString* getBase64Icon(NSImage* icon) {
        if (!icon) return @"";
        NSSize newSize = NSMakeSize(64, 64);
        NSImage *smallIcon = [NSImage imageWithSize:newSize flipped:NO drawingHandler:^BOOL(NSRect dstRect) {
            [icon drawInRect:dstRect fromRect:NSZeroRect operation:NSCompositingOperationCopy fraction:1.0];
            return YES;
        }];
        
        NSData *tiffData = [smallIcon TIFFRepresentation];
        if (tiffData) {
            NSBitmapImageRep *bitmap = [NSBitmapImageRep imageRepWithData:tiffData];
            if (bitmap) {
                NSData *pngData = [bitmap representationUsingType:NSBitmapImageFileTypePNG properties:@{}];
                if (pngData) {
                    return [pngData base64EncodedStringWithOptions:0];
                }
            }
        }
        return @"";
    }

    char* macos_get_running_applications_json() {
        @autoreleasepool {
            NSArray<NSRunningApplication*>* apps = [[NSWorkspace sharedWorkspace] runningApplications];
            NSMutableArray* array = [NSMutableArray array];
            for (NSRunningApplication* app in apps) {
                if (app.activationPolicy == NSApplicationActivationPolicyRegular) {
                    NSString* bundleId = app.bundleIdentifier;
                    NSString* name = app.localizedName;
                    if (bundleId && name) {
                        NSImage* icon = app.icon;
                        NSString* iconBase64 = getBase64Icon(icon);
                        [array addObject:@{
                            @"bundle_id": bundleId,
                            @"name": name,
                            @"icon": iconBase64
                        }];
                    }
                }
            }
            NSData* jsonData = [NSJSONSerialization dataWithJSONObject:array options:0 error:nil];
            NSString* jsonString = [[NSString alloc] initWithData:jsonData encoding:NSUTF8StringEncoding];
            return strdup([jsonString UTF8String]);
        }
    }

    char* macos_get_application_info_by_path_json(const char* path_cstr) {
        @autoreleasepool {
            NSString* path = [NSString stringWithUTF8String:path_cstr];
            NSBundle* bundle = [NSBundle bundleWithPath:path];
            if (!bundle) return NULL;
            NSString* bundleId = [bundle bundleIdentifier];
            NSString* name = [bundle objectForInfoDictionaryKey:@"CFBundleDisplayName"];
            if (!name) name = [bundle objectForInfoDictionaryKey:@"CFBundleName"];
            if (!name) name = [[path lastPathComponent] stringByDeletingPathExtension];
            
            NSImage* icon = [[NSWorkspace sharedWorkspace] iconForFile:path];
            NSString* iconBase64 = getBase64Icon(icon);
            
            NSDictionary* dict = @{
                @"bundle_id": bundleId ? bundleId : @"",
                @"name": name ? name : @"",
                @"icon": iconBase64
            };
            NSData* jsonData = [NSJSONSerialization dataWithJSONObject:dict options:0 error:nil];
            NSString* jsonString = [[NSString alloc] initWithData:jsonData encoding:NSUTF8StringEncoding];
            return strdup([jsonString UTF8String]);
        }
    }

    char* macos_get_application_info_by_bundle_id_json(const char* bundle_id_cstr) {
        @autoreleasepool {
            NSString* bundleId = [NSString stringWithUTF8String:bundle_id_cstr];
            NSURL* url = [[NSWorkspace sharedWorkspace] URLForApplicationWithBundleIdentifier:bundleId];
            if (!url) return NULL;
            NSString* path = [url path];
            if (!path) return NULL;
            return macos_get_application_info_by_path_json([path UTF8String]);
        }
    }

#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wdeprecated-declarations"
    char* macos_get_application_info_by_name_json(const char* name_cstr) {
        @autoreleasepool {
            NSString* name = [NSString stringWithUTF8String:name_cstr];
            NSString* path = [[NSWorkspace sharedWorkspace] fullPathForApplication:name];
            if (!path) return NULL;
            return macos_get_application_info_by_path_json([path UTF8String]);
        }
    }
#pragma clang diagnostic pop
}
