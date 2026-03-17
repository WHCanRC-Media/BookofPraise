melody = \relative c' {
  \clef treble
  \key bes \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r8 f2 g2 f4 bes2 a4 g4 g4 f2 r2 \break

  % Line 2
  bes2 bes4 a4 bes2 c2 d4 \break

  % Line 3
  d4 c4 bes4 bes4 a4 bes2 r2 \break

  % Line 4
  f2 g4 a4 bes4 a2 g2 fis4 g2 r2 \break

  % Line 5
  bes2 a4 g4 f4 ees4 d2 c4 \break

  % Line 6
  f4 f4 g4 a2 bes2 g2 f1 \bar "|."
}
