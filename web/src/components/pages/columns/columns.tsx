import classNames from 'classnames';
import { observer } from 'mobx-react';
import React from 'react';
import { SelectedItem, Selection } from '../../objects/entity-list/selection';
import Column from './column';
import { ColumnConfig, ColumnConfigs } from './columns-config';
import { ListenerToken, Shortcuts } from '../../../shortcuts';
import './columns.less';

interface IProps {
    config: string;
    onConfigChange: (config: ColumnConfigs) => void;
}

interface IState {
    activeColumn: number;
}

@observer
export default class Columns extends React.Component<IProps, IState> {
    private shortcutToken?: ListenerToken;

    constructor(props: IProps) {
        super(props);
        this.state = {
            activeColumn: 0,
        };
    }

    componentDidMount(): void {
        this.shortcutToken = Shortcuts.register([
            {
                key: 'Ctrl-b ArrowRight',
                callback: this.handleShortcutNext,
            },
            {
                key: 'Ctrl-b ArrowLeft',
                callback: this.handleShortcutPrev,
            },
            {
                key: 'Ctrl-b x',
                callback: this.handleShortcutClose,
            }
        ]);
    }

    componentWillUnmount(): void {
        if (this.shortcutToken) {
            Shortcuts.unregister(this.shortcutToken);
        }
    }

    render(): React.ReactNode {
        const renderedColumns = this.renderColumns();
        const nbColumns = renderedColumns.length;
        const classes = `columns count-${nbColumns}`;
        return (
            <div className={classes}>
                {renderedColumns}
            </div>
        );
    }

    private renderColumns() {
        const config = this.getConfig();
        return config.parts.flatMap((columnConfig, colId) => {
            if (columnConfig.isMultiple) {
                return [];
            }

            let selectionItems: SelectedItem[] = [];
            const nextColumnConfig = config.parts[colId + 1];
            if (nextColumnConfig) {
                if (nextColumnConfig.isMultiple) {
                    selectionItems = Array.from(nextColumnConfig.values.map((col) => this.configToSelection(col)));
                } else {
                    selectionItems = [this.configToSelection(nextColumnConfig)];
                }
            }
            const selection = new Selection(Array.from(selectionItems.filter((col => !!col))));

            const colClass = `column-container-${colId}`;
            const classes = classNames({
                'column-container': true,
                [colClass]: true
            });
            const colKey = this.columnKey(columnConfig);
            const active = this.state.activeColumn == colId;

            return [(
                <div
                    className={classes}
                    key={colKey}
                    onMouseEnter={() => this.onColumnHovered(colId)}
                    onMouseOver={() => this.onColumnHovered(colId)}>

                    <Column
                        key={colKey}
                        columnId={colId}
                        columnConfig={columnConfig}
                        active={active}
                        onClose={() => this.handleColumnClose(colId)}
                        selection={selection}
                        onSelectionChange={(selection) => this.handleColumnItemSelect(colId, selection)}
                    />
                </div>
            )];
        });
    }

    private handleShortcutNext = () => {
        let activeColumn = this.state.activeColumn;
        if (activeColumn >= this.getConfig().parts.length - 1) {
            activeColumn = 0;
        } else {
            activeColumn++;
        }

        this.setState({ activeColumn });

        return true;
    }

    private handleShortcutPrev = () => {
        let activeColumn = this.state.activeColumn;
        if (activeColumn == 0) {
            activeColumn = this.getConfig().parts.length - 1;
        } else {
            activeColumn--;
        }

        this.setState({ activeColumn });

        return true;
    }

    private handleShortcutClose = () => {
        this.handleColumnClose(this.state.activeColumn);
        return true;
    }

    private onColumnHovered(columnId: number): void {
        this.setState({
            activeColumn: columnId
        })
    }

    private columnKey(config: ColumnConfig): string {
        if (config.isSearch) {
            // prevent recreating component for every keystroke
            return 'search';
        } else {
            return config.toString();
        }
    }

    private configToSelection(config: ColumnConfig): SelectedItem {
        if (config.isEntity) {
            return SelectedItem.fromEntityId(config.first);
        } else if (config.isTrait) {
            return SelectedItem.fromEntityTraitId(config.first, config.second);
        }
    }

    private getConfig(props?: IProps): ColumnConfigs {
        props = props || this.props;
        let config = ColumnConfigs.fromString(props.config);
        if (config.empty) {
            config = ColumnConfigs.forInbox();
        }
        return config;
    }

    private handleColumnItemSelect(colId: number, selection: Selection) {
        let columnsConfig = this.getConfig();
        if (selection && !selection.isEmpty) {
            const selectionConfigs = Array.from(selection.items.map((item) => {
                if (item.traitId) {
                    return ColumnConfig.forTrait(item.entityId, item.traitId);
                } else {
                    return ColumnConfig.forEntity(item.entityId);
                }
            }));

            let selectionConfig: ColumnConfig;
            if (selectionConfigs.length == 1) {
                selectionConfig = selectionConfigs[0];
            } else if (selectionConfigs.length > 1) {
                selectionConfig = ColumnConfig.forMultiple(selectionConfigs);
            }

            if (selectionConfig) {
                columnsConfig = columnsConfig.set(colId + 1, selectionConfig);
            }

        } else {
            columnsConfig = columnsConfig.unset(colId + 1);
        }

        this.props.onConfigChange(columnsConfig);
    }

    private handleColumnClose(colId: number) {
        const config = this.getConfig();

        // we allow closing any columns, as long as it is not the last one
        const canClose = colId > 0 || config.parts.length > 1;
        if (!canClose) {
            return;
        }

        const columnsConfig = config.pop(colId);
        this.props.onConfigChange(columnsConfig);
    }
}

