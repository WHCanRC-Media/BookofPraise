melody = \relative c' {
  \clef treble
  \key bes \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r8 g'2 g4 f4 g4 a4 bes2 r2 \break

  % Line 2
  bes2 c4 bes4 a4 g4 f2 r2 \break

  % Line 3
  a2 bes4 d4 c4 bes4 a2 g2 r2 \break

  % Line 4
  d2 g4 f4 g4 ees4 d2 r2 \break

  % Line 5
  g2 bes4 a4 bes4 c4 a2 r2 \break

  % Line 6
  d2 a2 c2 bes4 c4 a2 g1 \bar "|."
}
