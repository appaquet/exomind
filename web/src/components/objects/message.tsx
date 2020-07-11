import * as React from 'react';

interface IProps {
    text: string;
    showAfterMs?: number;
}

interface IState {
    show: boolean;
}

export class Message extends React.Component<IProps, IState> {
    private mounted: boolean;

    constructor(props: IProps) {
        super(props);

        this.mounted = true;

        let show = true;
        if (props.showAfterMs != null) {
            show = false;
            setTimeout(() => {
                if (this.mounted) {
                    this.setState({
                        show: true,
                    });
                }
            }, props.showAfterMs);
        }
        this.state = { show };
    }

    componentWillUnmount(): void {
        this.mounted = false;
    }

    render(): React.ReactNode {
        return (
            <div className="entity-component">
                <div className="empty">
                    {this.state.show ? this.props.text : ''}
                </div>
            </div>
        );
    }
}

