
# Timestitch design

There are two use-cases supported by this tool:

1. "Journal mode": Constructing formatted overviews of data by period, sourced from a collection of implicitely per-day files (i.e., files whose naming scheme encodes the timeline position of the data contained within.)
2. "Timeline mode": Constructing formatted overvies of data by period, sourced from a collection of explicitly annotated date-time data fields without any particulare naming scheme.

To support both of these, we want to organize the inputs and process them into a uniform intermediate representation. From this representation, we can generate the desired output.

The following steps are an outline of how things should behave:

1. Process files
	+ File names are processed by NameMatcher
	+ File contents are processed by either deserialization or FileMatcher if deserialization fails.
	+ Matcher parameters are provided in the Prefs file.
	+ The result is a list of entries.
2. Process entries
	+ Entries are sorted and wrapped for rendering.
	+ Sort keys are provided by the Prefs file.
3. Write entries
	+ Output is written by table-gen-rs.

