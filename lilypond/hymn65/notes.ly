melody = \relative c' {
  \clef treble
  \key bes \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  a'4 d,4 g4 a4 bes2 a4 g2 a4 fis4 d4 r4 \break

  % Line 2
  f4 f4 ees4 d2 g4 g2 fis4 g2 \break

  % Line 3
  d4 g4 a4 bes2 a4 g2 a4 fis4 d4 r4 \break

  % Line 4
  f4 f4 ees4 d2 g4 g2 fis4 g2 \break

  % Line 5
  a4 bes4 c4 d2 d4 c2 c4 bes2 \break

  % Line 6
  d4 c4 bes4 a2 g4 c2 a4 g2 \bar "|."
}
