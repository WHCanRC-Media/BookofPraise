melody = \relative c'' {
  \clef treble
  \key bes \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  g2 a2 f4 bes4 a4 g2 fis4 g2 r2 \break

  % Line 2
  d2 g4 f4 g2 bes2 bes4 a4 bes2 r2 \break

  % Line 3
  bes2 c2 d2 g,4 a4 bes2 g2 f2 r2 \break

  % Line 4
  f2 g4 bes4 f2 g2 f4 ees4 d1 \bar "|."
}
