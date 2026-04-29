melody = \relative c' {
  \clef treble
  \key bes \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  g'1 g2 ees2 f2 g2 ees2 d2 c1 \break

  % Line 2
  g'1 g2 f2 bes2 g2 ees2 f2 g1 \break

  % Line 3
  g1 bes2 c2 ees2 d2 c2 bes2 c1 \break

  % Line 4
  c1 d2 c2 bes2 a2 g2 fis2 g1 \break

  % Line 5
  c1 bes2 a2 bes2 g2 g2 f2 ees1 \break

  % Line 6
  g1 aes2 g2 ees2 f2 ees2 d2 c1 \bar "|."
}
