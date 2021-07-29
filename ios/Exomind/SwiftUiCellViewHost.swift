import Foundation
import SwiftUI

class SwiftUICellViewHost: UITableViewCell {
    var host: UIHostingController<AnyView>

    init(view: AnyView) {
        self.host = UIHostingController(rootView: view)
        super.init(style: .default, reuseIdentifier: nil)

        // From https://stackoverflow.com/questions/59881164/uitableview-with-uiviewrepresentable-in-swiftui
        let cellContentView = self.host.view!
        cellContentView.translatesAutoresizingMaskIntoConstraints = false
        self.contentView.addSubview(cellContentView)
        cellContentView.topAnchor.constraint(equalTo: self.contentView.topAnchor).isActive = true
        cellContentView.leftAnchor.constraint(equalTo: self.contentView.leftAnchor).isActive = true
        cellContentView.bottomAnchor.constraint(equalTo: self.contentView.bottomAnchor).isActive = true
        cellContentView.rightAnchor.constraint(equalTo: self.contentView.rightAnchor).isActive = true

        self.setNeedsLayout()
        self.layoutIfNeeded()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}