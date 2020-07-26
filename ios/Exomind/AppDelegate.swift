//
//  AppDelegate.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2015-10-06.
//  Copyright © 2015 Exomind. All rights reserved.
//

import UIKit
import KeychainSwift
import GoogleSignIn

@UIApplicationMain
class AppDelegate: UIResponder, UIApplicationDelegate, GIDSignInDelegate {

    var window: UIWindow?
    var inForeground: Bool = true
    
    var googleSigninCallback: ((GIDGoogleUser) -> Void)?

    func application(_ application: UIApplication, didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?) -> Bool {
        HttpUtils.copyCookiesToKeychain()

        let gidSignIn = GIDSignIn.sharedInstance()
        gidSignIn?.delegate = self
        gidSignIn?.clientID = "1054868499665-2jnactlmaqpl24i4nfrf5mk007cu6t5i.apps.googleusercontent.com"
        gidSignIn?.serverClientID = "1054868499665-ikokqsknv905fn5lipkrglovnnfqjgha.apps.googleusercontent.com"
        gidSignIn?.scopes = ["https://mail.google.com/"]
        
        HCNamespaces.registerNamespace(ExomindNamespace())
        let websocketBridgeFactory = RealWebSocketBridgeFactory()
        let ajaxBridgeFactory = RealXMLHttpRequestBridgeFactory()
        DomainStore.instance = DomainStore(serverHost: "exomind.io", webSocketBridgeFactory: websocketBridgeFactory, ajaxBridgeFactory: ajaxBridgeFactory)

        ExocoreUtils.initialize()

        return true
    }

    func application(_ application: UIApplication, performActionFor shortcutItem: UIApplicationShortcutItem, completionHandler: @escaping (Bool) -> Void) {
        func openObject(_ entity: HCEntity?) {
            if let entity = entity {
                RootNavigationController.mainInstance()?.show(navigationObject: .entityOld(entity: entity))
            }
        }

        if (shortcutItem.type.contains("NewNote")) {
            AddSelectionViewController.createNote(nil, callback: openObject)
        } else if (shortcutItem.type.contains("NewTask")) {
            AddSelectionViewController.createTask(nil, callback: openObject)
        } else if (shortcutItem.type.contains("NewEmail")) {
            AddSelectionViewController.createEmail(nil, callback: openObject)
        } else if (shortcutItem.type.contains("NewCollection")) {
            AddSelectionViewController.createCollection(nil, callback: openObject)
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
    
    func application(_ app: UIApplication, open url: URL, options: [UIApplication.OpenURLOptionsKey : Any] = [:]) -> Bool {
        return GIDSignIn.sharedInstance().handle(url,
                                                    sourceApplication:options[UIApplication.OpenURLOptionsKey.sourceApplication] as? String,
                                                    annotation: [:])
    }
    
    func sign(_ signIn: GIDSignIn!, didSignInFor user: GIDGoogleUser!, withError error: Error!) {
        print("Got sigin from google for user \(String(describing: user))")
        self.googleSigninCallback?(user)
    }
    
    func sign(_ signIn: GIDSignIn!, didDisconnectWith user: GIDGoogleUser!, withError error: Error!) {
        print("Got disconnect from google \(String(describing: user)))")
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
        DomainStore.instance.pauseConnections()
    }

    func applicationWillEnterForeground(_ application: UIApplication) {
        // Called as part of the transition from the background to the inactive state; here you can undo many of the changes made on entering the background.
        print("AppDelegate > App in foreground")
        NotificationsController.clearNotifications()
        DomainStore.instance.resumeConnections()
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

