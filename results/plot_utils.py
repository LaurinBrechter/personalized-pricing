import matplotlib.pyplot as plt
import seaborn as sns

def plot_event_history(ax, data):

    sns.scatterplot(x="t", y="customer", data=data, hue="event", ax=ax)
    ax.grid(axis="y")
    ax.yaxis.set_major_locator(plt.MultipleLocator(1))
    ax.set_ylim(ax.get_ylim()[0] - 0.5, ax.get_ylim()[1] + 0.5)
    ax.yaxis.set_major_locator(plt.MultipleLocator(1))
    ax.yaxis.set_minor_locator(
        plt.MultipleLocator(0.5)
    )
    ax.grid(True, axis="y", which="minor")
    ax.grid(False, axis="y", which="major")