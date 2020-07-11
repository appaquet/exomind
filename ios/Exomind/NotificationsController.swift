//
//  NotificationsController.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2016-02-01.
//  Copyright © 2016 Exomind. All rights reserved.
//

import UIKit
import UserNotifications

class NotificationsController {
    static func maybeRegister() {
        if (!self.isRemoteRegistered()) {
            self.registerRemote()
        }
    }

    static func registerRemote() {
        print("NotificationsController > Trying to register for remote notification")
        let center = UNUserNotificationCenter.current()
        center.requestAuthorization(options: [.alert, .sound, .badge]) { (granted, error) in
            // Enable or disable features based on authorization.
        }
        UIApplication.shared.registerForRemoteNotifications()
    }

    static func isRemoteRegistered() -> Bool {
        if (!UIApplication.shared.isRegisteredForRemoteNotifications) {
            return false
        } else {
            return !getApplePushIntegrations().isEmpty
        }
    }
    
    static func getApplePushIntegrations() -> [Integration] {
        return SessionStore.integrations()
            .compactMap { entityTrait in
                switch (entityTrait.traitType) {
                case let .integration(integration: int) where int.typ == "apple_push":
                    return int
                default:
                    return nil
                }
            }
    }

    static func didFailToRegisterForRemoteNotificationsWithError(_ error: Error) {
        print("NotificationsController > Fail registering for remote notification \(error)")
    }

    static func didRegisterForRemoteNotificationsWithDeviceToken(_ deviceToken: Data) {
        // from http://stackoverflow.com/questions/9372815/how-can-i-convert-my-device-token-nsdata-into-an-nsstring
        let tokenChars = (deviceToken as NSData).bytes.bindMemory(to: CChar.self, capacity: deviceToken.count)
        var tokenString = ""
        for i in 0 ..< deviceToken.count {
            tokenString += String(format: "%02.2hhx", arguments: [tokenChars[i]])
        }

        ExomindDSL
            .on(HCEntity(id: tokenString, traits: []))
            .mutate
            .put(IntegrationFull(data: ["device_token" : tokenString], key: tokenString, typ: "apple_push"))
            .execute()
        
        print("NotificationsController > Successfully registered for remote notification with token \(tokenString)")
    }

    static func didReceiveRemoteNotification(_ payload: [AnyHashable: Any], inForeground: Bool) {
        print("NotificationsController > Received remote notification \(payload) \(inForeground)")

        // only show up if we are coming from background, which means we clicked on notification
        if (!inForeground) {
            if let entityId = payload["object_id"] as? String {
                RootNavigationController.mainInstance()?.show(navigationObject: .entityId(id: entityId))
            }
        }
    }

    static func clearNotifications() {
        UNUserNotificationCenter.current().removeAllDeliveredNotifications()
    }
}
