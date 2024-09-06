import UIKit
import KeychainSwift
import Reachability
import AppCenter
import AppCenterAnalytics
import AppCenterCrashes

@UIApplicationMain
class AppDelegate: UIResponder, UIApplicationDelegate {
    var window: UIWindow?
    var inForeground: Bool = true
    var reach: Reachability?

    func application(_ application: UIApplication, didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?) -> Bool {
        AppCenter.start(withAppSecret: "fd71ff63-e602-441d-8fc6-26932fcf55de", services: [
            Analytics.self,
            Crashes.self
        ])

        HttpUtils.copyCookiesToKeychain()
        JSBridge.instance = JSBridge()

        try? ExocoreUtils.initialize()
        self.startNetworkMonitoring()

        return true
    }

    func application(_ application: UIApplication, performActionFor shortcutItem: UIApplicationShortcutItem, completionHandler: @escaping (Bool) -> Void) {
        let openObject = { (res: EntityCreateResult) in
            if case let .success(entity) = res, let entity = entity {
                RootViewController.mainInstance()?.show(navigationObject: .entity(entity: entity))
            }
        }

        if (shortcutItem.type.contains("NewNote")) {
            Commands.createNote(nil, callback: openObject)
        } else if (shortcutItem.type.contains("NewTask")) {
            Commands.createTask(nil, callback: openObject)
        } else if (shortcutItem.type.contains("NewCollection")) {
            Commands.createCollection(nil, callback: openObject)
        }
    }

    func application(_ application: UIApplication, didRegisterForRemoteNotificationsWithDeviceToken deviceToken: Data) {
        Notifications.didRegisterForRemoteNotificationsWithDeviceToken(deviceToken)
    }

    func application(_ application: UIApplication, didFailToRegisterForRemoteNotificationsWithError error: Error) {
        Notifications.didFailToRegisterForRemoteNotificationsWithError(error)
    }

    func application(_ application: UIApplication, didReceiveRemoteNotification userInfo: [AnyHashable: Any]) {
        Notifications.didReceiveRemoteNotification(userInfo, inForeground: self.inForeground)
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
        Notifications.clearNotifications()
    }

    func applicationDidBecomeActive(_ application: UIApplication) {
        // Restart any tasks that were paused (or not yet started) while the application was inactive. If the application was previously in the background, optionally refresh the user interface.
        self.inForeground = true
        print("AppDelegate > App active")
    }

    private var previousReachability: Reachability?

    private func startNetworkMonitoring() {
        self.reach = try? Reachability()
        self.reach?.whenReachable = { reachability in
            if reachability.connection == .wifi {
                print("AppDelegate > Reachable via WiFi")
            } else {
                print("AppDelegate > Reachable via Cellular")
            }

            // we don't reset on first reachability change (on start)
            if self.previousReachability != nil {
                ExocoreUtils.resetTransport()
            }
            self.previousReachability = reachability
        }

        do {
            try self.reach?.startNotifier()
        } catch {
            print("AppDelegate > Unable to start notifier")
        }
    }

    func applicationWillTerminate(_ application: UIApplication) {
        // Called when the application is about to terminate. Save data if appropriate. See also applicationDidEnterBackground:.
    }
}
