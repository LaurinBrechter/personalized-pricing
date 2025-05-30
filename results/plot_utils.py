import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns

def plot_event_history(ax, data):

    sns.scatterplot(x="t", y="customer", data=data, hue="event", ax=ax)
    ax.grid(axis="y")
    ax.yaxis.set_major_locator(plt.MultipleLocator(1))
    ax.set_ylim(0,60)
    ax.yaxis.set_major_locator(plt.MultipleLocator(1))
    ax.yaxis.set_minor_locator(
        plt.MultipleLocator(0.5)
    )
    ax.grid(True, axis="y", which="minor")
    ax.grid(False, axis="y", which="major")


def read_event_history(file_path):
    """
    Read the event history from a file and return it as a pandas DataFrame.
    """
    # evolution_log = pd.read_csv("evolution_log.csv")
    event_history = pd.read_csv(file_path).dropna().sort_values(by="event")
    event_history.t = event_history.t.astype(float)
    num_customers = event_history.customer.max() + 1

    event_history['welfare'] = 0
    event_history.loc[event_history.event == 'sold', 'welfare'] = event_history['customer_wtp'] - event_history['price']
    event_history['profit'] = 0
    event_history.loc[event_history.event == 'sold', 'profit'] = event_history['price']
    event_history.customer = event_history.customer.astype(str)
    event_history.run_id = event_history.run_id.astype(int)
    event_history.actual_group = event_history.actual_group.replace({
        0.0: "mid",
        1.0: "high",
        2.0: "low",
    })

    return event_history, num_customers