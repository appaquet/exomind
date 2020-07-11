//
//  Debounce.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2016-12-16.
//  Copyright © 2016 Exomind. All rights reserved.
//

import Foundation

class Debouncer {
    
    // http://stackoverflow.com/questions/27116684/how-can-i-debounce-a-method-call
    static func debounce(delay:Int, queue:DispatchQueue, action: @escaping (()->())) -> ()->() {
        var lastFireTime = DispatchTime.now()
        let dispatchDelay = DispatchTimeInterval.milliseconds(delay)
        
        return {
            let dispatchTime: DispatchTime = lastFireTime + dispatchDelay
            queue.asyncAfter(deadline: dispatchTime, execute: {
                let when: DispatchTime = lastFireTime + dispatchDelay
                let now = DispatchTime.now()
                if now.rawValue >= when.rawValue {
                    lastFireTime = DispatchTime.now()
                    action()
                }
            })
        }
    }
    
}
