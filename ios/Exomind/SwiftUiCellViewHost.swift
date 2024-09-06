import Foundation
import SwiftUI

// Inspired from
//  - https://stackoverflow.com/questions/59881164/uitableview-with-uiviewrepresentable-in-swiftui
//  - https://github.com/noahsark769/NGSwiftUITableCellSizing/blob/main/NGSwiftUITableCellSizing/HostingCell.swift
class SwiftUICellViewHost<ContentView: View>: UITableViewCell {
    var host: UIHostingController<ContentView?> = UIHostingController(rootView: nil)

    override init(style: UITableViewCell.CellStyle, reuseIdentifier: String?) {
        super.init(style: style, reuseIdentifier: reuseIdentifier)
        host.view.backgroundColor = .clear
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func setView(view: ContentView, parentController: UIViewController) {
        self.host.rootView = view

        let requiresControllerMove = self.host.parent != parentController
        if requiresControllerMove {
            parentController.addChild(self.host)
        }

        let hostView = self.host.view!
        hostView.invalidateIntrinsicContentSize()

        if !self.contentView.subviews.contains(hostView) {
            self.contentView.addSubview(hostView)
            hostView.translatesAutoresizingMaskIntoConstraints = false
            hostView.topAnchor.constraint(equalTo: self.contentView.topAnchor).isActive = true
            hostView.leftAnchor.constraint(equalTo: self.contentView.leftAnchor).isActive = true
            hostView.bottomAnchor.constraint(equalTo: self.contentView.bottomAnchor).isActive = true
            hostView.rightAnchor.constraint(equalTo: self.contentView.rightAnchor).isActive = true
        }

        if requiresControllerMove {
            self.host.didMove(toParent: parentController)
        }

        self.setNeedsLayout()
        self.layoutIfNeeded()
    }
}
