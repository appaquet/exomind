import Foundation
import Exocore

class ExocoreUtils {

    static func initialize() {
        let config = """
                     ---
                     keypair: ae4WbDdfhv3416xs8S2tQgczBarmR8HKABvPCmRcNMujdVpDzuCJVQADVeqkqwvDmqYUUjLqv7kcChyCYn8R9BNgXP
                     public_key: pe5ZG43uAcfLxYSGaQgj1w8hQT4GBchEVg5mS2b1EfXcMb
                     name: ios
                     id: ""
                     path: "./local_conf"
                     listen_addresses:
                       - /ip4/0.0.0.0/tcp/0
                     cells:
                       - location:
                           Instance:
                             public_key: pe2AgPyBmJNztntK9n4vhLuEYN8P2kRfFXnaZFsiXqWacQ
                             keypair: ae55Nfv11ppyFVxCDaYovcxTcaTDaSzSFjiVoiC3VwGARfEuaqGcgoJUdVpqfwKQVDN4rvGKUvt4yqQc6w7it7PCpG
                             name: ""
                             id: ""
                             path: "./local_conf/./cell"
                             nodes:
                               - node:
                                   public_key: peAmfSDJn7x58GK8gPog8zjGcQVyJ8sBhanvWg2TsSmWZv
                                   name: di
                                   id: ""
                                   addresses:
                                     - /ip4/165.22.230.192/tcp/3330
                                     - /ip4/165.22.230.192/tcp/3341/ws
                                 roles:
                                   - 1
                                   - 2
                               - node:
                                   public_key: pe5ZG43uAcfLxYSGaQgj1w8hQT4GBchEVg5mS2b1EfXcMb
                                   name: ios
                                   id: ""
                                   addresses: []
                                 roles: []
                             apps:
                               - location:
                                   Instance:
                                     name: Exomind
                                     public_key: pe5PPd5U5t44CrECvDzbwBNeeeWWXwQkbGNoBoAMb3LjoJ
                                     path: "./local_conf/./cell/../../../exomind"
                                     schemas: []
                     """

        try! ExocoreClient.initialize(yamlConfig: config)
    }
}
