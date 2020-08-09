import UIKit
import KeychainSwift

@UIApplicationMain
class AppDelegate: UIResponder, UIApplicationDelegate {
    var window: UIWindow?
    var inForeground: Bool = true

    func application(_ application: UIApplication, didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?) -> Bool {
        HttpUtils.copyCookiesToKeychain()

        ExocoreUtils.initialize()

        let websocketBridgeFactory = RealWebSocketBridgeFactory()
        let ajaxBridgeFactory = RealXMLHttpRequestBridgeFactory()
        JSBridge.instance = JSBridge(serverHost: "exomind.io", webSocketBridgeFactory: websocketBridgeFactory, ajaxBridgeFactory: ajaxBridgeFactory)

        // see https://github.com/tokio-rs/mio/issues/949
        signal(SIGPIPE, SIG_IGN)

        return true
    }

    func application(_ application: UIApplication, performActionFor shortcutItem: UIApplicationShortcutItem, completionHandler: @escaping (Bool) -> Void) {
        func openObject(_ entity: EntityExt?) {
            if let entity = entity {
                RootNavigationController.mainInstance()?.show(navigationObject: .entity(entity: entity))
            }
        }

        if (shortcutItem.type.contains("NewNote")) {
            EntityCreationViewController.createNote(nil, callback: openObject)
        } else if (shortcutItem.type.contains("NewTask")) {
            EntityCreationViewController.createTask(nil, callback: openObject)
        } else if (shortcutItem.type.contains("NewEmail")) {
            EntityCreationViewController.createEmail(nil, callback: openObject)
        } else if (shortcutItem.type.contains("NewCollection")) {
            EntityCreationViewController.createCollection(nil, callback: openObject)
        }
    }

    func application(_ application: UIApplication, didRegisterForRemoteNotificationsWithDeviceToken deviceToken: Data) {
        NotificationsController.didRegisterForRemoteNotificationsWithDeviceToken(deviceToken)
    }

    func application(_ application: UIApplication, didFailToRegisterForRemoteNotificationsWithError error: Error) {
        NotificationsController.didFailToRegisterForRemoteNotificationsWithError(error)
    }

    func application(_ application: UIApplication, didReceiveRemoteNotification userInfo: [AnyHashable: Any]) {
        NotificationsController.didReceiveRemoteNotification(userInfo, inForeground: self.inForeground)
    }

    func applicationWillResignActive(_ application: UIApplication) {
        // Sent when the application is about to move from active to inactive state. This can occur for certain types of temporary interruptions (such as an incoming phone call or SMS message) or when the user quits the application and it begins the transition to the background state.
        // Use this method to pause ongoing tasks, disable timers, and throttle down OpenGL ES frame rates. Games should use this method to pause the game.
    }

    func applicationDidEnterBackground(_ application: UIApplication) {
        // Use this method to release shared resources, save user data, invalidate timers, and store enough application state information to restore your application to its current state in case it is terminated later.
        // If your application supports background execution, this method is called instead of applicationWillTerminate: when the user quits.
        self.inForeground = false
        print("AppDelegate > App in background")
    }

    func applicationWillEnterForeground(_ application: UIApplication) {
        // Called as part of the transition from the background to the inactive state; here you can undo many of the changes made on entering the background.
        print("AppDelegate > App in foreground")
        NotificationsController.clearNotifications()
    }

    func applicationDidBecomeActive(_ application: UIApplication) {
        // Restart any tasks that were paused (or not yet started) while the application was inactive. If the application was previously in the background, optionally refresh the user interface.
        self.inForeground = true
        print("AppDelegate > App active")
    }

    func applicationWillTerminate(_ application: UIApplication) {
        // Called when the application is about to terminate. Save data if appropriate. See also applicationDidEnterBackground:.
    }

}

